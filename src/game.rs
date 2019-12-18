use crate::components::*;
use crate::generate_dungeon::{generate_dungeon, Room};
use crate::movement::Direction;
use crate::spawn_enemies::spawn_enemies;
use legion::entity::Entity;
use legion::query::{IntoQuery, Read};
use legion::world::World;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub struct Game {
    pub world: World,
    pub player_entity: Entity,
    pub rooms: Vec<Room>,
    pub floor_positions: HashSet<PositionComponent>,
    pub floor_number: u32,

    pub rng: Pcg64,
    pub dungeon_generation_rng: Pcg64,

    pub message_log: Vec<Message>,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();
        let rooms = Vec::new();
        let floor_positions = HashSet::new();
        let floor_number = 0;

        let mut rng = Pcg64::from_entropy();
        let dungeon_generation_rng = Pcg64::from_entropy();

        let message_log = Vec::with_capacity(100);

        let max_health = rng.gen_range(12, 31);
        let player_entity = world.insert(
            (),
            Some((
                NameComponent {
                    name: "Player",
                    concealed_name: "???",
                    is_concealed: false,
                },
                PositionComponent { x: 0, y: 0 },
                SpriteComponent { id: "player" },
                PlayerComponent {
                    facing_direction: Direction::Up,
                    inventory: [None; 16],
                    turns_before_passive_healing: 10,
                },
                StatsComponent {
                    current_health: max_health,
                    max_health,
                    strength: rng.gen_range(1, 13),
                    luck: rng.gen_range(1, 13),
                    agility: rng.gen_range(1, 13),
                    focus: rng.gen_range(1, 13),
                },
                TeamComponent::Ally,
            )),
        )[0];

        let mut game = Self {
            world,
            player_entity,
            rooms,
            floor_positions,
            floor_number,

            rng,
            dungeon_generation_rng,

            message_log,
        };

        generate_dungeon(&mut game);
        spawn_enemies(&mut game);

        game
    }

    pub fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
        time_since_last_frame: Duration,
    ) {
        let player = *self
            .world
            .get_component::<PlayerComponent>(self.player_entity)
            .unwrap();
        let player_position = *self
            .world
            .get_component::<PositionComponent>(self.player_entity)
            .unwrap();

        // Render floor tiles
        {
            for position in self
                .floor_positions
                .iter()
                .map(|position| PositionComponent {
                    x: position.x - player_position.x + 7,
                    y: player_position.y - position.y + 7,
                })
                .filter(|position| (0..15).contains(&position.x))
                .filter(|position| (0..15).contains(&position.y))
            {
                let texture = texture_creator.load_texture("assets/floor.png").unwrap();
                let dest_rect =
                    Rect::new((position.x * 32) as i32, (position.y * 32) as i32, 32, 32);
                canvas.copy(&texture, None, dest_rect).unwrap();
            }
        }

        // Render entities
        {
            for (sprite, position) in <(Read<SpriteComponent>, Read<PositionComponent>)>::query()
                .iter_immutable(&self.world)
                .map(|(sprite, position)| {
                    (
                        sprite,
                        PositionComponent {
                            x: position.x - player_position.x + 7,
                            y: player_position.y - position.y + 7,
                        },
                    )
                })
                .filter(|(_, position)| (0..15).contains(&position.x))
                .filter(|(_, position)| (0..15).contains(&position.y))
            {
                let texture = texture_creator
                    .load_texture(format!("assets/{}.png", sprite.id))
                    .unwrap();
                let dest_rect =
                    Rect::new((position.x * 32) as i32, (position.y * 32) as i32, 32, 32);
                canvas.copy(&texture, None, dest_rect).unwrap();
            }
        }

        // Render player facing_direction indicator
        {
            let texture_id = match player.facing_direction {
                Direction::Up | Direction::Down | Direction::Left | Direction::Right => {
                    "assets/direction_indicator.png"
                }
                Direction::UpLeft
                | Direction::DownLeft
                | Direction::DownRight
                | Direction::UpRight => "assets/direction_indicator_diagonal.png",
            };
            let texture = texture_creator.load_texture(texture_id).unwrap();
            let dest_rect = Rect::new(224, 224, 32, 32);
            let rotation = match player.facing_direction {
                Direction::Up => 90.0,
                Direction::Down => 270.0,
                Direction::Left => 0.0,
                Direction::Right => 180.0,
                Direction::UpLeft => 0.0,
                Direction::DownLeft => 270.0,
                Direction::DownRight => 180.0,
                Direction::UpRight => 90.0,
            };
            canvas
                .copy_ex(&texture, None, dest_rect, rotation, None, false, false)
                .unwrap();
        }
    }

    pub fn recent_messages(&self) -> impl Iterator<Item = &Message> {
        self.message_log
            .iter()
            .rev()
            .take_while(|message| message.time_created.elapsed() <= Duration::from_secs(4))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Message {
    pub text: String,
    pub color: MessageColor,
    pub time_created: Instant,
}

impl Message {
    pub fn new(text: String, color: MessageColor) -> Self {
        Self {
            text,
            color,
            time_created: Instant::now(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MessageColor {
    White,
}

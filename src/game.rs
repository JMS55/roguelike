use crate::components::*;
use crate::entities;
use crate::generate_dungeon::{generate_dungeon, Room};
use crate::spawn_enemies::spawn_enemies;
use hecs::{Entity, World};
use noise::{NoiseFn, OpenSimplex};
use rand::SeedableRng;
use rand_pcg::Pcg64;
use sdl2::image::LoadTexture;
use sdl2::pixels::PixelFormatEnum;
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

    pub rng: RNG,
    pub dungeon_generation_rng: RNG,

    pub message_log: Vec<Message>,

    noise_generator: OpenSimplex,
    time_game_started: Instant,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut rng = RNG::from_entropy();
        let player_entity = entities::create_player(&mut world, &mut rng);

        let mut game = Self {
            world,
            player_entity,
            rooms: Vec::new(),
            floor_positions: HashSet::new(),
            floor_number: 0,

            rng,
            dungeon_generation_rng: RNG::from_entropy(),

            message_log: Vec::with_capacity(100),

            noise_generator: OpenSimplex::new(),
            time_game_started: Instant::now(),
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
    ) {
        let player = *self
            .world
            .get::<PlayerComponent>(self.player_entity)
            .unwrap();
        let player_position = *self
            .world
            .get::<PositionComponent>(self.player_entity)
            .unwrap();
        let player_stats = *self
            .world
            .get::<StatsComponent>(self.player_entity)
            .unwrap();

        // Render background
        if !cfg!(debug_assertions) {
            // Determine color modifier
            let (red_modifier, green_modifier, blue_modifier) =
                if (player_stats.current_health as f64 / player_stats.max_health as f64) < 0.3 {
                    (1.7, 0.2, 0.4)
                } else {
                    (1.0, 1.0, 1.8)
                };

            // Create pixel data
            let mut pixel_data: [u8; 480 * 480 * 3] = [0; 480 * 480 * 3];
            let time_since_game_started = self.time_game_started.elapsed().as_secs_f64();
            let mut i = 0;
            for y in 0..480 {
                for x in 0..480 {
                    let mut n = self.noise_generator.get([
                        x as f64 / 96.0,
                        y as f64 / 96.0,
                        time_since_game_started,
                    ]);
                    // If tile is neighboring player then draw the background slightly lighter
                    let m = if x >= 192 && x <= 288 && y >= 192 && y <= 288 {
                        40.0
                    } else {
                        16.0
                    };
                    n = (n + 1.0) * m;
                    pixel_data[i] = (n * red_modifier).round() as u8;
                    pixel_data[i + 1] = (n * green_modifier).round() as u8;
                    pixel_data[i + 2] = (n * blue_modifier).round() as u8;
                    i += 3;
                }
            }

            // Copy pixel data to canvas
            let mut texture = texture_creator
                .create_texture_static(PixelFormatEnum::RGB24, 480, 480)
                .unwrap();
            texture.update(None, &pixel_data, 480 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
        }

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
            for (sprite, position) in self
                .world
                .query::<(&SpriteComponent, &PositionComponent)>()
                .iter()
                .map(|(_, (sprite, position))| {
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
            let texture_id = if player.facing_direction.x == 0 || player.facing_direction.y == 0 {
                "assets/direction_indicator.png"
            } else {
                "assets/direction_indicator_diagonal.png"
            };
            let texture = texture_creator.load_texture(texture_id).unwrap();
            let dest_rect = Rect::new(224, 224, 32, 32);
            let rotation = match player.facing_direction {
                PositionComponent { x: 0, y: 1 } => 90.0,
                PositionComponent { x: 0, y: -1 } => 270.0,
                PositionComponent { x: -1, y: 0 } => 0.0,
                PositionComponent { x: 1, y: 0 } => 180.0,
                PositionComponent { x: -1, y: 1 } => 0.0,
                PositionComponent { x: -1, y: -1 } => 270.0,
                PositionComponent { x: 1, y: -1 } => 180.0,
                PositionComponent { x: 1, y: 1 } => 90.0,
                _ => unreachable!(),
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

pub type RNG = Pcg64;

use crate::data::{Direction, GameState, MessageColor, MessageLog, Player, Position, Sprite};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::Sdl;
use specs::{Join, World, WorldExt};

pub struct RenderSystem {
    canvas: WindowCanvas,
    ttf_context: Sdl2TtfContext,
}

impl RenderSystem {
    pub fn new(sdl_context: &Sdl) -> Self {
        let video_context = sdl_context.video().unwrap();
        let ttf_context = sdl2::ttf::init().unwrap();
        let window = video_context
            .window("roguelike", 15 * 8 * 4, 15 * 8 * 4)
            .build()
            .unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();
        Self {
            canvas,
            ttf_context,
        }
    }

    pub fn run(&mut self, world: &mut World) {
        self.canvas.clear();
        let texture_creator = self.canvas.texture_creator();

        let entities = world.entities();
        let player_data = world.read_storage::<Player>();
        let position_data = world.read_storage::<Position>();
        let sprite_data = world.read_storage::<Sprite>();
        let mut message_log = world.fetch_mut::<MessageLog>();

        let game_state = *world.fetch::<GameState>();
        if game_state == GameState::PlayerTurn || game_state == GameState::EnemyTurn {
            let player_position = (&player_data, &position_data).join().next().unwrap().1;
            for (entity, entity_position, entity_sprite) in
                (&entities, &position_data, &sprite_data).join()
            {
                let adjusted_entity_position_x = entity_position.x - player_position.x + 7;
                let adjusted_entity_position_y = player_position.y - entity_position.y + 7;
                if (0..15).contains(&adjusted_entity_position_x)
                    && (0..15).contains(&adjusted_entity_position_y)
                {
                    let dest_rect = Rect::new(
                        adjusted_entity_position_x * 8 * 4,
                        adjusted_entity_position_y * 8 * 4,
                        8 * 4,
                        8 * 4,
                    );
                    let texture = texture_creator
                        .load_texture(format!("assets/{}.png", entity_sprite.id))
                        .unwrap();
                    self.canvas.copy(&texture, None, dest_rect).unwrap();
                    if player_data.get(entity).is_some() {
                        let texture = texture_creator
                            .load_texture("assets/direction_indicator.png")
                            .unwrap();
                        let rotation = match entity_position.facing_direction {
                            Direction::Up => 90.0,
                            Direction::Down => 270.0,
                            Direction::Left => 0.0,
                            Direction::Right => 180.0,
                        };
                        self.canvas
                            .copy_ex(&texture, None, dest_rect, rotation, None, false, false)
                            .unwrap();
                    }
                }
            }

            let font = self
                .ttf_context
                .load_font("assets/04B_03__.ttf", 16)
                .unwrap();
            let mut height_used = 0;
            for (index, message) in message_log.recent_messages().enumerate() {
                let mut alpha = 255;
                let time_since_message_creation = message.time_created.elapsed();
                let fade_time = message.display_length.duration().div_f32(2.0);
                if time_since_message_creation > fade_time {
                    let t = (time_since_message_creation - fade_time).div_duration_f32(fade_time);
                    alpha = (255.0 - (t * 255.0)).round() as u8;
                    alpha = alpha.max(1); // For some reason SDL2 seems to draw at full opacity if alpha = 0
                }
                let surface = font
                    .render(&format!("* {}", message.text))
                    .blended_wrapped(message.color.sdl_color(alpha), 15 * 8 * 4 - 4)
                    .unwrap();
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .unwrap();
                let texture_info = texture.query();
                height_used += texture_info.height + if index == 0 { 4 } else { 0 };
                if height_used < 15 * 8 * 4 {
                    let dest_rect = Rect::new(
                        4,
                        (height_used - texture_info.height) as i32,
                        texture_info.width,
                        texture_info.height,
                    );
                    self.canvas.set_draw_color(Color::RGBA(0, 0, 0, alpha));
                    self.canvas.set_blend_mode(BlendMode::Blend);
                    self.canvas.fill_rect(dest_rect).unwrap();
                    self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
                    self.canvas.set_blend_mode(BlendMode::None);
                    self.canvas.copy(&texture, None, dest_rect).unwrap();
                } else {
                    break;
                };
            }
        }

        self.canvas.present();
    }
}

impl MessageColor {
    pub fn sdl_color(self, alpha: u8) -> Color {
        let (r, g, b) = match self {
            MessageColor::White => (255, 255, 255),
            MessageColor::Orange => (255, 96, 0),
            MessageColor::Red => (255, 0, 0),
        };
        Color::RGBA(r, g, b, alpha)
    }
}

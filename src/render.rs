use crate::data::{
    Attackable, Direction, GameState, MessageColor, MessageLog, Player, Position, Sprite,
};
use noise::{NoiseFn, OpenSimplex};
use sdl2::image::LoadTexture;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::Sdl;
use specs::{Join, World, WorldExt};
use std::time::Instant;

pub struct RenderSystem {
    canvas: WindowCanvas,
    ttf_context: Sdl2TtfContext,
    noise: OpenSimplex,
    timer: Instant,
    previous_noise_t: f64,
    previous_noise_modifier: (f64, f64, f64),
}

impl RenderSystem {
    pub fn new(sdl_context: &Sdl) -> Self {
        let video_context = sdl_context.video().unwrap();
        let ttf_context = sdl2::ttf::init().unwrap();
        let window = video_context.window("roguelike", 480, 480).build().unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();
        Self {
            canvas,
            ttf_context,
            noise: OpenSimplex::new(),
            timer: Instant::now(),
            previous_noise_t: 0.0,
            previous_noise_modifier: (1.0, 1.0, 1.0),
        }
    }

    pub fn run(&mut self, world: &mut World) {
        self.canvas.clear();
        let texture_creator = self.canvas.texture_creator();

        let entities = world.entities();
        let player_data = world.read_storage::<Player>();
        let position_data = world.read_storage::<Position>();
        let sprite_data = world.read_storage::<Sprite>();
        let attackable_data = world.read_storage::<Attackable>();
        let mut message_log = world.fetch_mut::<MessageLog>();

        let game_state = *world.fetch::<GameState>();
        if game_state == GameState::PlayerTurn || game_state == GameState::EnemyTurn {
            if !cfg!(debug_assertions) {
                let player_attackable = (&player_data, &attackable_data).join().next().unwrap().1;
                let player_health_percentage =
                    player_attackable.current_health as f64 / player_attackable.max_health as f64;
                let mut t = self.timer.elapsed().as_secs_f64();
                let mut modifier = (1.0, 1.0, 1.0);
                if player_health_percentage <= 0.3 {
                    t *= 2.5;
                    modifier = (2.0, 0.0, 0.0);
                }
                t = (t * 0.02) + (self.previous_noise_t * 0.98);
                modifier.0 = (modifier.0 * 0.05) + (self.previous_noise_modifier.0 * 0.95);
                modifier.1 = (modifier.1 * 0.05) + (self.previous_noise_modifier.1 * 0.95);
                modifier.2 = (modifier.2 * 0.05) + (self.previous_noise_modifier.2 * 0.95);
                self.previous_noise_t = t;
                self.previous_noise_modifier = modifier;
                let mut pixel_data: [u8; 480 * 480 * 3] = [0; 480 * 480 * 3];
                for x in 0..480 {
                    for y in 0..480 {
                        let mut n = self.noise.get([x as f64 / 369.0, y as f64 / 369.0, t]);
                        n = (n + 1.0) * 32.0;
                        pixel_data[3 * (y * 480 + x)] = (n * modifier.0).round() as u8;
                        pixel_data[3 * (y * 480 + x) + 1] = (n * modifier.1).round() as u8;
                        pixel_data[3 * (y * 480 + x) + 2] = (n * modifier.2).round() as u8;
                    }
                }
                let mut texture = texture_creator
                    .create_texture_static(PixelFormatEnum::RGB24, 480, 480)
                    .unwrap();
                texture.update(None, &pixel_data, 480 * 3).unwrap();
                self.canvas.copy(&texture, None, None).unwrap();
            }

            let mut render_objects = (&entities, &position_data, &sprite_data)
                .join()
                .collect::<Vec<_>>();
            render_objects.sort_unstable_by_key(|(_, _, sprite)| sprite.in_foreground);
            let player_position = (&player_data, &position_data).join().next().unwrap().1;
            for (entity, entity_position, entity_sprite) in render_objects {
                let adjusted_entity_position_x = entity_position.x - player_position.x + 7;
                let adjusted_entity_position_y = player_position.y - entity_position.y + 7;
                if (0..15).contains(&adjusted_entity_position_x)
                    && (0..15).contains(&adjusted_entity_position_y)
                {
                    let dest_rect = Rect::new(
                        (adjusted_entity_position_x * 32) as i32,
                        (adjusted_entity_position_y * 32) as i32,
                        32,
                        32,
                    );
                    let texture = texture_creator
                        .load_texture(format!("assets/{}.png", entity_sprite.id))
                        .unwrap();
                    self.canvas.copy(&texture, None, dest_rect).unwrap();
                    if let Some(player) = player_data.get(entity) {
                        let texture = match player.facing_direction {
                            Direction::Up
                            | Direction::Down
                            | Direction::Left
                            | Direction::Right => "assets/direction_indicator.png",
                            Direction::UpLeft
                            | Direction::DownLeft
                            | Direction::DownRight
                            | Direction::UpRight => "assets/direction_indicator_diagonal.png",
                        };
                        let texture = texture_creator.load_texture(texture).unwrap();
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
                let fade_time = message.display_length.duration().div_f64(2.0);
                if time_since_message_creation > fade_time {
                    let t = (time_since_message_creation - fade_time).as_secs_f64()
                        / fade_time.as_secs_f64();
                    alpha = (255.0 - (t * 255.0)).round() as u8;
                    alpha = alpha.max(1); // For some reason SDL2 seems to draw at full opacity if alpha = 0
                }
                let surface = font
                    .render(&format!("* {}", message.text))
                    .blended_wrapped(message.color.sdl_color(alpha), 476)
                    .unwrap();
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .unwrap();
                let texture_info = texture.query();
                height_used += texture_info.height + if index == 0 { 4 } else { 0 };
                if height_used < 480 {
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

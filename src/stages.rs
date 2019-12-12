use crate::components::*;
use crate::game::Game;
use legion::query::{IntoQuery, Read};
use noise::{NoiseFn, OpenSimplex};
use sdl2::image::LoadTexture;
use sdl2::keyboard::KeyboardState;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::time::Instant;

pub trait Stage {
    fn input(&mut self, keyboard: &KeyboardState);

    fn update(&self) -> Option<Box<dyn Stage>>;

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
        delta_time: f64,
    );
}

pub struct NewGameStage {}

impl Stage for NewGameStage {
    fn input(&mut self, _: &KeyboardState) {}

    fn update(&self) -> Option<Box<dyn Stage>> {
        Some(Box::new(PlayerTurnStage {
            game: Game::new(),
            background_noise: OpenSimplex::new(),
            start_of_background_render: Instant::now(),
            previous_background_t: 0.0,
            previous_background_color_modifier: (1.0, 1.0, 1.0),
        }))
    }

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        _: &mut TextureCreator<WindowContext>,
        _: &mut Font,
        _: f64,
    ) {
        canvas.clear();
    }
}

pub struct PlayerTurnStage {
    game: Game,
    background_noise: OpenSimplex,
    start_of_background_render: Instant,
    previous_background_t: f64,
    previous_background_color_modifier: (f64, f64, f64),
}

impl Stage for PlayerTurnStage {
    fn input(&mut self, keyboard: &KeyboardState) {}

    fn update(&self) -> Option<Box<dyn Stage>> {
        None
    }

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
        mut delta_time: f64,
    ) {
        canvas.clear();

        // Render background
        {
            let player_combat = *self
                .game
                .world
                .get_component::<CombatComponent>(self.game.player_entity)
                .unwrap();
            let player_health_percentage =
                player_combat.current_health as f64 / player_combat.max_health as f64;
            let mut t = self.start_of_background_render.elapsed().as_secs_f64();
            let mut color_modifier = if player_health_percentage <= 0.3 {
                (2.0, 0.0, 0.0)
            } else {
                (1.0, 1.0, 1.0)
            };

            // Determine t and color_modifier values
            delta_time *= 0.1;
            t = t * delta_time + self.previous_background_t * (1.0 - delta_time);
            color_modifier.0 = color_modifier.0 * delta_time
                + self.previous_background_color_modifier.0 * (1.0 - delta_time);
            color_modifier.1 = color_modifier.1 * delta_time
                + self.previous_background_color_modifier.1 * (1.0 - delta_time);
            color_modifier.2 = color_modifier.2 * delta_time
                + self.previous_background_color_modifier.2 * (1.0 - delta_time);
            self.previous_background_t = t;
            self.previous_background_color_modifier = color_modifier;

            // Create pixel data
            let mut pixel_data: [u8; 480 * 480 * 3] = [0; 480 * 480 * 3];
            let mut i = 0;
            for y in 0..480 {
                for x in 0..480 {
                    let mut n = self
                        .background_noise
                        .get([x as f64 / 256.0, y as f64 / 256.0, t]);
                    n = (n + 1.0) * 32.0;
                    pixel_data[i] = (n * color_modifier.0).round() as u8;
                    pixel_data[i + 1] = (n * color_modifier.1).round() as u8;
                    pixel_data[i + 2] = (n * color_modifier.2).round() as u8;
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

        // Render sprites
        {
            let player_position = *self
                .game
                .world
                .get_component::<PositionComponent>(self.game.player_entity)
                .unwrap();
            let mut render_objects = <(Read<SpriteComponent>, Read<PositionComponent>)>::query()
                .iter(&mut self.game.world)
                .filter_map(move |(sprite, position)| {
                    let adjusted_position = PositionComponent {
                        x: position.x - player_position.x + 7,
                        y: player_position.y - position.y + 7,
                    };
                    if (0..15).contains(&adjusted_position.x)
                        && (0..15).contains(&adjusted_position.y)
                    {
                        Some((sprite, adjusted_position))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            render_objects.sort_unstable_by_key(|(sprite, _)| sprite.in_foreground);

            for (sprite, position) in render_objects {
                let texture = texture_creator
                    .load_texture(format!("assets/{}.png", sprite.id))
                    .unwrap();
                let dest_rect =
                    Rect::new((position.x * 32) as i32, (position.y * 32) as i32, 32, 32);
                canvas.copy(&texture, None, dest_rect).unwrap();
            }
        }

        canvas.present();
    }
}

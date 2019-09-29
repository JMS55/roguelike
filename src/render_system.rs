use crate::components::{
    MessageColor, MessageLog, PlayerComponent, PositionComponent, SpriteComponent,
};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::Sdl;
use specs::{Join, ReadStorage, System, Write};
use std::time::Duration;

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
}

impl<'s> System<'s> for RenderSystem {
    type SystemData = (
        ReadStorage<'s, PlayerComponent>,
        ReadStorage<'s, PositionComponent>,
        ReadStorage<'s, SpriteComponent>,
        Write<'s, MessageLog>,
    );

    fn run(
        &mut self,
        (player_data, position_data, sprite_data, mut message_log): Self::SystemData,
    ) {
        self.canvas.clear();
        let texture_creator = self.canvas.texture_creator();

        let player_position = (&player_data, &position_data).join().next().unwrap().1;
        for (entity_position, entity_sprite) in (&position_data, &sprite_data).join() {
            let adjusted_entity_position_x = entity_position.x - player_position.x + 7;
            let adjusted_entity_position_y = player_position.y - entity_position.y + 7;
            if (0..15).contains(&adjusted_entity_position_x)
                && (0..15).contains(&adjusted_entity_position_y)
            {
                let texture = texture_creator
                    .load_texture(format!("assets/{}.png", entity_sprite.id))
                    .unwrap();
                let dest_rect = Rect::new(
                    adjusted_entity_position_x * 8 * 4,
                    adjusted_entity_position_y * 8 * 4,
                    8 * 4,
                    8 * 4,
                );
                self.canvas.copy(&texture, None, dest_rect).unwrap();
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
            if time_since_message_creation > Duration::from_secs(2) {
                alpha = (1.0
                    - 255.0 * time_since_message_creation.div_duration_f32(Duration::from_secs(5)))
                    as u8;
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
                self.canvas.fill_rect(dest_rect).unwrap();
                self.canvas.copy(&texture, None, dest_rect).unwrap();
            } else {
                break;
            };
        }

        self.canvas.present();
    }
}

impl MessageColor {
    pub fn sdl_color(&self, alpha: u8) -> Color {
        let (r, g, b) = match self {
            MessageColor::White => (255, 255, 255),
            MessageColor::Orange => (255, 96, 0),
            MessageColor::Red => (255, 0, 0),
        };
        Color::RGBA(r, g, b, alpha)
    }
}

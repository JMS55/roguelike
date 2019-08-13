use crate::components::{PlayerComponent, PositionComponent, SpriteComponent};
use sdl2::image::{InitFlag, LoadTexture, Sdl2ImageContext};
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use specs::{Join, ReadStorage, System};

pub struct RenderSystem {
    canvas: WindowCanvas,
    _sdl_image_context: Sdl2ImageContext,
}

impl RenderSystem {
    pub fn new(sdl_context: &Sdl) -> Self {
        let _sdl_image_context = sdl2::image::init(InitFlag::PNG).unwrap();
        let sdl_video_context = sdl_context.video().unwrap();
        let window = sdl_video_context
            .window("roguelike", 15 * 8 * 4, 15 * 8 * 4)
            .build()
            .unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();
        Self {
            canvas,
            _sdl_image_context,
        }
    }
}

impl<'s> System<'s> for RenderSystem {
    type SystemData = (
        ReadStorage<'s, PlayerComponent>,
        ReadStorage<'s, PositionComponent>,
        ReadStorage<'s, SpriteComponent>,
    );

    fn run(&mut self, (player_data, position_data, sprite_data): Self::SystemData) {
        let player_position = (&player_data, &position_data).join().next().unwrap().1;
        self.canvas.clear();
        for (entity_position, entity_sprite) in (&position_data, &sprite_data).join() {
            let adjusted_entity_position = PositionComponent {
                x: entity_position.x - player_position.x + 7,
                y: player_position.y - entity_position.y + 7,
            };
            if (0..15).contains(&adjusted_entity_position.x)
                && (0..15).contains(&adjusted_entity_position.y)
            {
                let texture_creator = self.canvas.texture_creator();
                let texture = texture_creator
                    .load_texture(format!("assets/{}.png", entity_sprite.id))
                    .unwrap();
                let dest_rect = Rect::new(
                    adjusted_entity_position.x * 8 * 4,
                    adjusted_entity_position.y * 8 * 4,
                    8 * 4,
                    8 * 4,
                );
                self.canvas.copy(&texture, None, dest_rect).unwrap();
            }
        }
        self.canvas.present();
    }
}

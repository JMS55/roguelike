mod components;
mod player_move_system;
mod render_system;

use components::*;
use player_move_system::{Direction, PlayerMoveSystem};
use render_system::RenderSystem;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use specs::{Builder, RunNow, World, WorldExt};
use std::time::{Duration, Instant};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut world = World::new();
    world.register::<PlayerComponent>();
    world.register::<PositionComponent>();
    world.register::<SpriteComponent>();
    let mut player_move_system = PlayerMoveSystem::new();
    let mut render_system = RenderSystem::new(&sdl_context);

    world
        .create_entity()
        .with(PlayerComponent {})
        .with(PositionComponent { x: 0, y: 0 })
        .with(SpriteComponent { id: "player" })
        .build();

    let mut time_accumulator = Duration::from_secs(0);
    let mut previous_time = Instant::now();
    'game_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'game_loop,
                Event::KeyDown { scancode, .. } => match scancode {
                    #[cfg(debug_assertions)]
                    Some(Scancode::Escape) => break 'game_loop,
                    Some(Scancode::W) => {
                        player_move_system.direction_to_move = Some(Direction::Up);
                    }
                    Some(Scancode::A) => {
                        player_move_system.direction_to_move = Some(Direction::Left);
                    }
                    Some(Scancode::S) => {
                        player_move_system.direction_to_move = Some(Direction::Down);
                    }
                    Some(Scancode::D) => {
                        player_move_system.direction_to_move = Some(Direction::Right);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let current_time = Instant::now();
        time_accumulator += current_time - previous_time;
        previous_time = current_time;
        while time_accumulator >= Duration::from_nanos(16700000) {
            player_move_system.run_now(&world);
            time_accumulator -= Duration::from_nanos(16700000);
        }
        render_system.run_now(&world);
    }
}

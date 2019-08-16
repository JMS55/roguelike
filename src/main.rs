mod attack_system;
mod components;
mod player_system;
mod render_system;

use attack_system::AttackSystem;
use components::*;
use player_system::{PlayerAction, PlayerSystem};
use render_system::RenderSystem;
use sdl2::event::Event;
use sdl2::keyboard::{Mod, Scancode};
use specs::{Builder, RunNow, World, WorldExt};
use std::time::{Duration, Instant};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut world = World::new();
    world.register::<PlayerComponent>();
    world.register::<PositionComponent>();
    world.register::<SpriteComponent>();
    world.register::<HealthComponent>();
    world.register::<QueuedAttack>();
    world.register::<QueuedMovement>();
    let mut player_system = PlayerSystem::new();
    let mut attack_system = AttackSystem::new();
    let mut render_system = RenderSystem::new(&sdl_context);

    world
        .create_entity()
        .with(PlayerComponent {})
        .with(PositionComponent {
            x: 0,
            y: 0,
            facing_direction: Direction::Right,
        })
        .with(SpriteComponent { id: "player" })
        .build();

    let mut time_accumulator = Duration::from_secs(0);
    let mut previous_time = Instant::now();
    'game_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'game_loop,
                Event::KeyDown {
                    scancode, keymod, ..
                } => match scancode {
                    #[cfg(debug_assertions)]
                    Some(Scancode::Escape) => break 'game_loop,
                    Some(Scancode::W) => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Up);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Up);
                        }
                    }
                    Some(Scancode::A) => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Left);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Left);
                        }
                    }
                    Some(Scancode::S) => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Down);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Down);
                        }
                    }
                    Some(Scancode::D) => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Right);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Right);
                        }
                    }
                    Some(Scancode::Q) => {
                        player_system.action = PlayerAction::Attack;
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
            player_system.run_now(&world);
            attack_system.run_now(&world);
            world.maintain();
            time_accumulator -= Duration::from_nanos(16700000);
        }
        render_system.run_now(&world);
    }
}

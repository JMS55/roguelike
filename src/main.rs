mod advance_floor_system;
mod ai_attack_player_system;
mod attack_system;
mod components;
mod drain_crystals_system;
mod generate_dungeon_system;
mod movement_system;
mod player_system;
mod render_system;

use advance_floor_system::AdvanceFloorSystem;
use ai_attack_player_system::AIAttackPlayerSystem;
use attack_system::AttackSystem;
use components::*;
use drain_crystals_system::DrainCrystalsSystem;
use generate_dungeon_system::GenerateDungeonSystem;
use movement_system::MovementSystem;
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
    world.insert(IsPlayerTurn(true));
    world.insert(ShouldAdvanceFloor(true));
    world.insert(MessageLog::new());
    world.register::<PlayerComponent>();
    world.register::<PositionComponent>();
    world.register::<SpriteComponent>();
    world.register::<HealthComponent>();
    world.register::<StaircaseComponent>();
    world.register::<AIAttackPlayerComponent>();
    world.register::<QueuedAttack>();
    world.register::<QueuedMovement>();
    let mut advance_floor_system = AdvanceFloorSystem::new();
    let mut generate_dungeon_system = GenerateDungeonSystem::new();
    let mut player_system = PlayerSystem::new();
    let mut ai_attack_player_system = AIAttackPlayerSystem::new();
    let mut attack_system = AttackSystem::new();
    let mut movement_system = MovementSystem::new();
    let mut drain_crystals_system = DrainCrystalsSystem::new();
    let mut render_system = RenderSystem::new(&sdl_context);
    world
        .create_entity()
        .with(PlayerComponent::new(10000))
        .with(PositionComponent {
            x: 0,
            y: 0,
            facing_direction: Direction::Right,
        })
        .with(HealthComponent {
            current_health: 10,
            max_health: 10,
        })
        .with(SpriteComponent { id: "red" })
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
                    Some(Scancode::W) if world.fetch::<IsPlayerTurn>().0 => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Up);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Up);
                        }
                    }
                    Some(Scancode::A) if world.fetch::<IsPlayerTurn>().0 => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Left);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Left);
                        }
                    }
                    Some(Scancode::S) if world.fetch::<IsPlayerTurn>().0 => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Down);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Down);
                        }
                    }
                    Some(Scancode::D) if world.fetch::<IsPlayerTurn>().0 => {
                        if keymod.contains(Mod::LSHIFTMOD) {
                            player_system.action = PlayerAction::TurnToFace(Direction::Right);
                        } else {
                            player_system.action = PlayerAction::Move(Direction::Right);
                        }
                    }
                    Some(Scancode::Q) if world.fetch::<IsPlayerTurn>().0 => {
                        player_system.action = PlayerAction::Interact;
                    }
                    Some(Scancode::E) if world.fetch::<IsPlayerTurn>().0 => {
                        player_system.action = PlayerAction::Pass;
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
            if world.fetch::<ShouldAdvanceFloor>().0 {
                advance_floor_system.run_now(&world);
                world.maintain();
                generate_dungeon_system.run_now(&world);
                world.maintain();
            } else {
                if world.fetch::<IsPlayerTurn>().0 {
                    player_system.run_now(&world);
                } else {
                    ai_attack_player_system.run_now(&world);
                    world.insert(IsPlayerTurn(true));
                }
                attack_system.run_now(&world);
                world.maintain();
                movement_system.run_now(&world);
                world.maintain();
                if !world.fetch::<IsPlayerTurn>().0 {
                    drain_crystals_system.run_now(&world);
                }
            }
            time_accumulator -= Duration::from_nanos(16700000);
        }
        render_system.run_now(&world);
    }
}

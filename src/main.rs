mod attack;
mod data;
mod drain_crystals;
mod enemy_controller;
mod entities;
mod generate_dungeon;
mod movement;
mod player_controller;
mod render;

use data::*;
use drain_crystals::drain_crystals_system;
use enemy_controller::enemy_controller_system;
use generate_dungeon::GenerateDungeonSystem;
use player_controller::{PlayerActed, PlayerAction, PlayerControllerSystem};
use render::RenderSystem;
use sdl2::event::Event;
use sdl2::keyboard::{Mod, Scancode};
use specs::{World, WorldExt};
use std::time::{Duration, Instant};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut world = World::new();
    world.register::<Name>();
    world.register::<Position>();
    world.register::<Sprite>();
    world.register::<Attackable>();
    world.register::<HealAttackerOnDeath>();
    world.register::<AI>();
    world.register::<RNG>();
    world.register::<Intangible>();
    world.register::<Player>();
    world.register::<Staircase>();
    world.register::<Spawner>();
    world.insert(GameState::NewGame);
    world.insert(MessageLog::new());
    let mut player_controller_system = PlayerControllerSystem::new();
    let mut generate_dungeon_system = GenerateDungeonSystem::new();
    let mut render_system = RenderSystem::new(&sdl_context);

    let mut time_accumulator = Duration::from_secs(0);
    let mut previous_time = Instant::now();
    'game_loop: loop {
        let game_state = *world.fetch::<GameState>();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'game_loop,
                Event::KeyDown {
                    scancode, keymod, ..
                } => match scancode {
                    #[cfg(debug_assertions)]
                    Some(Scancode::Escape) => break 'game_loop,
                    Some(Scancode::W) => {
                        if game_state == GameState::PlayerTurn {
                            if keymod.contains(Mod::LSHIFTMOD) {
                                player_controller_system.action = PlayerAction::Turn(Direction::Up);
                            } else {
                                player_controller_system.action = PlayerAction::Move(Direction::Up);
                            }
                        }
                    }
                    Some(Scancode::A) => {
                        if game_state == GameState::PlayerTurn {
                            if keymod.contains(Mod::LSHIFTMOD) {
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Left);
                            } else {
                                player_controller_system.action =
                                    PlayerAction::Move(Direction::Left);
                            }
                        }
                    }
                    Some(Scancode::S) => {
                        if game_state == GameState::PlayerTurn {
                            if keymod.contains(Mod::LSHIFTMOD) {
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Down);
                            } else {
                                player_controller_system.action =
                                    PlayerAction::Move(Direction::Down);
                            }
                        }
                    }
                    Some(Scancode::D) => {
                        if game_state == GameState::PlayerTurn {
                            if keymod.contains(Mod::LSHIFTMOD) {
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Right);
                            } else {
                                player_controller_system.action =
                                    PlayerAction::Move(Direction::Right);
                            }
                        }
                    }
                    Some(Scancode::Q) => {
                        if game_state == GameState::PlayerTurn {
                            player_controller_system.action = PlayerAction::Interact;
                        }
                    }
                    Some(Scancode::E) => {
                        if game_state == GameState::PlayerTurn {
                            player_controller_system.action = PlayerAction::Pass;
                        }
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
            let game_state = *world.fetch::<GameState>();
            match game_state {
                GameState::NewGame => {
                    world.insert(GameState::PlayerTurn);
                    world.fetch_mut::<MessageLog>().empty();
                    entities::create_player(&mut world);
                    generate_dungeon_system.run(&mut world);
                }
                GameState::PlayerTurn => {
                    if player_controller_system.run(&mut generate_dungeon_system, &mut world)
                        == PlayerActed(true)
                    {
                        drain_crystals_system(&mut world);
                    }
                }
                GameState::EnemyTurn => {
                    enemy_controller_system(&mut world);
                    world.insert(GameState::PlayerTurn);
                }
            }
            time_accumulator -= Duration::from_nanos(16700000);
        }
        render_system.run(&mut world);
    }
}

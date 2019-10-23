mod attack;
mod data;
mod drain_crystals;
mod enemy_controller;
mod entities;
mod generate_dungeon;
mod items;
mod movement;
mod player_controller;
mod render;
mod spawn;

use data::*;
use drain_crystals::drain_crystals_system;
use enemy_controller::enemy_controller_system;
use generate_dungeon::GenerateDungeonSystem;
use player_controller::{PlayerActed, PlayerAction, PlayerControllerSystem};
use render::RenderSystem;
use spawn::tick_spawners;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
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
    world.register::<AI>();
    world.register::<AICounter>();
    world.register::<Intangible>();
    world.register::<Player>();
    world.register::<Staircase>();
    world.register::<Spawner>();
    world.register::<Item>();
    world.insert(GameState::NewGame);
    world.insert(MessageLog::new());
    world.insert(RNG::new());
    let mut player_controller_system = PlayerControllerSystem::new();
    let mut generate_dungeon_system = GenerateDungeonSystem::new();
    let mut render_system = RenderSystem::new(&sdl_context);

    let mut frames_since_last_input = 5;
    let mut time_accumulator = Duration::from_secs(0);
    let mut previous_time = Instant::now();
    'game_loop: loop {
        let game_state = *world.fetch::<GameState>();

        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'game_loop;
            }
        }
        let keyboard = event_pump.keyboard_state();
        if keyboard.is_scancode_pressed(Scancode::Escape) {
            break 'game_loop;
        }
        if game_state == GameState::PlayerTurn && frames_since_last_input >= 5 {
            let mut keystate = (0, 0, true);
            if keyboard.is_scancode_pressed(Scancode::LShift) {
                frames_since_last_input = 0;
                keystate.2 = false;
            }
            if keyboard.is_scancode_pressed(Scancode::W) {
                frames_since_last_input = 0;
                keystate.1 = 1;
            }
            if keyboard.is_scancode_pressed(Scancode::A) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::Turn(Direction::Left);
                keystate.0 = -1;
            }
            if keyboard.is_scancode_pressed(Scancode::S) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::Turn(Direction::Down);
                keystate.1 = -1;
            }
            if keyboard.is_scancode_pressed(Scancode::D) {
                frames_since_last_input = 0;
                keystate.0 = 1;
            }
            player_controller_system.action = match keystate {
                (0, 0, _) => PlayerAction::None,
                (1, 0, true) => PlayerAction::Move(Direction::Right),
                (-1, 0, true) => PlayerAction::Move(Direction::Left),
                (0, 1, true) => PlayerAction::Move(Direction::Up),
                (0, -1, true) => PlayerAction::Move(Direction::Down),
                (1, 1, true) => PlayerAction::Move(Direction::UpRight),
                (1, -1, true) => PlayerAction::Move(Direction::DownRight),
                (-1, 1, true) => PlayerAction::Move(Direction::UpLeft),
                (-1, -1, true) => PlayerAction::Move(Direction::DownLeft),
                (1, 0, false) => PlayerAction::Turn(Direction::Right),
                (-1, 0, false) => PlayerAction::Turn(Direction::Left),
                (0, 1, false) => PlayerAction::Turn(Direction::Up),
                (0, -1, false) => PlayerAction::Turn(Direction::Down),
                (1, 1, false) => PlayerAction::Turn(Direction::UpRight),
                (1, -1, false) => PlayerAction::Turn(Direction::DownRight),
                (-1, 1, false) => PlayerAction::Turn(Direction::UpLeft),
                (-1, -1, false) => PlayerAction::Turn(Direction::DownLeft),
                _ => unreachable!(),
            };
            if keyboard.is_scancode_pressed(Scancode::E) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::Pass;
            }
            if keyboard.is_scancode_pressed(Scancode::Q) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::Interact;
            }
            if keyboard.is_scancode_pressed(Scancode::Num1) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::UseItem(ItemSlot::One);
            }
            if keyboard.is_scancode_pressed(Scancode::Num2) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::UseItem(ItemSlot::Two);
            }
            if keyboard.is_scancode_pressed(Scancode::Num3) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::UseItem(ItemSlot::Three);
            }
            if keyboard.is_scancode_pressed(Scancode::Num4) {
                frames_since_last_input = 0;
                player_controller_system.action = PlayerAction::UseItem(ItemSlot::Four);
            }
        } else {
            frames_since_last_input += 1;
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
                    world.insert(RNG::new());
                    entities::create_player(&mut world);
                    generate_dungeon_system.run(&mut world);
                    tick_spawners(&mut world);
                }
                GameState::PlayerTurn => {
                    if player_controller_system.run(&mut generate_dungeon_system, &mut world)
                        == PlayerActed(true)
                    {
                        drain_crystals_system(&mut world);
                    }
                }
                GameState::EnemyTurn => {
                    tick_spawners(&mut world);
                    enemy_controller_system(&mut world);
                    world.insert(GameState::PlayerTurn);
                }
            }
            time_accumulator -= Duration::from_nanos(16700000);
        }
        render_system.run(&mut world);
    }
}

mod attack;
mod data;
mod drain_crystals;
mod end_of_turn;
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
use end_of_turn::end_of_turn;
use enemy_controller::enemy_controller_system;
use generate_dungeon::GenerateDungeonSystem;
use player_controller::{PlayerActed, PlayerAction, PlayerControllerSystem};
use render::RenderSystem;
use spawn::tick_spawners;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use specs::{Join, World, WorldExt};
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
    world.register::<Counter>();
    world.register::<Intangible>();
    world.register::<Player>();
    world.register::<Staircase>();
    world.register::<Spawner>();
    world.register::<Item>();
    world.insert(GameState::NewGame);
    world.insert(MessageLog::new());
    let mut rng = RNG::new();
    world.insert(ScrollInfo::new(&mut rng));
    world.insert(rng);
    let mut player_controller_system = PlayerControllerSystem::new();
    let mut generate_dungeon_system = GenerateDungeonSystem::new();
    let mut render_system = RenderSystem::new(&sdl_context);

    let mut last_input_time = Instant::now();
    let mut time_accumulator = Duration::from_secs(0);
    let mut previous_time = Instant::now();
    'game_loop: loop {
        {
            for event in event_pump.poll_iter() {
                if let Event::Quit { .. } = event {
                    break 'game_loop;
                }
            }

            let game_state = { *world.fetch::<GameState>() };

            let keyboard = event_pump.keyboard_state();
            if keyboard.is_scancode_pressed(Scancode::Escape) {
                break 'game_loop;
            }
            if game_state == GameState::PlayerTurn
                && last_input_time.elapsed() >= Duration::from_millis(150)
            {
                let mut keystate = (0, 0, true);
                if keyboard.is_scancode_pressed(Scancode::LShift) {
                    last_input_time = Instant::now();
                    keystate.2 = false;
                }
                if keyboard.is_scancode_pressed(Scancode::W)
                    || keyboard.is_scancode_pressed(Scancode::Up)
                {
                    last_input_time = Instant::now();
                    keystate.1 = 1;
                }
                if keyboard.is_scancode_pressed(Scancode::A)
                    || keyboard.is_scancode_pressed(Scancode::Left)
                {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::Turn(Direction::Left);
                    keystate.0 = -1;
                }
                if keyboard.is_scancode_pressed(Scancode::S)
                    || keyboard.is_scancode_pressed(Scancode::Down)
                {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::Turn(Direction::Down);
                    keystate.1 = -1;
                }
                if keyboard.is_scancode_pressed(Scancode::D)
                    || keyboard.is_scancode_pressed(Scancode::Right)
                {
                    last_input_time = Instant::now();
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
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::Pass;
                }
                if keyboard.is_scancode_pressed(Scancode::Q) {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::Interact;
                }
                if keyboard.is_scancode_pressed(Scancode::Num1) {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::UseItem(ItemSlot::One);
                }
                if keyboard.is_scancode_pressed(Scancode::Num2) {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::UseItem(ItemSlot::Two);
                }
                if keyboard.is_scancode_pressed(Scancode::Num3) {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::UseItem(ItemSlot::Three);
                }
                if keyboard.is_scancode_pressed(Scancode::Num4) {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::UseItem(ItemSlot::Four);
                }
                if keyboard.is_scancode_pressed(Scancode::Space) {
                    last_input_time = Instant::now();
                    player_controller_system.action = PlayerAction::None;
                    *world.fetch_mut::<GameState>() = GameState::BagUI(BagUIState::Overview(0, 0));
                }
            }

            if let GameState::BagUI(bag_ui_state) = game_state {
                if last_input_time.elapsed() >= Duration::from_millis(150) {
                    let mut new_bag_ui_state;
                    match bag_ui_state {
                        BagUIState::Overview(ref selected_item_x, ref selected_item_y) => {
                            let mut keystate = (0, 0);
                            if keyboard.is_scancode_pressed(Scancode::W)
                                || keyboard.is_scancode_pressed(Scancode::Up)
                            {
                                last_input_time = Instant::now();
                                keystate.1 = -1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::A)
                                || keyboard.is_scancode_pressed(Scancode::Left)
                            {
                                last_input_time = Instant::now();
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Left);
                                keystate.0 = -1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::S)
                                || keyboard.is_scancode_pressed(Scancode::Down)
                            {
                                last_input_time = Instant::now();
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Down);
                                keystate.1 = 1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::D)
                                || keyboard.is_scancode_pressed(Scancode::Right)
                            {
                                last_input_time = Instant::now();
                                keystate.0 = 1;
                            }
                            new_bag_ui_state = BagUIState::Overview(
                                (*selected_item_x + keystate.0).rem_euclid(4),
                                (*selected_item_y + keystate.1).rem_euclid(4),
                            );
                            if keyboard.is_scancode_pressed(Scancode::Return) {
                                last_input_time = Instant::now();
                                let item_entity = {
                                    let mut player_data = world.write_storage::<Player>();
                                    let player = (&mut player_data).join().next().unwrap();
                                    let index = (*selected_item_x + *selected_item_y * 4) as usize;
                                    player.inventory[index]
                                };
                                if item_entity.is_some() {
                                    new_bag_ui_state =
                                        BagUIState::ItemMenu(*selected_item_x, *selected_item_y, 0);
                                }
                            }
                        }
                        BagUIState::ItemMenu(
                            ref selected_item_x,
                            ref selected_item_y,
                            ref selected_menu_option,
                        ) => {
                            let mut keystate = 0;
                            if keyboard.is_scancode_pressed(Scancode::W)
                                || keyboard.is_scancode_pressed(Scancode::Up)
                            {
                                last_input_time = Instant::now();
                                keystate = -1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::A)
                                || keyboard.is_scancode_pressed(Scancode::Left)
                            {
                                last_input_time = Instant::now();
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Left);
                                keystate = -1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::S)
                                || keyboard.is_scancode_pressed(Scancode::Down)
                            {
                                last_input_time = Instant::now();
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Down);
                                keystate = 1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::D)
                                || keyboard.is_scancode_pressed(Scancode::Right)
                            {
                                last_input_time = Instant::now();
                                keystate = 1;
                            }
                            new_bag_ui_state = BagUIState::ItemMenu(
                                *selected_item_x,
                                *selected_item_y,
                                (*selected_menu_option + keystate).rem_euclid(3),
                            );
                            if keyboard.is_scancode_pressed(Scancode::Return) {
                                last_input_time = Instant::now();
                                new_bag_ui_state = match selected_menu_option {
                                    0 => BagUIState::Overview(*selected_item_x, *selected_item_y),
                                    1 => BagUIState::MoveItem(
                                        *selected_item_x,
                                        *selected_item_y,
                                        if *selected_item_x == 0 && *selected_item_y == 0 {
                                            1
                                        } else {
                                            0
                                        },
                                        0,
                                    ),
                                    2 => {
                                        let item_entity = {
                                            let mut player_data = world.write_storage::<Player>();
                                            let player = (&mut player_data).join().next().unwrap();
                                            let index =
                                                (*selected_item_x + *selected_item_y * 4) as usize;
                                            player.inventory[index].take()
                                        };
                                        if let Some(item_entity) = item_entity {
                                            world.delete_entity(item_entity).unwrap();
                                        }
                                        BagUIState::Overview(*selected_item_x, *selected_item_y)
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                        BagUIState::MoveItem(
                            ref selected_item1_x,
                            ref selected_item1_y,
                            ref selected_item2_x,
                            ref selected_item2_y,
                        ) => {
                            let mut keystate = (0, 0);
                            if keyboard.is_scancode_pressed(Scancode::W)
                                || keyboard.is_scancode_pressed(Scancode::Up)
                            {
                                last_input_time = Instant::now();
                                keystate.1 = -1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::A)
                                || keyboard.is_scancode_pressed(Scancode::Left)
                            {
                                last_input_time = Instant::now();
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Left);
                                keystate.0 = -1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::S)
                                || keyboard.is_scancode_pressed(Scancode::Down)
                            {
                                last_input_time = Instant::now();
                                player_controller_system.action =
                                    PlayerAction::Turn(Direction::Down);
                                keystate.1 = 1;
                            }
                            if keyboard.is_scancode_pressed(Scancode::D)
                                || keyboard.is_scancode_pressed(Scancode::Right)
                            {
                                last_input_time = Instant::now();
                                keystate.0 = 1;
                            }
                            let new_2x = (*selected_item2_x + keystate.0).rem_euclid(4);
                            let new_2y = (*selected_item2_y + keystate.1).rem_euclid(4);
                            new_bag_ui_state = BagUIState::MoveItem(
                                *selected_item1_x,
                                *selected_item1_y,
                                new_2x,
                                new_2y,
                            );
                            if keyboard.is_scancode_pressed(Scancode::Return) {
                                last_input_time = Instant::now();
                                let mut player_data = world.write_storage::<Player>();
                                let player = (&mut player_data).join().next().unwrap();
                                let index_1 = (selected_item1_x + selected_item1_y * 4) as usize;
                                let index_2 = (new_2x + new_2y * 4) as usize;
                                player.inventory.swap(index_1, index_2);
                                if (index_1 <= 3 && index_2 > 3) || (index_2 <= 3 && index_1 > 3) {
                                    player_controller_system.action = PlayerAction::Pass;
                                    *world.fetch_mut::<GameState>() = GameState::PlayerTurn;
                                }
                                new_bag_ui_state =
                                    BagUIState::Overview(*selected_item1_x, *selected_item1_y);
                            }
                        }
                    }
                    {
                        let mut game_state = world.fetch_mut::<GameState>();
                        match *game_state {
                            GameState::BagUI(_) => *game_state = GameState::BagUI(new_bag_ui_state),
                            _ => {}
                        }
                    }

                    if keyboard.is_scancode_pressed(Scancode::Space) {
                        last_input_time = Instant::now();
                        *world.fetch_mut::<GameState>() = GameState::PlayerTurn;
                    }
                }
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
                        end_of_turn(&mut world);
                    }
                }
                GameState::EnemyTurn => {
                    tick_spawners(&mut world);
                    enemy_controller_system(&mut world);
                    end_of_turn(&mut world);
                    world.insert(GameState::PlayerTurn);
                }
                _ => {}
            }
            time_accumulator -= Duration::from_nanos(16700000);
        }
        render_system.run(&mut world);
    }
}

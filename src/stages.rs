use crate::components::*;
use crate::game::Game;
use crate::generate_dungeon::generate_dungeon;
use crate::movement::{try_move, turn_player_towards, Direction};
use legion::filter::filter_fns;
use legion::query::{IntoQuery, Read};
use sdl2::keyboard::{KeyboardState, Scancode};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::time::{Duration, Instant};

pub trait Stage {
    fn input(&mut self, keyboard: &KeyboardState);

    fn update(self: Box<Self>) -> Box<dyn Stage>;

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
    );
}

pub struct NewGameStage {}

impl Stage for NewGameStage {
    fn input(&mut self, _: &KeyboardState) {}

    fn update(self: Box<Self>) -> Box<dyn Stage> {
        Box::new(PlayerTurnStage {
            game: Game::new(),
            action: PlayerAction::None,
            last_input_time: Instant::now(),
        })
    }

    fn render(
        &mut self,
        _: &mut WindowCanvas,
        _: &mut TextureCreator<WindowContext>,
        _: &mut Font,
    ) {
    }
}

pub struct PlayerTurnStage {
    game: Game,
    action: PlayerAction,
    last_input_time: Instant,
}

impl Stage for PlayerTurnStage {
    fn input(&mut self, keyboard: &KeyboardState) {
        if self.last_input_time.elapsed() >= Duration::from_millis(150) {
            let mut keystate = (0, 0, true);
            if keyboard.is_scancode_pressed(Scancode::LShift) {
                self.last_input_time = Instant::now();
                keystate.2 = false;
            }
            if keyboard.is_scancode_pressed(Scancode::W)
                || keyboard.is_scancode_pressed(Scancode::Up)
            {
                self.last_input_time = Instant::now();
                keystate.1 = 1;
            }
            if keyboard.is_scancode_pressed(Scancode::A)
                || keyboard.is_scancode_pressed(Scancode::Left)
            {
                self.last_input_time = Instant::now();
                keystate.0 = -1;
            }
            if keyboard.is_scancode_pressed(Scancode::S)
                || keyboard.is_scancode_pressed(Scancode::Down)
            {
                self.last_input_time = Instant::now();
                keystate.1 = -1;
            }
            if keyboard.is_scancode_pressed(Scancode::D)
                || keyboard.is_scancode_pressed(Scancode::Right)
            {
                self.last_input_time = Instant::now();
                keystate.0 = 1;
            }
            self.action = match keystate {
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
                self.last_input_time = Instant::now();
                self.action = PlayerAction::Pass;
            }
            if keyboard.is_scancode_pressed(Scancode::Q) {
                self.last_input_time = Instant::now();
                self.action = PlayerAction::Interact;
            }
            if keyboard.is_scancode_pressed(Scancode::Num1) {
                self.last_input_time = Instant::now();
                self.action = PlayerAction::UseItem(ItemSlot::One);
            }
            if keyboard.is_scancode_pressed(Scancode::Num2) {
                self.last_input_time = Instant::now();
                self.action = PlayerAction::UseItem(ItemSlot::Two);
            }
            if keyboard.is_scancode_pressed(Scancode::Num3) {
                self.last_input_time = Instant::now();
                self.action = PlayerAction::UseItem(ItemSlot::Three);
            }
            if keyboard.is_scancode_pressed(Scancode::Num4) {
                self.last_input_time = Instant::now();
                self.action = PlayerAction::UseItem(ItemSlot::Four);
            }
        }
    }

    fn update(mut self: Box<Self>) -> Box<dyn Stage> {
        match self.action {
            PlayerAction::None => {}
            PlayerAction::Pass => {
                return Box::new(AITurnStage { game: self.game });
            }
            PlayerAction::Interact => {
                let player = *self
                    .game
                    .world
                    .get_component::<PlayerComponent>(self.game.player_entity)
                    .unwrap();
                let player_position = *self
                    .game
                    .world
                    .get_component::<PositionComponent>(self.game.player_entity)
                    .unwrap();
                let offset = match player.facing_direction {
                    Direction::Up => PositionComponent { x: 0, y: 1 },
                    Direction::Down => PositionComponent { x: 0, y: -1 },
                    Direction::Left => PositionComponent { x: -1, y: 0 },
                    Direction::Right => PositionComponent { x: 1, y: 0 },
                    Direction::UpLeft => PositionComponent { x: -1, y: 1 },
                    Direction::UpRight => PositionComponent { x: 1, y: 1 },
                    Direction::DownLeft => PositionComponent { x: -1, y: -1 },
                    Direction::DownRight => PositionComponent { x: 1, y: -1 },
                };
                let interacting_with_position = PositionComponent {
                    x: player_position.x + offset.x,
                    y: player_position.y + offset.y,
                };

                // If player facing and next to a staircase
                if Read::<PositionComponent>::query()
                    .filter(filter_fns::component::<StaircaseComponent>())
                    .iter_immutable(&self.game.world)
                    .any(|position| *position == interacting_with_position)
                {
                    // Heal the player by 20% of their max health
                    {
                        let mut player_stats = self
                            .game
                            .world
                            .get_component_mut::<StatsComponent>(self.game.player_entity)
                            .unwrap();
                        player_stats.current_health = player_stats
                            .max_health
                            .min((player_stats.max_health as f64 * 0.2).round() as u16);
                    }

                    // Reset player position
                    *self
                        .game
                        .world
                        .get_component_mut::<PositionComponent>(self.game.player_entity)
                        .unwrap() = PositionComponent { x: 0, y: 0 };

                    // Delete all entities besides the player
                    // TODO

                    // Generate a new floor
                    generate_dungeon(&mut self.game);

                    return Box::new(PlayerTurnStage {
                        game: self.game,
                        action: PlayerAction::None,
                        last_input_time: Instant::now(),
                    });
                }
            }
            PlayerAction::Turn(direction) => {
                turn_player_towards(direction, &mut self.game);
            }
            PlayerAction::Move(direction) => {
                if try_move(self.game.player_entity, direction, &mut self.game).is_ok() {
                    return Box::new(AITurnStage { game: self.game });
                }
            }
            PlayerAction::UseItem(item_slot) => {}
        }

        self.action = PlayerAction::None;
        self
    }

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
    ) {
        self.game.render(canvas, texture_creator, font);
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum PlayerAction {
    None,
    Pass,
    Interact,
    Turn(Direction),
    Move(Direction),
    UseItem(ItemSlot),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ItemSlot {
    One,
    Two,
    Three,
    Four,
}

pub struct AITurnStage {
    game: Game,
}

impl Stage for AITurnStage {
    fn input(&mut self, _: &KeyboardState) {}

    fn update(self: Box<Self>) -> Box<dyn Stage> {
        Box::new(PlayerTurnStage {
            game: self.game,
            action: PlayerAction::None,
            last_input_time: Instant::now(),
        })
    }

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
    ) {
        self.game.render(canvas, texture_creator, font);
    }
}

use crate::game::Game;
use crate::movement::{try_move, turn, Direction};
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
        delta_time: f64,
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
        let mut should_end_turn = false;
        match self.action {
            PlayerAction::None => {}
            PlayerAction::Pass => {
                should_end_turn = true;
            }
            PlayerAction::Interact => {}
            PlayerAction::Turn(direction) => {
                turn(direction, &mut self.game);
                should_end_turn = true;
            }
            PlayerAction::Move(direction) => {
                let move_result = try_move(self.game.player_entity, direction, &mut self.game);
                should_end_turn = move_result.is_ok();
            }
            PlayerAction::UseItem(item_slot) => {}
        }

        self.action = PlayerAction::None;
        if should_end_turn {
            Box::new(AITurnStage { game: self.game })
        } else {
            self
        }
    }

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
        delta_time: f64,
    ) {
        self.game.render(canvas, texture_creator, font, delta_time);
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
        delta_time: f64,
    ) {
        self.game.render(canvas, texture_creator, font, delta_time);
    }
}

use crate::combat::heal;
use crate::components::*;
use crate::game::Game;
use crate::generate_dungeon::generate_dungeon;
use crate::movement::{try_move, turn_player_towards, Direction};
use crate::spawn_enemies::spawn_enemies;
use hecs::Entity;
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

impl PlayerTurnStage {
    fn end_of_turn(&mut self) {
        let mut should_heal_player = false;
        {
            let mut player = self
                .game
                .world
                .get_mut::<PlayerComponent>(self.game.player_entity)
                .unwrap();
            player.turns_before_passive_healing -= 1;
            if player.turns_before_passive_healing == 0 {
                player.turns_before_passive_healing = 10;
                should_heal_player = true;
            }
        }
        if should_heal_player {
            heal(self.game.player_entity, 2, &mut self.game);
        }
    }
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
                self.end_of_turn();
                return Box::new(AITurnStage { game: self.game });
            }
            PlayerAction::Interact => {
                let player = *self
                    .game
                    .world
                    .get::<PlayerComponent>(self.game.player_entity)
                    .unwrap();
                let player_position = *self
                    .game
                    .world
                    .get::<PositionComponent>(self.game.player_entity)
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
                if self
                    .game
                    .world
                    .query::<&PositionComponent>()
                    .with::<StaircaseComponent>()
                    .iter()
                    .any(|(_, position)| *position == interacting_with_position)
                {
                    // Heal the player by 20% of their max health
                    let player_max_health = self
                        .game
                        .world
                        .get::<StatsComponent>(self.game.player_entity)
                        .unwrap()
                        .max_health;
                    heal(
                        self.game.player_entity,
                        (player_max_health as f64 * 0.2).round() as u16,
                        &mut self.game,
                    );

                    // Reset the player's position
                    *self
                        .game
                        .world
                        .get_mut::<PositionComponent>(self.game.player_entity)
                        .unwrap() = PositionComponent { x: 0, y: 0 };

                    // Delete all entities besides the player
                    let entities_to_delete = self
                        .game
                        .world
                        .iter()
                        .filter_map(|(entity, _)| {
                            if entity != self.game.player_entity {
                                Some(entity)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<Entity>>();
                    for entity in entities_to_delete {
                        self.game.world.despawn(entity).unwrap();
                    }

                    // Generate a new floor
                    generate_dungeon(&mut self.game);
                    spawn_enemies(&mut self.game);

                    self.end_of_turn();
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
                    self.end_of_turn();
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

    fn update(mut self: Box<Self>) -> Box<dyn Stage> {
        let ai_entities_to_run = self
            .game
            .world
            .query::<()>()
            .with::<AIComponent>()
            .iter()
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>();
        for ai_entity in ai_entities_to_run {
            // Duplicate the AI in case the entity dies during run()
            let ai = self
                .game
                .world
                .get_mut::<AIComponent>(ai_entity)
                .map(|ai_component| ai_component.ai.clone());
            if let Ok(mut ai) = ai {
                // Run the enitiy's AI. This mutates the copy we made.
                ai.run(ai_entity, &mut self.game);
                // Overwrite the old AI with the copy we made if it still exists and still has an AI
                if self.game.world.get::<AIComponent>(ai_entity).is_ok() {
                    self.game
                        .world
                        .insert_one(ai_entity, AIComponent { ai })
                        .unwrap();
                }
            }
        }

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

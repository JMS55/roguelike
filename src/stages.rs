use crate::components::*;
use crate::game::Game;
use crate::generate_dungeon::generate_dungeon;
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
            if keystate.0 == 0 && keystate.1 == 0 {
                self.action = PlayerAction::None;
            } else if keystate.2 {
                self.action = PlayerAction::Move(PositionComponent {
                    x: keystate.0,
                    y: keystate.1,
                });
            } else {
                self.action = PlayerAction::Turn(PositionComponent {
                    x: keystate.0,
                    y: keystate.1,
                });
            }
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
                if self.end_of_turn() {
                    return Box::new(GameOverStage { game: self.game });
                }
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
                let interacting_with_position = player_position + player.facing_direction;

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
                    {
                        let mut player_stats = self
                            .game
                            .world
                            .get_mut::<StatsComponent>(self.game.player_entity)
                            .unwrap();
                        player_stats.current_health = player_stats.max_health.min(
                            player_stats.current_health
                                + (player_stats.max_health as f64 * 0.2).round() as u32,
                        );
                    }

                    // Remove all debuffs
                    let _ = self
                        .game
                        .world
                        .remove_one::<BurnComponent>(self.game.player_entity);

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

                    if self.end_of_turn() {
                        return Box::new(GameOverStage { game: self.game });
                    }
                    return Box::new(PlayerTurnStage {
                        game: self.game,
                        action: PlayerAction::None,
                        last_input_time: Instant::now(),
                    });
                }
            }
            PlayerAction::Turn(direction) => {
                self.game
                    .world
                    .get_mut::<PlayerComponent>(self.game.player_entity)
                    .unwrap()
                    .facing_direction = direction;
            }
            PlayerAction::Move(direction) => {
                self.game
                    .world
                    .get_mut::<PlayerComponent>(self.game.player_entity)
                    .unwrap()
                    .facing_direction = direction;

                let player_position = *self
                    .game
                    .world
                    .get::<PositionComponent>(self.game.player_entity)
                    .unwrap();
                let attempted_move_position = player_position + direction;
                if self
                    .game
                    .world
                    .query::<&PositionComponent>()
                    .iter()
                    .all(|(_, position)| *position != attempted_move_position)
                {
                    *self
                        .game
                        .world
                        .get_mut::<PositionComponent>(self.game.player_entity)
                        .unwrap() = attempted_move_position;

                    if self.end_of_turn() {
                        return Box::new(GameOverStage { game: self.game });
                    }
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

impl PlayerTurnStage {
    fn end_of_turn(&mut self) -> bool {
        let mut should_end_turn = false;

        // Heal player by 2 health every 10 turns
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
            let mut player_stats = self
                .game
                .world
                .get_mut::<StatsComponent>(self.game.player_entity)
                .unwrap();
            player_stats.current_health =
                player_stats.max_health.min(player_stats.current_health + 2);
        }

        // Apply burn damage
        if self
            .game
            .world
            .get::<BurnComponent>(self.game.player_entity)
            .is_ok()
        {
            {
                let mut player_stats = self
                    .game
                    .world
                    .get_mut::<StatsComponent>(self.game.player_entity)
                    .unwrap();
                let mut burn = self
                    .game
                    .world
                    .get_mut::<BurnComponent>(self.game.player_entity)
                    .unwrap();

                player_stats.current_health = player_stats
                    .current_health
                    .saturating_sub(burn.damage_per_turn);
                burn.turns_left -= 1;
            }

            let burn = *self
                .game
                .world
                .get::<BurnComponent>(self.game.player_entity)
                .unwrap();
            if burn.turns_left == 0 {
                self.game
                    .world
                    .remove_one::<BurnComponent>(self.game.player_entity)
                    .unwrap();
            }

            let player_stats = *self
                .game
                .world
                .get::<StatsComponent>(self.game.player_entity)
                .unwrap();
            if player_stats.current_health == 0 {
                should_end_turn = true;
            }
        }

        should_end_turn
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum PlayerAction {
    None,
    Pass,
    Interact,
    Turn(PositionComponent),
    Move(PositionComponent),
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
        let mut ai_entities_to_run = self
            .game
            .world
            .query::<()>()
            .with::<AIComponent>()
            .iter()
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>();
        // Sort AI entities by distance to the player
        let player_position = *self
            .game
            .world
            .get::<PositionComponent>(self.game.player_entity)
            .unwrap();
        ai_entities_to_run.sort_by_key(|ai_entity| {
            if let Ok(ai_entity_position) = self.game.world.get::<PositionComponent>(*ai_entity) {
                (ai_entity_position.x - player_position.x)
                    .abs()
                    .max((ai_entity_position.y - player_position.y).abs()) as u32
            } else {
                u32::max_value()
            }
        });

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

                // If player is dead, game over
                let player_stats = *self
                    .game
                    .world
                    .get::<StatsComponent>(self.game.player_entity)
                    .unwrap();
                if player_stats.current_health == 0 {
                    return Box::new(GameOverStage { game: self.game });
                }
            }
        }

        // Apply burn damage
        let burn_entities = self
            .game
            .world
            .query::<()>()
            .with::<BurnComponent>()
            .without::<PlayerComponent>()
            .iter()
            .map(|(entity, _)| entity)
            .collect::<Vec<Entity>>();
        for entity in &burn_entities {
            let mut stats = self.game.world.get_mut::<StatsComponent>(*entity).unwrap();
            let mut burn = self.game.world.get_mut::<BurnComponent>(*entity).unwrap();
            stats.current_health = stats.current_health.saturating_sub(burn.damage_per_turn);
            burn.turns_left -= 1;
        }
        for entity in burn_entities {
            let stats = *self.game.world.get::<StatsComponent>(entity).unwrap();
            let burn = *self.game.world.get::<BurnComponent>(entity).unwrap();
            if burn.turns_left == 0 {
                self.game.world.remove_one::<BurnComponent>(entity).unwrap();
            }
            if stats.current_health == 0 {
                self.game.world.despawn(entity).unwrap();
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

pub struct GameOverStage {
    game: Game,
}

impl Stage for GameOverStage {
    fn input(&mut self, _: &KeyboardState) {}

    fn update(self: Box<Self>) -> Box<dyn Stage> {
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

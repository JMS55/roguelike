use crate::components::{Direction, PlayerComponent, PositionComponent, QueuedAttack};
use specs::{Entities, Join, System, WriteStorage};
use std::collections::HashSet;

pub struct PlayerSystem {
    pub action: PlayerAction,
}

impl PlayerSystem {
    pub fn new() -> Self {
        Self {
            action: PlayerAction::None,
        }
    }
}

impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        WriteStorage<'s, PlayerComponent>,
        WriteStorage<'s, PositionComponent>,
        WriteStorage<'s, QueuedAttack>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (mut player_data, mut position_data, mut queued_attack_data, entities): Self::SystemData,
    ) {
        match self.action {
            PlayerAction::Move(direction_to_move) => {
                let other_entity_positions = (!&player_data, &position_data)
                    .join()
                    .map(|(_, position)| (position.x, position.y))
                    .collect::<HashSet<(i32, i32)>>();
                let (_, player_position) = (&mut player_data, &mut position_data)
                    .join()
                    .next()
                    .unwrap();
                let mut new_player_x = player_position.x;
                let mut new_player_y = player_position.y;
                match direction_to_move {
                    Direction::Up => new_player_y += 1,
                    Direction::Down => new_player_y -= 1,
                    Direction::Left => new_player_x -= 1,
                    Direction::Right => new_player_x += 1,
                };
                if !other_entity_positions.contains(&(new_player_x, new_player_y)) {
                    player_position.x = new_player_x;
                    player_position.y = new_player_y;
                    player_position.facing_direction = direction_to_move;
                }
            }
            PlayerAction::TurnToFace(direction_to_face) => {
                let player_position = (&player_data, &mut position_data).join().next().unwrap().1;
                player_position.facing_direction = direction_to_face;
            }
            PlayerAction::Attack => {
                let (player_entity, _, player_position) =
                    (&entities, &mut player_data, &mut position_data)
                        .join()
                        .next()
                        .unwrap();
                let mut target_x = player_position.x;
                let mut target_y = player_position.y;
                match player_position.facing_direction {
                    Direction::Up => target_y += 1,
                    Direction::Down => target_y -= 1,
                    Direction::Left => target_x -= 1,
                    Direction::Right => target_x += 1,
                };
                let target_entity = (&entities, &position_data)
                    .join()
                    .find(|(_, position)| position.x == target_x && position.y == target_y)
                    .map(|(target_entity, _)| target_entity);
                if let Some(target_entity) = target_entity {
                    queued_attack_data
                        .insert(player_entity, QueuedAttack { target_entity })
                        .unwrap();
                }
            }
            PlayerAction::None => {}
        };

        self.action = PlayerAction::None;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum PlayerAction {
    None,
    Move(Direction),
    TurnToFace(Direction),
    Attack,
}

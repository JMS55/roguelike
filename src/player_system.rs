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
                    .map(|(_, position)| *position)
                    .collect::<HashSet<PositionComponent>>();
                let (player, player_position) = (&mut player_data, &mut position_data)
                    .join()
                    .next()
                    .unwrap();
                let new_player_position = match direction_to_move {
                    Direction::Up => PositionComponent {
                        x: player_position.x,
                        y: player_position.y + 1,
                    },
                    Direction::Down => PositionComponent {
                        x: player_position.x,
                        y: player_position.y - 1,
                    },
                    Direction::Left => PositionComponent {
                        x: player_position.x - 1,
                        y: player_position.y,
                    },
                    Direction::Right => PositionComponent {
                        x: player_position.x + 1,
                        y: player_position.y,
                    },
                };
                if !other_entity_positions.contains(&new_player_position) {
                    *player_position = new_player_position;
                    player.facing_direction = direction_to_move;
                }
            }
            PlayerAction::TurnToFace(direction_to_face) => {
                let player = (&mut player_data).join().next().unwrap();
                player.facing_direction = direction_to_face;
            }
            PlayerAction::Attack => {
                let (player_entity, player, player_position) =
                    (&entities, &mut player_data, &mut position_data)
                        .join()
                        .next()
                        .unwrap();
                let target_position = match player.facing_direction {
                    Direction::Up => PositionComponent {
                        x: player_position.x,
                        y: player_position.y + 1,
                    },
                    Direction::Down => PositionComponent {
                        x: player_position.x,
                        y: player_position.y - 1,
                    },
                    Direction::Left => PositionComponent {
                        x: player_position.x - 1,
                        y: player_position.y,
                    },
                    Direction::Right => PositionComponent {
                        x: player_position.x + 1,
                        y: player_position.y,
                    },
                };
                let target_entity = (&entities, &position_data)
                    .join()
                    .find(|(_, position)| position == &&target_position)
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

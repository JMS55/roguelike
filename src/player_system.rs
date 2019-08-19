use crate::components::{
    Direction, HealthComponent, PlayerComponent, PositionComponent, QueuedAttack, QueuedMovement,
};
use specs::{Entities, Join, ReadStorage, System, WriteStorage};

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
        ReadStorage<'s, HealthComponent>,
        WriteStorage<'s, QueuedMovement>,
        WriteStorage<'s, QueuedAttack>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (
            mut player_data,
            mut position_data,
            health_data,
            mut queued_movement_data,
            mut queued_attack_data,
            entities,
        ): Self::SystemData,
    ) {
        match self.action {
            PlayerAction::Move(direction_to_move) => {
                let (player_entity, _, player_position) =
                    (&entities, &mut player_data, &mut position_data)
                        .join()
                        .next()
                        .unwrap();
                let mut goal_x = player_position.x;
                let mut goal_y = player_position.y;
                match direction_to_move {
                    Direction::Up => goal_y += 1,
                    Direction::Down => goal_y -= 1,
                    Direction::Left => goal_x -= 1,
                    Direction::Right => goal_x += 1,
                };
                queued_movement_data
                    .insert(player_entity, QueuedMovement { goal_x, goal_y })
                    .unwrap();
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
                let target_entity = (&entities, &health_data, &position_data)
                    .join()
                    .find(|(_, _, position)| position.x == target_x && position.y == target_y)
                    .map(|(target_entity, _, _)| target_entity);
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

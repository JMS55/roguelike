use crate::components::{
    AIAttackPlayerComponent, MovementType, PlayerComponent, PositionComponent, QueuedAttack,
    QueuedMovement,
};
use specs::{Entities, Join, ReadStorage, System, WriteStorage};

pub struct AIAttackPlayerSystem {}

impl AIAttackPlayerSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'s> System<'s> for AIAttackPlayerSystem {
    type SystemData = (
        ReadStorage<'s, AIAttackPlayerComponent>,
        ReadStorage<'s, PositionComponent>,
        ReadStorage<'s, PlayerComponent>,
        WriteStorage<'s, QueuedAttack>,
        WriteStorage<'s, QueuedMovement>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (
            ai_attack_player_data,
            position_data,
            player_data,
            mut queued_attack_data,
            mut queued_movement_data,
            entities,
        ): Self::SystemData,
    ) {
        let (player_entity, _, player_position) = (&entities, &player_data, &position_data)
            .join()
            .next()
            .unwrap();
        for (_, ai_position, ai_entity) in
            (&ai_attack_player_data, &position_data, &entities).join()
        {
            if ai_position.is_next_to(player_position) {
                queued_attack_data
                    .insert(
                        ai_entity,
                        QueuedAttack {
                            target_entity: player_entity,
                        },
                    )
                    .unwrap();
            } else {
                queued_movement_data
                    .insert(
                        ai_entity,
                        QueuedMovement {
                            goal_x: player_position.x,
                            goal_y: player_position.y,
                            movement_type: MovementType::StandNextTo,
                        },
                    )
                    .unwrap();
            }
        }
    }
}

impl PositionComponent {
    pub fn is_next_to(&self, other: &Self) -> bool {
        match (self.x - other.x, self.y - other.y) {
            (1, 0) => true,
            (-1, 0) => true,
            (0, 1) => true,
            (0, -1) => true,
            _ => false,
        }
    }
}

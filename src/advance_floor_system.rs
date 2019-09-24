use crate::components::{Direction, PlayerComponent, PositionComponent, ShouldAdvanceFloor};
use specs::{Entities, Join, System, Write, WriteStorage};

pub struct AdvanceFloorSystem {}

impl AdvanceFloorSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'s> System<'s> for AdvanceFloorSystem {
    type SystemData = (
        WriteStorage<'s, PlayerComponent>,
        WriteStorage<'s, PositionComponent>,
        Entities<'s>,
        Write<'s, ShouldAdvanceFloor>,
    );

    fn run(
        &mut self,
        (mut player_data, mut position_data, entities, mut should_advance_floor): Self::SystemData,
    ) {
        let (player_entity, player, player_position) =
            (&entities, &mut player_data, &mut position_data)
                .join()
                .next()
                .unwrap();
        player.turns_taken = 0;
        *player_position = PositionComponent {
            x: 0,
            y: 0,
            facing_direction: Direction::Right,
        };

        for entity in (&entities).join() {
            if entity != player_entity {
                entities.delete(entity).unwrap();
            }
        }

        *should_advance_floor = ShouldAdvanceFloor(false);
    }
}

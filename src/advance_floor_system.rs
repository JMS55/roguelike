use crate::components::{
    Direction, MessageColor, MessageLog, PlayerComponent, PositionComponent, ShouldAdvanceFloor,
};
use specs::{Entities, Join, System, Write, WriteStorage};

pub struct AdvanceFloorSystem {
    current_floor: u32,
}

impl AdvanceFloorSystem {
    pub fn new() -> Self {
        Self { current_floor: 1 }
    }
}

impl<'s> System<'s> for AdvanceFloorSystem {
    type SystemData = (
        WriteStorage<'s, PlayerComponent>,
        WriteStorage<'s, PositionComponent>,
        Entities<'s>,
        Write<'s, ShouldAdvanceFloor>,
        Write<'s, MessageLog>,
    );

    fn run(
        &mut self,
        (mut player_data, mut position_data, entities, mut should_advance_floor, mut message_log): Self::SystemData,
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

        message_log.new_message(
            format!("Entering floor {}", self.current_floor),
            MessageColor::White,
        );
        self.current_floor += 1;
        *should_advance_floor = ShouldAdvanceFloor(false);
    }
}

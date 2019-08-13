use crate::components::{PlayerComponent, PositionComponent};
use specs::{Join, ReadStorage, System, WriteStorage};
use std::collections::HashSet;

pub struct PlayerMoveSystem {
    pub direction_to_move: Option<Direction>,
}

impl PlayerMoveSystem {
    pub fn new() -> Self {
        Self {
            direction_to_move: None,
        }
    }
}

impl<'s> System<'s> for PlayerMoveSystem {
    type SystemData = (
        ReadStorage<'s, PlayerComponent>,
        WriteStorage<'s, PositionComponent>,
    );

    fn run(&mut self, (player_data, mut position_data): Self::SystemData) {
        if let Some(direction_to_move) = self.direction_to_move {
            let other_entity_positions = (!&player_data, &position_data)
                .join()
                .map(|(_, position)| *position)
                .collect::<HashSet<PositionComponent>>();
            let player_position = (&player_data, &mut position_data).join().next().unwrap().1;
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
            }
            self.direction_to_move = None;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

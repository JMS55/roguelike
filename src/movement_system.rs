use crate::components::{Direction, PositionComponent, QueuedMovement};
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage};
use std::cmp::{Ord, Ordering};
use std::collections::{BinaryHeap, HashMap, HashSet};

pub struct MovementSystem {}

impl MovementSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'s> System<'s> for MovementSystem {
    type SystemData = (
        ReadStorage<'s, QueuedMovement>,
        WriteStorage<'s, PositionComponent>,
        Entities<'s>,
        Read<'s, LazyUpdate>,
    );

    fn run(
        &mut self,
        (queued_movement_data, mut position_data, entities, lazy_update): Self::SystemData,
    ) {
        for (moving_entity, movement_info) in (&entities, &queued_movement_data).join() {
            let obstacles = (&position_data)
                .join()
                .map(|position| (position.x, position.y))
                .collect::<HashSet<(i32, i32)>>();
            let moving_entity_position = position_data.get_mut(moving_entity).unwrap();

            let mut frontier = BinaryHeap::new();
            let mut came_from = HashMap::new();
            let mut cost_so_far = HashMap::new();
            frontier.push(FrontierNode {
                x: moving_entity_position.x,
                y: moving_entity_position.y,
                priority: 0,
            });
            cost_so_far.insert((moving_entity_position.x, moving_entity_position.y), 0);

            while let Some(visiting) = frontier.pop() {
                if visiting.x == movement_info.goal_x && visiting.y == movement_info.goal_y {
                    break;
                }
                for (neighbor_x, neighbor_y) in visiting.get_neighbors(&obstacles) {
                    let new_cost = cost_so_far[&(visiting.x, visiting.y)] + 1;
                    if !cost_so_far.contains_key(&(neighbor_x, neighbor_y))
                        || new_cost < cost_so_far[&(neighbor_x, neighbor_y)]
                    {
                        cost_so_far.insert((neighbor_x, neighbor_y), new_cost);
                        let priority = new_cost
                            + (neighbor_x - movement_info.goal_x).abs() as u32
                            + (neighbor_y - movement_info.goal_y).abs() as u32;
                        frontier.push(FrontierNode {
                            x: neighbor_x,
                            y: neighbor_y,
                            priority,
                        });
                        came_from.insert((neighbor_x, neighbor_y), (visiting.x, visiting.y));
                    }
                }
            }

            let mut entity_moved = true;
            let mut new_x = movement_info.goal_x;
            let mut new_y = movement_info.goal_y;
            loop {
                if let Some((previous_x, previous_y)) = came_from.get(&(new_x, new_y)) {
                    if previous_x == &moving_entity_position.x
                        && previous_y == &moving_entity_position.y
                    {
                        break;
                    } else {
                        new_x = *previous_x;
                        new_y = *previous_y;
                    }
                } else {
                    entity_moved = false;
                    break;
                }
            }

            if entity_moved {
                moving_entity_position.facing_direction = match (
                    new_x - moving_entity_position.x,
                    new_y - moving_entity_position.y,
                ) {
                    (0, 1) => Direction::Up,
                    (0, -1) => Direction::Down,
                    (-1, 0) => Direction::Left,
                    (1, 0) => Direction::Right,
                    _ => unreachable!(),
                };
                moving_entity_position.x = new_x;
                moving_entity_position.y = new_y;
            }

            lazy_update.remove::<QueuedMovement>(moving_entity);
        }
    }
}

#[derive(PartialEq, Eq)]
struct FrontierNode {
    x: i32,
    y: i32,
    priority: u32,
}

impl FrontierNode {
    fn get_neighbors(&self, obstacles: &HashSet<(i32, i32)>) -> Vec<(i32, i32)> {
        let mut neighbors = Vec::with_capacity(4);
        for (x_offset, y_offset) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if !obstacles.contains(&(self.x + x_offset, self.y + y_offset)) {
                neighbors.push((self.x + x_offset, self.y + y_offset));
            }
        }
        neighbors
    }
}

impl Ord for FrontierNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.x.cmp(&other.x))
            .then_with(|| self.y.cmp(&other.y))
    }
}

impl PartialOrd for FrontierNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

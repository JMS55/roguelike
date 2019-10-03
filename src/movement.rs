use crate::data::{Direction, Intangible, Position};
use specs::{Entity, Join, World, WorldExt};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

pub fn try_move(entity: Entity, direction: Direction, world: &mut World) -> Result<(), ()> {
    let obstacles;
    let entity_position;
    {
        let position_data = world.read_storage::<Position>();
        let intangible_data = world.read_storage::<Intangible>();
        obstacles = (&position_data, !&intangible_data)
            .join()
            .map(|(position, _)| (position.x, position.y))
            .collect::<HashSet<(i32, i32)>>();
        entity_position = *position_data.get(entity).unwrap();
    }

    let (new_x, new_y) = match direction {
        Direction::Up => (entity_position.x, entity_position.y + 1),
        Direction::Down => (entity_position.x, entity_position.y - 1),
        Direction::Left => (entity_position.x - 1, entity_position.y),
        Direction::Right => (entity_position.x + 1, entity_position.y),
    };

    if !obstacles.contains(&(new_x, new_y)) {
        try_turn(entity, direction, world)?;
        let mut position_data = world.write_storage::<Position>();
        let mut entity_position = position_data.get_mut(entity).unwrap();
        entity_position.x = new_x;
        entity_position.y = new_y;
        Ok(())
    } else {
        Err(())
    }
}

pub fn try_turn(entity: Entity, direction: Direction, world: &World) -> Result<(), ()> {
    let mut position_data = world.write_storage::<Position>();
    position_data.get_mut(entity).unwrap().facing_direction = direction;
    Ok(())
}

pub fn pathfind(
    start_x: i32,
    start_y: i32,
    goal_x: i32,
    goal_y: i32,
    world: &mut World,
) -> Vec<(i32, i32)> {
    let position_data = world.read_storage::<Position>();
    let intangible_data = world.read_storage::<Intangible>();
    let obstacles = (&position_data, !&intangible_data)
        .join()
        .map(|(position, _)| (position.x, position.y))
        .collect::<HashSet<(i32, i32)>>();
    let mut frontier = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut cost_so_far = HashMap::new();
    frontier.push(FrontierNode::new(start_x, start_y, 0));
    cost_so_far.insert((start_x, start_y), 0u32);

    let mut last_node = None;
    while let Some(visiting) = frontier.pop() {
        if visiting.is_next_to(goal_x, goal_y) {
            last_node = Some((visiting.x, visiting.y));
            break;
        }
        for next in visiting.neighbors(&obstacles) {
            let new_cost = cost_so_far[&(visiting.x, visiting.y)] + 1;
            if !cost_so_far.contains_key(&next) || new_cost < cost_so_far[&next] {
                cost_so_far.insert(next, new_cost);
                let priority = distance_from(next.0, next.1, goal_x, goal_y);
                frontier.push(FrontierNode::new(next.0, next.1, priority));
                came_from.insert(next, (visiting.x, visiting.y));
            }
        }
    }

    let mut path = Vec::with_capacity(distance_from(start_x, start_y, goal_x, goal_y) as usize);
    if let Some(mut last_node) = last_node {
        while let Some(next_node) = came_from.get(&last_node) {
            path.push(last_node);
            last_node = *next_node;
        }
        path.reverse();
    }
    path
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct FrontierNode {
    x: i32,
    y: i32,
    priority: u32,
}

impl FrontierNode {
    fn new(x: i32, y: i32, priority: u32) -> Self {
        Self { x, y, priority }
    }

    fn neighbors(&self, obstacles: &HashSet<(i32, i32)>) -> Vec<(i32, i32)> {
        let mut neighbors = Vec::with_capacity(4);
        for (x_offset, y_offset) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if !obstacles.contains(&(self.x + x_offset, self.y + y_offset)) {
                neighbors.push((self.x + x_offset, self.y + y_offset));
            }
        }
        neighbors
    }

    fn is_next_to(&self, other_x: i32, other_y: i32) -> bool {
        match (self.x - other_x, self.y - other_y) {
            (1, 0) => true,
            (-1, 0) => true,
            (0, 1) => true,
            (0, -1) => true,
            _ => false,
        }
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

fn distance_from(x1: i32, y1: i32, x2: i32, y2: i32) -> u32 {
    (x2 - x1).abs() as u32 + (y2 - y1).abs() as u32
}

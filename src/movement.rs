use crate::data::{Direction, Intangible, Position, AI};
use specs::{Entity, Join, World, WorldExt};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

pub fn try_move(entity: Entity, direction: Direction, world: &mut World) -> Result<(), ()> {
    if can_move(entity, direction, world) {
        let mut position_data = world.write_storage::<Position>();
        let entity_position = position_data.get_mut(entity).unwrap();
        *entity_position = entity_position.offset_by(direction);
        Ok(())
    } else {
        Err(())
    }
}

pub fn can_move(entity: Entity, direction: Direction, world: &World) -> bool {
    let obstacles;
    let entity_position;
    {
        let position_data = world.read_storage::<Position>();
        let intangible_data = world.read_storage::<Intangible>();
        obstacles = (&position_data, !&intangible_data)
            .join()
            .map(|(position, _)| *position)
            .collect::<HashSet<Position>>();
        entity_position = *position_data.get(entity).unwrap();
    }
    let new_position = entity_position.offset_by(direction);
    !obstacles.contains(&new_position)
}

pub fn pathfind(start: Position, goal: Position, world: &mut World) -> Vec<Position> {
    let position_data = world.read_storage::<Position>();
    let intangible_data = world.read_storage::<Intangible>();
    let ai_data = world.read_storage::<AI>();
    let obstacles = (&position_data, !&intangible_data, !&ai_data)
        .join()
        .map(|(position, _, _)| *position)
        .collect::<HashSet<Position>>();
    let mut frontier = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut cost_so_far = HashMap::new();
    frontier.push(FrontierNode::new(start, 0));
    cost_so_far.insert(start, 0u32);

    let mut last_node = None;
    while let Some(visiting) = frontier.pop() {
        if visiting.position.distance_from(goal) == 1 {
            last_node = Some(visiting);
            break;
        }
        for next in visiting.position.neighbors(&obstacles) {
            let new_cost = cost_so_far[&visiting.position] + 1;
            if !cost_so_far.contains_key(&next) || new_cost < cost_so_far[&next] {
                cost_so_far.insert(next, new_cost);
                frontier.push(FrontierNode::new(next, next.distance_from(goal)));
                came_from.insert(next, visiting.position);
            }
        }
    }

    let mut path = Vec::with_capacity(start.distance_from(goal) as usize);
    if let Some(last_node) = last_node {
        let mut last_node = last_node.position;
        while let Some(next_node) = came_from.get(&last_node) {
            path.push(last_node);
            last_node = *next_node;
        }
        path.reverse();
    }
    path
}

pub fn try_move_towards(
    moving_entity: Entity,
    target: Entity,
    world: &mut World,
) -> Result<(), ()> {
    let (moving_entity_position, target_position) = {
        let position_data = world.read_storage::<Position>();
        (
            *position_data.get(moving_entity).unwrap(),
            *position_data.get(target).unwrap(),
        )
    };
    let path = pathfind(moving_entity_position, target_position, world);
    if let Some(new_position) = path.get(0) {
        let direction = match (
            new_position.x - moving_entity_position.x,
            new_position.y - moving_entity_position.y,
        ) {
            (0, 1) => Direction::Up,
            (0, -1) => Direction::Down,
            (-1, 0) => Direction::Left,
            (1, 0) => Direction::Right,
            (-1, 1) => Direction::UpLeft,
            (-1, -1) => Direction::DownLeft,
            (1, -1) => Direction::DownRight,
            (1, 1) => Direction::UpRight,
            _ => unreachable!(),
        };
        try_move(moving_entity, direction, world)?
    }
    Ok(())
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct FrontierNode {
    position: Position,
    priority: u32,
}

impl FrontierNode {
    fn new(position: Position, priority: u32) -> Self {
        Self { position, priority }
    }
}

impl Ord for FrontierNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.position.x.cmp(&other.position.x))
            .then_with(|| self.position.y.cmp(&other.position.y))
    }
}

impl PartialOrd for FrontierNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use crate::components::*;
use crate::game::Game;
use legion::entity::Entity;
use legion::filter::filter_fns;
use legion::query::{IntoQuery, Read};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

pub fn try_move_towards(
    moving_entity: Entity,
    goal: PositionComponent,
    game: &mut Game,
) -> Result<(), ()> {
    let moving_entity_position = *game
        .world
        .get_component::<PositionComponent>(moving_entity)
        .unwrap();
    let path = pathfind(moving_entity_position, goal, game);
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
        try_move(moving_entity, direction, game)?
    } else {
        return Err(());
    }
    Ok(())
}

pub fn try_move(entity: Entity, direction: Direction, game: &mut Game) -> Result<(), ()> {
    if entity == game.player_entity {
        turn_player_towards(direction, game);
    }

    if can_move(entity, direction, game) {
        let offset = match direction {
            Direction::Up => PositionComponent { x: 0, y: 1 },
            Direction::Down => PositionComponent { x: 0, y: -1 },
            Direction::Left => PositionComponent { x: -1, y: 0 },
            Direction::Right => PositionComponent { x: 1, y: 0 },
            Direction::UpLeft => PositionComponent { x: -1, y: 1 },
            Direction::UpRight => PositionComponent { x: 1, y: 1 },
            Direction::DownLeft => PositionComponent { x: -1, y: -1 },
            Direction::DownRight => PositionComponent { x: 1, y: -1 },
        };
        let mut entity_position = game
            .world
            .get_component_mut::<PositionComponent>(entity)
            .unwrap();
        let new_entity_position = PositionComponent {
            x: entity_position.x + offset.x,
            y: entity_position.y + offset.y,
        };
        *entity_position = new_entity_position;
        Ok(())
    } else {
        Err(())
    }
}

pub fn can_move(entity: Entity, direction: Direction, game: &Game) -> bool {
    let offset = match direction {
        Direction::Up => PositionComponent { x: 0, y: 1 },
        Direction::Down => PositionComponent { x: 0, y: -1 },
        Direction::Left => PositionComponent { x: -1, y: 0 },
        Direction::Right => PositionComponent { x: 1, y: 0 },
        Direction::UpLeft => PositionComponent { x: -1, y: 1 },
        Direction::UpRight => PositionComponent { x: 1, y: 1 },
        Direction::DownLeft => PositionComponent { x: -1, y: -1 },
        Direction::DownRight => PositionComponent { x: 1, y: -1 },
    };
    let entity_position = game
        .world
        .get_component::<PositionComponent>(entity)
        .unwrap();
    let new_entity_position = PositionComponent {
        x: entity_position.x + offset.x,
        y: entity_position.y + offset.y,
    };
    Read::<PositionComponent>::query()
        .iter_immutable(&game.world)
        .all(|position| *position != new_entity_position)
}

pub fn turn_player_towards(direction: Direction, game: &mut Game) {
    game.world
        .get_component_mut::<PlayerComponent>(game.player_entity)
        .unwrap()
        .facing_direction = direction;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

fn pathfind(
    start: PositionComponent,
    goal: PositionComponent,
    game: &Game,
) -> Vec<PositionComponent> {
    let obstacles = Read::<PositionComponent>::query()
        .filter(!filter_fns::component::<AIComponent>())
        .iter_immutable(&game.world)
        .map(|position| *position)
        .collect::<HashSet<PositionComponent>>();
    let mut frontier = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut cost_so_far = HashMap::new();
    frontier.push(FrontierNode {
        position: start,
        priority: 0,
    });
    cost_so_far.insert(start, 0);

    let mut end_node = None;
    while let Some(visiting) = frontier.pop() {
        if visiting.position.distance_from(goal) == 1 {
            end_node = Some(visiting.position);
            break;
        }
        for next in visiting.position.neighbors(&obstacles) {
            let new_cost = cost_so_far[&visiting.position] + 1;
            if !cost_so_far.contains_key(&next) || new_cost < cost_so_far[&next] {
                cost_so_far.insert(next, new_cost);
                frontier.push(FrontierNode {
                    position: next,
                    priority: next.distance_from(goal),
                });
                came_from.insert(next, visiting.position);
            }
        }
    }

    if let Some(end_node) = end_node {
        let mut path = Vec::with_capacity(start.distance_from(end_node) as usize);
        let mut current = end_node;
        while current != start {
            path.push(current);
            current = *came_from.get(&current).unwrap();
        }
        path.reverse();
        path
    } else {
        Vec::with_capacity(0)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct FrontierNode {
    position: PositionComponent,
    priority: u16,
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

impl PositionComponent {
    fn distance_from(self, other: Self) -> u16 {
        (other.x - self.x).abs() as u16 + (other.y - self.y).abs() as u16
    }

    fn neighbors(self, obstacles: &HashSet<Self>) -> Vec<Self> {
        let mut neighbors = Vec::with_capacity(4);
        for (x_offset, y_offset) in &[
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let mut new_position = self;
            new_position.x += x_offset;
            new_position.y += y_offset;
            if !obstacles.contains(&new_position) {
                neighbors.push(new_position);
            }
        }
        neighbors
    }
}

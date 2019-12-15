use crate::components::{PlayerComponent, PositionComponent};
use crate::game::Game;
use legion::entity::Entity;
use legion::query::{IntoQuery, Read};

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

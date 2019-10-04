use crate::attack::*;
use crate::data::*;
use crate::movement::*;
use rand::Rng;
use specs::{Builder, Join, World, WorldExt};

pub fn create_phase_bat(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Phase Bat"))
        .with(AI::new(|ai_entity, world| {
            let (ai_position, player_entity, player_position) = {
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();

                let ai_position = position_data.get(ai_entity).unwrap();
                let (player_entity, _, player_position) = (&entities, &player_data, &position_data)
                    .join()
                    .next()
                    .unwrap();

                (*ai_position, player_entity, *player_position)
            };

            if ai_position.can_attack(1, &player_position) {
                let attack_twice = {
                    let mut rng_data = world.write_storage::<RNG>();
                    let ai_rng = rng_data.get_mut(ai_entity).unwrap();
                    ai_rng.0.gen_ratio(1, 10)
                };
                attack(Damage(3), Range(1), ai_entity, player_entity, world);
                if attack_twice {
                    attack(Damage(2), Range(1), ai_entity, player_entity, world);
                }
            } else {
                let path = pathfind(
                    ai_position.x,
                    ai_position.y,
                    player_position.x,
                    player_position.y,
                    world,
                );
                if let Some((new_x, new_y)) = path.get(0) {
                    let direction = match (new_x - ai_position.x, new_y - ai_position.y) {
                        (0, 1) => Direction::Up,
                        (0, -1) => Direction::Down,
                        (-1, 0) => Direction::Left,
                        (1, 0) => Direction::Right,
                        _ => unreachable!(),
                    };
                    let _ = try_move(ai_entity, direction, world);
                }
            }
        }))
        .with(Position::new(x, y))
        .with(Attackable::new(6))
        .with(Sprite::new("green"))
        .with(RNG::new())
        .build();
}

pub fn create_danger_spider(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Danger! Spider"))
        .with(AI::new(|ai_entity, world| {
            let (ai_position, player_entity, player_position) = {
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();

                let ai_position = position_data.get(ai_entity).unwrap();
                let (player_entity, _, player_position) = (&entities, &player_data, &position_data)
                    .join()
                    .next()
                    .unwrap();

                (*ai_position, player_entity, *player_position)
            };

            if ai_position.can_attack(1, &player_position) {
                attack(Damage(3), Range(1), ai_entity, player_entity, world);
            } else {
                let path = pathfind(
                    ai_position.x,
                    ai_position.y,
                    player_position.x,
                    player_position.y,
                    world,
                );
                if let Some((new_x, new_y)) = path.get(0) {
                    let direction = match (new_x - ai_position.x, new_y - ai_position.y) {
                        (0, 1) => Direction::Up,
                        (0, -1) => Direction::Down,
                        (-1, 0) => Direction::Left,
                        (1, 0) => Direction::Right,
                        _ => unreachable!(),
                    };
                    let _ = try_move(ai_entity, direction, world);
                }
            }
        }))
        .with(Position::new(x, y))
        .with(Attackable::new(5))
        .with(Sprite::new("green"))
        .build();
}

pub fn create_pungent_ooze(x: i32, y: i32, world: &mut World) {
    let mut attackable = Attackable::new(7);
    attackable.has_oozing_buff = true;
    world
        .create_entity()
        .with(Name("Pungent Ooze"))
        .with(AI::new(|ai_entity, world| {
            let (ai_position, player_entity, player_position) = {
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();

                let ai_position = position_data.get(ai_entity).unwrap();
                let (player_entity, _, player_position) = (&entities, &player_data, &position_data)
                    .join()
                    .next()
                    .unwrap();

                (*ai_position, player_entity, *player_position)
            };

            if ai_position.can_attack(1, &player_position) {
                attack(Damage(4), Range(1), ai_entity, player_entity, world);
            } else {
                let path = pathfind(
                    ai_position.x,
                    ai_position.y,
                    player_position.x,
                    player_position.y,
                    world,
                );
                if let Some((new_x, new_y)) = path.get(0) {
                    let direction = match (new_x - ai_position.x, new_y - ai_position.y) {
                        (0, 1) => Direction::Up,
                        (0, -1) => Direction::Down,
                        (-1, 0) => Direction::Left,
                        (1, 0) => Direction::Right,
                        _ => unreachable!(),
                    };
                    let _ = try_move(ai_entity, direction, world);
                }
            }
        }))
        .with(Position::new(x, y))
        .with(attackable)
        .with(Sprite::new("green"))
        .build();
}

pub fn create_skeleton_scout(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Skeleton Scout"))
        .with(AI::new(|ai_entity, world| {
            let (ai_position, player_entity, player_position) = {
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();

                let ai_position = position_data.get(ai_entity).unwrap();
                let (player_entity, _, player_position) = (&entities, &player_data, &position_data)
                    .join()
                    .next()
                    .unwrap();

                (*ai_position, player_entity, *player_position)
            };

            if ai_position.can_attack(2, &player_position) {
                let move_before_attacking = {
                    let mut rng_data = world.write_storage::<RNG>();
                    let ai_rng = rng_data.get_mut(ai_entity).unwrap();
                    ai_rng.0.gen_ratio(1, 4)
                };
                let mut range = 2;
                if move_before_attacking {
                    let new_ai_position = {
                        let position_data = world.read_storage::<Position>();
                        *position_data.get(ai_entity).unwrap()
                    };
                    let change_in_x = new_ai_position.x - player_position.x;
                    let change_in_y = new_ai_position.y - player_position.y;
                    let mut direction_to_move_in = Direction::Up;
                    if change_in_x < 0 {
                        direction_to_move_in = Direction::Left;
                    }
                    if change_in_x > 0 {
                        direction_to_move_in = Direction::Right;
                    }
                    if change_in_y < 0 {
                        direction_to_move_in = Direction::Down;
                    }
                    if try_move(ai_entity, direction_to_move_in, world).is_ok() {
                        range = 3;
                    }
                }
                attack(Damage(5), Range(range), ai_entity, player_entity, world);
            } else {
                let path = pathfind(
                    ai_position.x,
                    ai_position.y,
                    player_position.x,
                    player_position.y,
                    world,
                );
                if let Some((new_x, new_y)) = path.get(0) {
                    let direction = match (new_x - ai_position.x, new_y - ai_position.y) {
                        (0, 1) => Direction::Up,
                        (0, -1) => Direction::Down,
                        (-1, 0) => Direction::Left,
                        (1, 0) => Direction::Right,
                        _ => unreachable!(),
                    };
                    let _ = try_move(ai_entity, direction, world);
                }
            }
        }))
        .with(Position::new(x, y))
        .with(Attackable::new(9))
        .with(Sprite::new("green"))
        .with(RNG::new())
        .build();
}

impl Position {
    fn can_attack(&self, range: u32, other: &Self) -> bool {
        let range = range as i32;
        let straight_path = self.x == other.x || self.y == other.y;
        let in_range = (self.x - other.x).abs() + (self.y - other.y).abs() <= range;
        straight_path && in_range
    }
}

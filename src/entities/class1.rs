use crate::attack::*;
use crate::data::*;
use crate::movement::*;
use rand::Rng;
use specs::{Builder, Entity, Join, World, WorldExt};

pub fn create_phase_bat(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Phase Bat"))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            match try_attack(3, 1, 1, ai_entity, player_entity, world) {
                Ok(true) => {}
                Ok(false) => {
                    let attack_twice = {
                        let mut rng = world.fetch_mut::<RNG>();
                        rng.0.gen_ratio(1, 10)
                    };
                    if attack_twice {
                        let _ = try_attack(2, 1, 1, ai_entity, player_entity, world);
                    }
                }
                Err(_) => {
                    let _ = try_move_towards(ai_entity, player_entity, world);
                }
            }
        }))
        .with(Position::new(x, y))
        .with(Attackable::new(6))
        .with(Sprite::new("green"))
        .build();
}

pub fn create_danger_spider(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Danger! Spider"))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(3, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
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
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(3, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
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

            if can_attack(1, 2, ai_entity, player_entity, world) {
                let change_in_x = ai_position.x - player_position.x;
                let change_in_y = ai_position.y - player_position.y;
                let move_before_attacking = {
                    let mut rng = world.fetch_mut::<RNG>();
                    rng.0.gen_ratio(1, 4) && (change_in_x.abs() == 1 || change_in_y.abs() == 1)
                };
                if move_before_attacking {
                    let mut direction_to_move = Direction::Up;
                    if change_in_x < 0 {
                        direction_to_move = Direction::Left;
                    }
                    if change_in_x > 0 {
                        direction_to_move = Direction::Right;
                    }
                    if change_in_y < 0 {
                        direction_to_move = Direction::Down;
                    }
                    let _ = try_move(ai_entity, direction_to_move, world);
                    let _ = try_turn(ai_entity, direction_to_move.opposite(), world);
                }
                let _ = try_attack(5, 1, 2, ai_entity, player_entity, world);
            } else {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(Position::new(x, y))
        .with(Attackable::new(9))
        .with(Sprite::new("green"))
        .build();
}

pub fn create_volatile_husk(x: i32, y: i32, world: &mut World) {
    let mut attackable = Attackable::new(6);
    attackable.on_death = Some(|ai_entity, _, world| {
        let ai_position = {
            let position_data = world.read_storage::<Position>();
            *position_data.get(ai_entity).unwrap()
        };
        let targets = {
            let attackable_data = world.read_storage::<Attackable>();
            let position_data = world.read_storage::<Position>();
            let entities = world.entities();
            (&entities, &position_data, &attackable_data)
                .join()
                .filter(|(_, position, _)| {
                    position.x - ai_position.x <= 2 && position.y - ai_position.y <= 2
                })
                .map(|(entity, _, _)| entity)
                .collect::<Vec<Entity>>()
        };
        for target in targets {
            if target != ai_entity {
                damage(6, false, ai_entity, target, world);
            }
        }
        let mut message_log = world.fetch_mut::<MessageLog>();
        message_log.new_message(
            "Volatile Husk exploded!",
            MessageColor::White,
            MessageDisplayLength::Medium,
        );
    });

    world
        .create_entity()
        .with(Name("Volatile Husk"))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(2, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(Position::new(x, y))
        .with(attackable)
        .with(Sprite::new("green"))
        .build();
}

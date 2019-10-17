use crate::attack::*;
use crate::data::*;
use crate::movement::*;
use rand::seq::SliceRandom;
use rand::Rng;
use specs::{Builder, Join, World, WorldExt};
use std::collections::HashSet;

pub fn create_random_layer1(rarity: Rarity, position: Position, world: &mut World) {
    let create_function = {
        let rng = &mut world.fetch_mut::<RNG>().0;
        let choices: Vec<fn(Position, &mut World)> = match rarity {
            Rarity::Common => vec![create_phase_bat, create_danger_spider, create_pungent_ooze],
            Rarity::Uncommon => vec![create_skeleton_scout, create_volatile_husk],
            Rarity::Rare => vec![create_jack_spectre],
            Rarity::Epic => vec![create_king_of_lanterns, create_moth_priestess],
        };
        *choices.choose(rng).unwrap()
    };
    (create_function)(position, world);
}

pub fn create_phase_bat(position: Position, world: &mut World) {
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
                        let rng = &mut world.fetch_mut::<RNG>().0;
                        rng.gen_ratio(1, 7)
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
        .with(position)
        .with(Attackable::new(6, false))
        .with(Sprite::new("phase_bat"))
        .build();
}

pub fn create_danger_spider(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Danger! Spider"))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(5, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(Attackable::new(7, false))
        .with(Sprite::new("danger_spider"))
        .build();
}

pub fn create_pungent_ooze(position: Position, world: &mut World) {
    let mut attackable = Attackable::new(7, false);
    attackable.is_oozing = true;
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
        .with(position)
        .with(attackable)
        .with(Sprite::new("pungent_ooze"))
        .build();
}

pub fn create_skeleton_scout(position: Position, world: &mut World) {
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
                    let rng = &mut world.fetch_mut::<RNG>().0;
                    rng.gen_ratio(1, 4) && (change_in_x.abs() == 1 || change_in_y.abs() == 1)
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
                }
                let _ = try_attack(5, 1, 2, ai_entity, player_entity, world);
            } else {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(Attackable::new(9, false))
        .with(Sprite::new("skeleton_scout"))
        .build();
}

pub fn create_volatile_husk(position: Position, world: &mut World) {
    let mut attackable = Attackable::new(6, false);
    attackable.explode_on_death = (6, 2);
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
        .with(position)
        .with(attackable)
        .with(Sprite::new("volatile_husk"))
        .build();
}

pub fn create_jack_spectre(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Jack Spectre"))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            let mut attacked_this_turn = {
                let ai_counter_data = world.read_storage::<AICounter>();
                ai_counter_data.get(ai_entity).unwrap().0 == 1
            };
            for _ in 0..2 {
                if can_attack(1, 1, ai_entity, player_entity, world) {
                    if attacked_this_turn {
                        let (ai_position, player_position) = {
                            let position_data = world.read_storage::<Position>();
                            let ai_position = position_data.get(ai_entity).unwrap();
                            let player_position = position_data.get(player_entity).unwrap();
                            (*ai_position, *player_position)
                        };
                        let distance_from_player = player_position.distance_from(ai_position);
                        for direction in &[
                            Direction::Up,
                            Direction::Down,
                            Direction::Left,
                            Direction::Right,
                            Direction::UpLeft,
                            Direction::DownLeft,
                            Direction::DownRight,
                            Direction::UpRight,
                        ] {
                            let new_ai_position = ai_position.offset_by(*direction);
                            if can_move(ai_entity, *direction, &world)
                                && player_position.distance_from(new_ai_position)
                                    > distance_from_player
                            {
                                let _ = try_move(ai_entity, *direction, world);
                                break;
                            }
                        }

                        let spawn_creature = {
                            let rng = &mut world.fetch_mut::<RNG>().0;
                            rng.gen_ratio(1, 6)
                        };
                        if spawn_creature {
                            let obstacles = {
                                let position_data = world.read_storage::<Position>();
                                let intangible_data = world.read_storage::<Intangible>();
                                (&position_data, !&intangible_data)
                                    .join()
                                    .map(|(position, _)| *position)
                                    .collect::<HashSet<Position>>()
                            };
                            for direction in &[
                                Direction::Up,
                                Direction::Down,
                                Direction::Left,
                                Direction::Right,
                            ] {
                                let spawn_position = ai_position.offset_by(*direction);
                                if !obstacles.contains(&spawn_position) {
                                    create_random_layer1(Rarity::Common, spawn_position, world);
                                    break;
                                }
                            }
                        }
                    } else {
                        let _ = try_attack(6, 1, 1, ai_entity, player_entity, world);
                        attacked_this_turn = true;
                    }
                } else {
                    let _ = try_move_towards(ai_entity, player_entity, world);
                }
            }
            let mut ai_counter_data = world.write_storage::<AICounter>();
            let _ = ai_counter_data.insert(ai_entity, AICounter(0));
        }))
        .with(AICounter(0))
        .with(position)
        .with(Attackable::new(7, false))
        .with(Sprite::new("jack_spectre"))
        .build();
}

pub fn create_king_of_lanterns(position: Position, world: &mut World) {
    let attackable = Attackable::new(47, true);
    world
        .create_entity()
        .with(Name("Siro, King of the Lanterns"))
        // TODO
        // .with(AI::new(|ai_entity, world| {}))
        .with(position) // TODO: generate staircase on death
        .with(attackable)
        .with(Sprite::new("placeholder"))
        .build();
}

pub fn create_moth_priestess(position: Position, world: &mut World) {
    let attackable = Attackable::new(47, true);
    world
        .create_entity()
        .with(Name("Xilphne, The Moth Priestess"))
        // TODO
        // .with(AI::new(|ai_entity, world| {}))
        .with(position)
        .with(attackable)
        .with(Sprite::new("placeholder"))
        .build();
}

pub fn create_moth_worshipper(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Moth Worshipper"))
        // TODO
        // .with(AI::new(|ai_entity, world| {}))
        .with(position)
        .with(Attackable::new(12, false))
        .with(Sprite::new("placeholder"))
        .build();
}

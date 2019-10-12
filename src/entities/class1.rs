use super::replace_with_staircase_on_death;
use crate::attack::*;
use crate::data::*;
use crate::movement::*;
use rand::seq::SliceRandom;
use rand::Rng;
use specs::{Builder, Entity, Join, World, WorldExt};
use std::collections::HashSet;

pub fn create_random_class1(rarity: Rarity, x: i32, y: i32, world: &mut World) {
    let create_function = {
        let rng = &mut world.fetch_mut::<RNG>().0;
        let choices: Vec<fn(i32, i32, &mut World)> = match rarity {
            Rarity::Common => vec![create_phase_bat, create_danger_spider, create_pungent_ooze],
            Rarity::Uncommon => vec![create_skeleton_scout, create_volatile_husk],
            Rarity::Rare => vec![create_jack_spectre],
            Rarity::Epic => vec![create_king_of_lanterns, create_moth_priestess],
        };
        *choices.choose(rng).unwrap()
    };
    (create_function)(x, y, world);
}

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
        .with(Position::new(x, y))
        .with(Attackable::new(6))
        .with(Sprite::new("phase_bat"))
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
            if try_attack(5, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(Position::new(x, y))
        .with(Attackable::new(7))
        .with(Sprite::new("danger_spider"))
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
        .with(Sprite::new("pungent_ooze"))
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
                    let _ = try_turn(ai_entity, direction_to_move.opposite(), world);
                }
                let _ = try_attack(5, 1, 2, ai_entity, player_entity, world);
            } else {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(Position::new(x, y))
        .with(Attackable::new(9))
        .with(Sprite::new("skeleton_scout"))
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
            damage(6, false, Some(ai_entity), target, world);
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
        .with(Sprite::new("volatile_husk"))
        .build();
}

pub fn create_jack_spectre(x: i32, y: i32, world: &mut World) {
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
                        let distance_from_player = player_position.distance_from(&ai_position);
                        for direction in &[
                            Direction::Up,
                            Direction::Down,
                            Direction::Left,
                            Direction::Right,
                        ] {
                            let new_ai_position = match direction {
                                Direction::Up => (ai_position.x, ai_position.y + 1),
                                Direction::Down => (ai_position.x, ai_position.y - 1),
                                Direction::Left => (ai_position.x - 1, ai_position.y),
                                Direction::Right => (ai_position.x + 1, ai_position.y),
                            };
                            if can_move(ai_entity, *direction, &world)
                                && player_position.distance_from_tuple(new_ai_position)
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
                                    .map(|(position, _)| (position.x, position.y))
                                    .collect::<HashSet<(i32, i32)>>()
                            };
                            for direction in &[
                                Direction::Up,
                                Direction::Down,
                                Direction::Left,
                                Direction::Right,
                            ] {
                                let (spawn_position_x, spawn_position_y) = match direction {
                                    Direction::Up => (ai_position.x, ai_position.y + 1),
                                    Direction::Down => (ai_position.x, ai_position.y - 1),
                                    Direction::Left => (ai_position.x - 1, ai_position.y),
                                    Direction::Right => (ai_position.x + 1, ai_position.y),
                                };
                                if !obstacles.contains(&(spawn_position_x, spawn_position_y)) {
                                    let _ = create_random_class1(
                                        Rarity::Common,
                                        spawn_position_x,
                                        spawn_position_y,
                                        world,
                                    );
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
        .with(Position::new(x, y))
        .with(Attackable::new(7))
        .with(Sprite::new("jack_spectre"))
        .build();
}

pub fn create_king_of_lanterns(x: i32, y: i32, world: &mut World) {
    let mut attackable = Attackable::new(47);
    attackable.on_death = Some(replace_with_staircase_on_death);
    world
        .create_entity()
        .with(Name("King of the Lanterns"))
        // TODO
        // .with(AI::new(|ai_entity, world| {}))
        .with(Position::new(x, y)) // TODO: generate staircase on death
        .with(attackable)
        .with(Sprite::new("placeholder"))
        .build();
}

pub fn create_king_of_lanterns_illusion(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("King of the Lanterns"))
        .with(Position::new(x, y))
        .with(Attackable::new(1))
        .with(Sprite::new("placeholder"))
        .build();
}

pub fn create_king_of_lanterns_flame(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Pillar of Flame"))
        // TODO
        // .with(AI::new(|ai_entity, world| {}))
        .with(Position::new(x, y))
        .with(Intangible {})
        .with(Sprite::new("orange"))
        .build();
}

pub fn create_moth_priestess(x: i32, y: i32, world: &mut World) {
    let mut attackable = Attackable::new(47);
    attackable.on_death = Some(replace_with_staircase_on_death);
    world
        .create_entity()
        .with(Name("The Moth Priestess"))
        // TODO
        // .with(AI::new(|ai_entity, world| {}))
        .with(Position::new(x, y))
        .with(attackable)
        .with(Sprite::new("placeholder"))
        .build();
}

pub fn create_moth_worshipper(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Moth Worshipper"))
        // TODO
        // .with(AI::new(|ai_entity, world| {}))
        .with(Position::new(x, y))
        .with(Attackable::new(12))
        .with(Sprite::new("placeholder"))
        .build();
}

impl Position {
    pub fn distance_from_tuple(&self, (other_x, other_y): (i32, i32)) -> u32 {
        (self.x - other_x).abs() as u32 + (self.y - other_y).abs() as u32
    }
}

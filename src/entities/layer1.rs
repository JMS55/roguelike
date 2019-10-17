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
            Rarity::Rare => vec![create_arcane_ooze, create_soul_spectre],
            Rarity::Epic => vec![create_siro_king_of_hell, create_xilphene_the_moth_priestess],
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
                        rng.gen_ratio(1, 5)
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
        .with(Attackable::new(8, 20, false))
        .with(Sprite::new("phase_bat"))
        .build();
}

pub fn create_danger_spider(position: Position, world: &mut World) {
    let mut attackable = Attackable::new(11, 20, false);
    attackable.lower_spawn_times = (0.5, 3);
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
        .with(attackable)
        .with(Sprite::new("danger_spider"))
        .build();
}

pub fn create_pungent_ooze(position: Position, world: &mut World) {
    let mut attackable = Attackable::new(10, 20, false);
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
            if try_attack(4, 1, 1, ai_entity, player_entity, world).is_err() {
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
                let _ = try_attack(4, 1, 2, ai_entity, player_entity, world);
            } else {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(Attackable::new(12, 30, false))
        .with(Sprite::new("skeleton_scout"))
        .build();
}

pub fn create_volatile_husk(position: Position, world: &mut World) {
    let mut attackable = Attackable::new(9, 30, false);
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

pub fn create_arcane_ooze(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Arcane Ooze"))
        .with(AI::new(|ai_entity, world| {
            // TODO
        }))
        .with(position)
        .with(Attackable::new(10, 50, false))
        .with(Sprite::new("arcane_ooze"))
        .build();
}

pub fn create_soul_spectre(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Soul Spectre"))
        .with(AI::new(|ai_entity, world| {
            let has_been_attacked = {
                let mut ai_counter_data = world.write_storage::<AICounter>();
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let attackable_data = world.read_storage::<Attackable>();
                let ai_counter = ai_counter_data.get_mut(ai_entity).unwrap();
                let ai_position = position_data.get(ai_entity).unwrap();
                let ai_attackable = attackable_data.get(ai_entity).unwrap();
                let player_position = (&player_data, &position_data).join().next().unwrap().1;

                if *ai_counter == AICounter(0) && ai_position.distance_from(*player_position) <= 6 {
                    let mut message_log = world.fetch_mut::<MessageLog>();
                    message_log.new_message(
                        "Hello there. May I have your soul?",
                        MessageColor::White,
                        MessageDisplayLength::Medium,
                    );
                    *ai_counter = AICounter(1);
                }

                if ai_attackable.current_health != ai_attackable.max_health
                    && ai_position.distance_from(*player_position) <= 6
                {
                    if *ai_counter != AICounter(2)
                    {
                    let mut message_log = world.fetch_mut::<MessageLog>();
                    message_log.new_message("Wow, that was rude. All I was asking for was your immortal soul, no need to overreact. Now I'm ANRGY!", MessageColor::Red, MessageDisplayLength::Medium);
                    }
                    *ai_counter = AICounter(2);
                }

                *ai_counter == AICounter(2)
            };

            if has_been_attacked {
                let spawn_discordant_soul = {
                    let rng = &mut world.fetch_mut::<RNG>().0;
                    rng.gen_ratio(1, 12)
                };
                if spawn_discordant_soul {
                    let ai_position = {
                        let position_data = world.read_storage::<Position>();
                        *position_data.get(ai_entity).unwrap()
                    };
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
                            create_discordant_soul(spawn_position, world);
                            break;
                        }
                    }
                } else {
                    let player_entity = {
                        let player_data = world.read_storage::<Player>();
                        let entities = world.entities();
                        (&entities, &player_data).join().next().unwrap().0
                    };
                    if try_attack(5, 1, 1, ai_entity, player_entity, world).is_err() {
                        let _ = try_move_towards(ai_entity, player_entity, world);
                        let _ = try_move_towards(ai_entity, player_entity, world);
                    }
                }
            }
        }))
        .with(AICounter(0))
        .with(position)
        .with(Attackable::new(16, 50, false))
        .with(Sprite::new("soul_spectre"))
        .build();
}

pub fn create_discordant_soul(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Discordant Soul"))
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
        .with(Attackable::new(6, 5, false))
        .with(Sprite::new("discordant_soul"))
        .build();
}

pub fn create_siro_king_of_hell(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Siro, King of Hell"))
        .with(AI::new(|ai_entity, world| {
            // TODO
        }))
        .with(position)
        .with(Attackable::new(50, 200, true))
        .with(Sprite::new("placeholder"))
        .build();
}

pub fn create_xilphene_the_moth_priestess(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Xilphene, The Moth_ Priestes"))
        .with(AI::new(|ai_entity, world| {
            // TODO
        }))
        .with(position)
        .with(Attackable::new(40, 200, true))
        .with(Sprite::new("placeholder"))
        .build();
}

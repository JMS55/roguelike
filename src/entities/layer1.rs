use crate::attack::*;
use crate::data::*;
use crate::items;
use crate::movement::*;
use rand::seq::SliceRandom;
use rand::Rng;
use specs::{Builder, Entity, Join, World, WorldExt};
use std::collections::HashSet;

pub fn create_random_layer1(rarity: Rarity, position: Position, world: &mut World) -> Entity {
    let create_function = {
        let rng = &mut world.fetch_mut::<RNG>().0;
        let choices: Vec<fn(Position, &mut World) -> Entity> = match rarity {
            Rarity::Common => vec![create_phase_bat, create_danger_spider, create_pungent_ooze],
            Rarity::Uncommon => vec![create_skeleton_scout, create_volatile_husk],
            Rarity::Rare => vec![create_arcane_ooze, create_soul_spectre],
            Rarity::Epic => vec![
                create_siro_king_of_hell,
                create_xilphene_the_moth_priestess,
                create_ume_the_dungeon_heart,
            ],
        };
        *choices.choose(rng).unwrap()
    };
    (create_function)(position, world)
}

pub fn create_phase_bat(position: Position, world: &mut World) -> Entity {
    let attackable = Attackable::new(
        9,
        20,
        items::create_random_layer1(Rarity::Common, world),
        false,
    );
    world
        .create_entity()
        .with(Name::new("Phase Bat", false))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            match try_attack(3, true, false, 1, 1, ai_entity, player_entity, world) {
                Ok(true) => {}
                Ok(false) => {
                    let attack_twice = {
                        let rng = &mut world.fetch_mut::<RNG>().0;
                        rng.gen_ratio(1, 5)
                    };
                    if attack_twice {
                        let _ = try_attack(2, true, false, 1, 1, ai_entity, player_entity, world);
                    }
                }
                Err(_) => {
                    let _ = try_move_towards(ai_entity, player_entity, world);
                }
            }
        }))
        .with(position)
        .with(attackable)
        .with(Sprite::new("phase_bat"))
        .build()
}

pub fn create_danger_spider(position: Position, world: &mut World) -> Entity {
    let mut attackable = Attackable::new(
        11,
        20,
        items::create_random_layer1(Rarity::Common, world),
        false,
    );
    attackable.lower_spawn_times = (0.5, 3);
    world
        .create_entity()
        .with(Name::new("Danger! Spider", false))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(5, true, false, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(attackable)
        .with(Sprite::new("danger_spider"))
        .build()
}

pub fn create_pungent_ooze(position: Position, world: &mut World) -> Entity {
    let mut attackable = Attackable::new(
        10,
        20,
        items::create_random_layer1(Rarity::Common, world),
        false,
    );
    attackable.is_oozing = true;
    world
        .create_entity()
        .with(Name::new("Pungent Ooze", false))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(4, true, false, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(attackable)
        .with(Sprite::new("pungent_ooze"))
        .build()
}

pub fn create_skeleton_scout(position: Position, world: &mut World) -> Entity {
    let attackable = Attackable::new(
        12,
        30,
        items::create_random_layer1(Rarity::Uncommon, world),
        false,
    );
    world
        .create_entity()
        .with(Name::new("Skeleton Scout", false))
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
                let _ = try_attack(4, false, false, 1, 2, ai_entity, player_entity, world);
            } else {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(attackable)
        .with(Sprite::new("skeleton_scout"))
        .build()
}

pub fn create_volatile_husk(position: Position, world: &mut World) -> Entity {
    let mut attackable = Attackable::new(
        9,
        30,
        items::create_random_layer1(Rarity::Uncommon, world),
        false,
    );
    attackable.explode_on_death = (6, 1);
    world
        .create_entity()
        .with(Name::new("Volatile Husk", false))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(3, true, false, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(attackable)
        .with(Sprite::new("volatile_husk"))
        .build()
}

pub fn create_arcane_ooze(position: Position, world: &mut World) -> Entity {
    let mut attackable = Attackable::new(18, 50, Some(items::create_random_good_scroll), false);
    attackable.is_oozing = true;
    attackable.is_magic_immune = true;
    world
        .create_entity()
        .with(Name::new("Arcane Ooze", false))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(6, true, true, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(attackable)
        .with(Sprite::new("arcane_ooze"))
        .build()
}

pub fn create_soul_spectre(position: Position, world: &mut World) -> Entity {
    let attackable = Attackable::new(
        16,
        50,
        items::create_random_layer1(Rarity::Rare, world),
        false,
    );
    world
        .create_entity()
        .with(Name::new("Soul Spectre", false))
        .with(AI::new(|ai_entity, world| {
            let has_been_attacked = {
                let mut counter_data = world.write_storage::<Counter>();
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let attackable_data = world.read_storage::<Attackable>();
                let ai_counter = counter_data.get_mut(ai_entity).unwrap();
                let ai_position = position_data.get(ai_entity).unwrap();
                let ai_attackable = attackable_data.get(ai_entity).unwrap();
                let player_position = (&player_data, &position_data).join().next().unwrap().1;

                if *ai_counter == Counter(0) && ai_position.distance_from(*player_position) <= 6 {
                    let mut message_log = world.fetch_mut::<MessageLog>();
                    message_log.new_message(
                        "Hello there. May I have your soul?",
                        MessageColor::White,
                        MessageDisplayLength::Medium,
                    );
                    *ai_counter = Counter(1);
                }

                if ai_attackable.current_health != ai_attackable.max_health
                    && ai_position.distance_from(*player_position) <= 6
                {
                    if *ai_counter != Counter(2) {
                        let mut message_log = world.fetch_mut::<MessageLog>();
                        message_log.new_message("Wow, that was rude. All I was asking for was your immortal soul, no need to overreact. Now I'm ANRGY!", MessageColor::Red, MessageDisplayLength::Medium);
                    }
                    *ai_counter = Counter(2);
                }

                *ai_counter == Counter(2)
            };

            if has_been_attacked {
                let spawn_discordant_soul = {
                    let rng = &mut world.fetch_mut::<RNG>().0;
                    rng.gen_ratio(1, 6)
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
                    if try_attack(5, true, false, 1, 1, ai_entity, player_entity, world).is_err() {
                        let _ = try_move_towards(ai_entity, player_entity, world);
                        let _ = try_move_towards(ai_entity, player_entity, world);
                    }
                }
            }
        }))
        .with(Counter(0))
        .with(position)
        .with(attackable)
        .with(Sprite::new("soul_spectre"))
        .build()
}

pub fn create_discordant_soul(position: Position, world: &mut World) -> Entity {
    world
        .create_entity()
        .with(Name::new("Discordant Soul", false))
        .with(AI::new(|ai_entity, world| {
            let player_entity = {
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();
                (&entities, &player_data).join().next().unwrap().0
            };
            if try_attack(3, true, false, 1, 1, ai_entity, player_entity, world).is_err() {
                let _ = try_move_towards(ai_entity, player_entity, world);
            }
        }))
        .with(position)
        .with(Attackable::new(6, 5, None, false))
        .with(Sprite::new("discordant_soul"))
        .build()
}

pub fn create_siro_king_of_hell(position: Position, world: &mut World) -> Entity {
    let attackable = Attackable::new(
        50,
        200,
        items::create_random_layer1(Rarity::Epic, world),
        true,
    );
    world
        .create_entity()
        .with(Name::new("Siro, King of Hell", false))
        .with(AI::new(|ai_entity, world| {
            // TODO
        }))
        .with(position)
        .with(attackable)
        .with(Sprite {
            id: "placeholder",
            double_sized: true,
            in_foreground: true,
        })
        .build()
}

pub fn create_xilphene_the_moth_priestess(position: Position, world: &mut World) -> Entity {
    let attackable = Attackable::new(
        40,
        200,
        items::create_random_layer1(Rarity::Epic, world),
        true,
    );
    world
        .create_entity()
        .with(Name::new("Xilphene, The Moth Priestess", false))
        .with(AI::new(|ai_entity, world| {
            // TODO
        }))
        .with(position)
        .with(attackable)
        .with(Sprite {
            id: "placeholder",
            double_sized: true,
            in_foreground: true,
        })
        .build()
}

pub fn create_ume_the_dungeon_heart(position: Position, world: &mut World) -> Entity {
    let attackable = Attackable::new(
        50,
        200,
        items::create_random_layer1(Rarity::Epic, world),
        true,
    );
    world
        .create_entity()
        .with(Name::new("Ume, The Dungeon Heart", false))
        .with(AI::new(|ai_entity, world| {
            // TODO
        }))
        .with(position)
        .with(attackable)
        .with(Sprite {
            id: "ume_the_dungeon_heart",
            double_sized: true,
            in_foreground: true,
        })
        .build()
}

use crate::attack::{damage, player_get_target, try_attack};
use crate::data::*;
use crate::items::create_random_scroll;
use crate::movement::try_move;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::Triangular;
use specs::{Builder, Entity, Join, World, WorldExt};

pub fn create_random_layer1(
    rarity: Rarity,
    world: &mut World,
) -> Option<fn(Option<Position>, &mut World) -> Entity> {
    let rng = &mut world.fetch_mut::<RNG>().0;
    let should_generate_item = match rarity {
        Rarity::Common => rng.gen_ratio(1, 8),
        Rarity::Uncommon => rng.gen_ratio(1, 4),
        Rarity::Rare | Rarity::Epic => true,
    };
    if should_generate_item {
        let choices: Vec<fn(Option<Position>, &mut World) -> Entity> = match rarity {
            Rarity::Common => vec![
                create_jump_saber,
                create_twister_staff,
                create_edge_of_ebony,
                create_blight_bow,
            ],
            Rarity::Uncommon => vec![create_improvised_spellbook, create_random_scroll],
            Rarity::Rare => vec![create_netherbane],
            Rarity::Epic => vec![],
        };
        return Some(*choices.choose(rng).unwrap());
    }
    None
}

pub fn create_jump_saber(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Name::new("Jump Saber", false))
        .with(Item::new(0, |_, world| {
            let mut attack_succeeded = false;
            if let Some(target_entity) = player_get_target(2, 2, world) {
                let player_entity = {
                    let (player_entity, player_facing_direction) = {
                        let entities = world.entities();
                        let player_data = world.read_storage::<Player>();
                        let (player_entity, player) =
                            (&entities, &player_data).join().next().unwrap();
                        (player_entity, player.facing_direction)
                    };
                    if try_move(player_entity, player_facing_direction, world).is_ok() {
                        Some(player_entity)
                    } else {
                        None
                    }
                };
                if let Some(player_entity) = player_entity {
                    attack_succeeded =
                        try_attack(11, true, 1, 1, player_entity, target_entity, world).is_ok();
                }
            }
            ItemResult {
                should_end_turn: attack_succeeded,
                should_consume_item: false,
            }
        }))
        .with(Sprite::new("jump_saber"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_twister_staff(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Name::new("Twister Staff", false))
        .with(Item::new(10, |_, world| {
            if let Some(target_entity) = player_get_target(1, 2, world) {
                let player_entity = {
                    let entities = world.entities();
                    let player_data = world.read_storage::<Player>();
                    (&entities, &player_data).join().next().unwrap().0
                };
                let attack_result = try_attack(8, false, 1, 2, player_entity, target_entity, world);
                match attack_result {
                    Ok(false) => {
                        let player_facing_direction = {
                            let player_data = world.read_storage::<Player>();
                            player_data.get(player_entity).unwrap().facing_direction
                        };
                        if try_move(target_entity, player_facing_direction, world).is_err() {
                            damage(2, false, Some(player_entity), target_entity, world);
                        }
                    }
                    _ => {}
                }
                ItemResult {
                    should_end_turn: attack_result.is_ok(),
                    should_consume_item: false,
                }
            } else {
                ItemResult {
                    should_end_turn: false,
                    should_consume_item: false,
                }
            }
        }))
        .with(Sprite::new("twister_staff"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_edge_of_ebony(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Name::new("Edge of Ebony", false))
        .with(Item::new(5, |_, world| {
            if let Some(target_entity) = player_get_target(1, 1, world) {
                let player_entity = {
                    let entities = world.entities();
                    let player_data = world.read_storage::<Player>();
                    (&entities, &player_data).join().next().unwrap().0
                };
                let attack_result = try_attack(10, true, 1, 1, player_entity, target_entity, world);
                if attack_result == Ok(false) {
                    let rng = &mut world.fetch_mut::<RNG>().0;
                    if rng.gen_ratio(1, 5) {
                        let mut attackable_data = world.write_storage::<Attackable>();
                        let target_attackable = attackable_data.get_mut(target_entity).unwrap();
                        target_attackable.cant_attack_turns += 2;
                    }
                }
                ItemResult {
                    should_end_turn: attack_result.is_ok(),
                    should_consume_item: false,
                }
            } else {
                ItemResult {
                    should_end_turn: false,
                    should_consume_item: false,
                }
            }
        }))
        .with(Sprite::new("edge_of_ebony"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_blight_bow(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Name::new("Blight Bow", false))
        .with(Item::new(8, |_, world| {
            if let Some(target_entity) = player_get_target(1, 2, world) {
                let player_entity = {
                    let entities = world.entities();
                    let player_data = world.read_storage::<Player>();
                    (&entities, &player_data).join().next().unwrap().0
                };
                let attack_result = try_attack(4, false, 1, 2, player_entity, target_entity, world);
                if attack_result == Ok(false) {
                    let mut attackable_data = world.write_storage::<Attackable>();
                    let target_attackable = attackable_data.get_mut(target_entity).unwrap();
                    target_attackable.blight_stacks += 6;
                }
                ItemResult {
                    should_end_turn: attack_result.is_ok(),
                    should_consume_item: false,
                }
            } else {
                ItemResult {
                    should_end_turn: false,
                    should_consume_item: false,
                }
            }
        }))
        .with(Sprite::new("blight_bow"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_improvised_spellbook(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Name::new("Improvised Spellbook", false))
        .with(Item::new(20, |_, world| {
            if let Some(target_entity) = player_get_target(1, 3, world) {
                let player_entity = {
                    let entities = world.entities();
                    let player_data = world.read_storage::<Player>();
                    (&entities, &player_data).join().next().unwrap().0
                };
                let damage = {
                    let rng = &mut world.fetch_mut::<RNG>().0;
                    rng.sample(Triangular::new(0.0, 15.0, 8.0).unwrap()) as u32
                };
                let attack_result =
                    try_attack(damage, false, 1, 3, player_entity, target_entity, world);
                ItemResult {
                    should_end_turn: attack_result.is_ok(),
                    should_consume_item: false,
                }
            } else {
                ItemResult {
                    should_end_turn: false,
                    should_consume_item: false,
                }
            }
        }))
        .with(Sprite::new("improvised_spellbook"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_netherbane(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Name::new("Netherbane", false))
        .with(Item::new(0, |item_entity, world| {
            if let Some(target_entity) = player_get_target(1, 1, world) {
                let (player_entity, damage) = {
                    let entities = world.entities();
                    let player_data = world.read_storage::<Player>();
                    let counter_data = world.read_storage::<Counter>();
                    (
                        (&entities, &player_data).join().next().unwrap().0,
                        counter_data.get(item_entity).unwrap().0,
                    )
                };
                let attack_result =
                    try_attack(damage, true, 1, 1, player_entity, target_entity, world);
                if attack_result == Ok(true) {
                    let mut counter_data = world.write_storage::<Counter>();
                    let item_damage_counter = counter_data.get_mut(item_entity).unwrap();
                    item_damage_counter.0 += 1;
                }
                ItemResult {
                    should_end_turn: attack_result.is_ok(),
                    should_consume_item: false,
                }
            } else {
                ItemResult {
                    should_end_turn: false,
                    should_consume_item: false,
                }
            }
        }))
        .with(Counter(3))
        .with(Sprite::new("netherbane"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

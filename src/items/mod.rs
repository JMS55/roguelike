mod layer1;

pub use layer1::*;

use crate::attack::{damage, player_get_target, try_attack};
use crate::data::*;
use rand::seq::SliceRandom;
use specs::{Builder, Entities, Entity, Join, ReadStorage, World, WorldExt};
use std::collections::HashSet;

pub fn create_makeshift_dagger(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Name::new("Makeshift Dagger", false))
        .with(Item::new(0, |_, world| {
            let mut attack_succeeded = false;
            if let Some(target_entity) = player_get_target(1, 1, world) {
                let player_entity = {
                    let entities = world.entities();
                    let player_data = world.read_storage::<Player>();
                    (&entities, &player_data).join().next().unwrap().0
                };
                if try_attack(8, true, false, 1, 1, player_entity, target_entity, world).is_ok() {
                    attack_succeeded = true;
                }
            }
            ItemResult {
                should_end_turn: attack_succeeded,
                should_consume_item: false,
            }
        }))
        .with(Sprite::new("makeshift_dagger"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_random_scroll(item_position: Option<Position>, world: &mut World) -> Entity {
    let create_function = {
        let rng = &mut world.fetch_mut::<RNG>().0;
        let choices = [create_random_good_scroll, create_bad_scroll];
        *choices.choose(rng).unwrap()
    };
    (create_function)(item_position, world)
}

pub fn create_random_good_scroll(item_position: Option<Position>, world: &mut World) -> Entity {
    let create_function = {
        let rng = &mut world.fetch_mut::<RNG>().0;
        let choices = [
            create_scroll_of_displacement,
            create_scroll_of_entanglement,
            create_scroll_of_lightning,
        ];
        *choices.choose(rng).unwrap()
    };
    (create_function)(item_position, world)
}

pub fn create_bad_scroll(item_position: Option<Position>, world: &mut World) -> Entity {
    let create_function = {
        let rng = &mut world.fetch_mut::<RNG>().0;
        let choices = [create_scroll_of_shadows];
        *choices.choose(rng).unwrap()
    };
    (create_function)(item_position, world)
}

pub fn create_scroll_of_shadows(item_position: Option<Position>, world: &mut World) -> Entity {
    let sprite = world.fetch::<ScrollInfo>().scroll_of_shadows_sprite;
    let concealed = !world.fetch::<ScrollInfo>().scroll_of_shadows_identified;
    let mut e = world
        .create_entity()
        .with(Name::new("Scroll of Shadows", concealed))
        .with(Item::new(0, |_, world| {
            let mut message_log = world.fetch_mut::<MessageLog>();
            message_log.new_message(
                "You used a Scroll of Shadows!",
                MessageColor::White,
                MessageDisplayLength::Medium,
            );

            world.fetch_mut::<ScrollInfo>().scroll_of_shadows_identified = true;
            let mut player_data = world.write_storage::<Player>();
            let mut name_data = world.write_storage::<Name>();
            let player = (&mut player_data).join().next().unwrap();
            for item_entity in player.inventory.iter().flatten() {
                let item_name = name_data.get_mut(*item_entity).unwrap();
                if item_name.text == "Scroll of Shadows" {
                    item_name.concealed = false;
                }
            }

            let mut spawner_data = world.write_storage::<Spawner>();
            for spawner in (&mut spawner_data).join() {
                spawner.spawn_concealed = true;
            }

            let mut sprite_data = world.write_storage::<Sprite>();
            let attackable_data = world.read_storage::<Attackable>();
            for (sprite, _, _) in (&mut sprite_data, &attackable_data, !&player_data).join() {
                sprite.id = "concealed";
            }

            ItemResult {
                should_end_turn: true,
                should_consume_item: true,
            }
        }))
        .with(Sprite::new(sprite));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_scroll_of_displacement(item_position: Option<Position>, world: &mut World) -> Entity {
    let sprite = world.fetch::<ScrollInfo>().scroll_of_displacement_sprite;
    let concealed = !world
        .fetch::<ScrollInfo>()
        .scroll_of_displacement_identified;
    let mut e = world
        .create_entity()
        .with(Name::new("Scroll of Displacement", concealed))
        .with(Item::new(0, |_, world| {
            let mut message_log = world.fetch_mut::<MessageLog>();
            message_log.new_message(
                "You used a Scroll of Displacement!",
                MessageColor::White,
                MessageDisplayLength::Medium,
            );

            world
                .fetch_mut::<ScrollInfo>()
                .scroll_of_displacement_identified = true;
            let mut player_data = world.write_storage::<Player>();
            let mut name_data = world.write_storage::<Name>();
            let player = (&mut player_data).join().next().unwrap();
            for item_entity in player.inventory.iter().flatten() {
                let item_name = name_data.get_mut(*item_entity).unwrap();
                if item_name.text == "Scroll of Displacement" {
                    item_name.concealed = false;
                }
            }

            let mut new_player_position = None;
            let mut position_data = world.write_storage::<Position>();
            let intangible_data = world.read_storage::<Intangible>();
            let mut rng = world.fetch_mut::<RNG>();
            let mut possible_new_positions = (&position_data, &intangible_data)
                .join()
                .map(|(position, _)| position)
                .collect::<Vec<&Position>>();
            possible_new_positions.shuffle(&mut rng.0);
            for position1 in possible_new_positions {
                if !(&position_data, !&intangible_data)
                    .join()
                    .any(|(position2, _)| position1 == position2)
                {
                    new_player_position = Some(*position1);
                    break;
                }
            }
            if let Some(new_player_position) = new_player_position {
                let player_position = (&player_data, &mut position_data).join().next().unwrap().1;
                *player_position = new_player_position;
            }

            ItemResult {
                should_end_turn: true,
                should_consume_item: true,
            }
        }))
        .with(Sprite::new(sprite));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_scroll_of_entanglement(item_position: Option<Position>, world: &mut World) -> Entity {
    let sprite = world.fetch::<ScrollInfo>().scroll_of_entanglement_sprite;
    let concealed = !world
        .fetch::<ScrollInfo>()
        .scroll_of_entanglement_identified;
    let mut e = world
        .create_entity()
        .with(Name::new("Scroll of Entanglement", concealed))
        .with(Item::new(0, |_, world| {
            let mut message_log = world.fetch_mut::<MessageLog>();
            message_log.new_message(
                "You used a Scroll of Entanglement!",
                MessageColor::White,
                MessageDisplayLength::Medium,
            );

            world
                .fetch_mut::<ScrollInfo>()
                .scroll_of_entanglement_identified = true;
            let mut player_data = world.write_storage::<Player>();
            let mut name_data = world.write_storage::<Name>();
            let player = (&mut player_data).join().next().unwrap();
            for item_entity in player.inventory.iter().flatten() {
                let item_name = name_data.get_mut(*item_entity).unwrap();
                if item_name.text == "Scroll of Entanglement" {
                    item_name.concealed = false;
                }
            }

            let mut attackable_data = world.write_storage::<Attackable>();
            let position_data = world.read_storage::<Position>();
            let player_position = (&player_data, &position_data).join().next().unwrap().1;
            for (entity_attackable, entity_position, _) in
                (&mut attackable_data, &position_data, !&player_data).join()
            {
                if (player_position.x - entity_position.x).abs() <= 5
                    && (player_position.y - entity_position.y).abs() <= 5
                {
                    entity_attackable.cant_attack_turns += 8;
                    entity_attackable.cant_move_turns += 16;
                }
            }

            ItemResult {
                should_end_turn: true,
                should_consume_item: true,
            }
        }))
        .with(Sprite::new(sprite));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

pub fn create_scroll_of_lightning(item_position: Option<Position>, world: &mut World) -> Entity {
    let sprite = world.fetch::<ScrollInfo>().scroll_of_lightning_sprite;
    let concealed = !world.fetch::<ScrollInfo>().scroll_of_lightning_identified;
    let mut e = world
        .create_entity()
        .with(Name::new("Scroll of Lightning", concealed))
        .with(Item::new(0, |_, world| {
            {
                let mut message_log = world.fetch_mut::<MessageLog>();
                message_log.new_message(
                    "You used a Scroll of Lightning!",
                    MessageColor::White,
                    MessageDisplayLength::Medium,
                );

                world
                    .fetch_mut::<ScrollInfo>()
                    .scroll_of_lightning_identified = true;
                let mut player_data = world.write_storage::<Player>();
                let mut name_data = world.write_storage::<Name>();
                let player = (&mut player_data).join().next().unwrap();
                for item_entity in player.inventory.iter().flatten() {
                    let item_name = name_data.get_mut(*item_entity).unwrap();
                    if item_name.text == "Scroll of Lightning" {
                        item_name.concealed = false;
                    }
                }
            }

            fn add_targets(
                entity: Entity,
                seen_entities: &mut HashSet<Entity>,
                player_position: Position,
                entities: &Entities,
                attackable_data: &ReadStorage<Attackable>,
                position_data: &ReadStorage<Position>,
            ) {
                let entity_position = position_data.get(entity).unwrap();
                for (next_entity, next_entity_position, _) in
                    (entities, position_data, attackable_data).join()
                {
                    if (entity_position.x - next_entity_position.x).abs() <= 3
                        && (entity_position.y - next_entity_position.y).abs() <= 3
                        && next_entity_position != &player_position
                        && !seen_entities.contains(&next_entity)
                    {
                        seen_entities.insert(next_entity);
                        add_targets(
                            next_entity,
                            seen_entities,
                            player_position,
                            entities,
                            attackable_data,
                            position_data,
                        );
                    }
                }
            };
            let mut targets;
            {
                let mut seen_entities = HashSet::new();
                let entities = world.entities();
                let attackable_data = world.read_storage::<Attackable>();
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let mut rng = world.fetch_mut::<RNG>();
                let (player_entity, _, player_position) = (&entities, &player_data, &position_data)
                    .join()
                    .next()
                    .unwrap();
                add_targets(
                    player_entity,
                    &mut seen_entities,
                    *player_position,
                    &entities,
                    &attackable_data,
                    &position_data,
                );
                targets = seen_entities.into_iter().collect::<Vec<Entity>>();
                targets.shuffle(&mut rng.0);
            }
            for entity in targets {
                damage(9, false, true, None, entity, world);
            }

            ItemResult {
                should_end_turn: true,
                should_consume_item: true,
            }
        }))
        .with(Sprite::new(sprite));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

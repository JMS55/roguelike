mod layer1;

pub use layer1::*;

use crate::attack::{player_get_target, try_attack};
use crate::data::*;
use rand::seq::SliceRandom;
use specs::{Builder, Entity, Join, World, WorldExt};

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
                if try_attack(8, true, 1, 1, player_entity, target_entity, world).is_ok() {
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
            let mut scroll_info = world.fetch_mut::<ScrollInfo>();
            if !scroll_info.scroll_of_shadows_identified {
                let mut message_log = world.fetch_mut::<MessageLog>();
                message_log.new_message(
                    "You used a Scroll of Shadows!",
                    MessageColor::White,
                    MessageDisplayLength::Medium,
                );
            }

            scroll_info.scroll_of_shadows_identified = true;
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

mod layer1;

pub use layer1::*;

use crate::attack::{player_can_attack, try_attack};
use crate::data::*;
use specs::{Builder, Entity, Join, World, WorldExt};

pub fn create_makeshift_dagger(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Item::new(0, |world| {
            if let Some(target_entity) = player_can_attack(1, 1, world) {
                let player_entity = {
                    let entities = world.entities();
                    let player_data = world.read_storage::<Player>();
                    (&entities, &player_data).join().next().unwrap().0
                };
                try_attack(6, true, 1, 1, player_entity, target_entity, world).map(|_| ())
            } else {
                Err(())
            }
        }))
        .with(Sprite::new("placeholder"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

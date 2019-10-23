mod layer1;

pub use layer1::*;

use crate::data::*;
use crate::try_basic_weapon;
use specs::{Builder, Entity, Join, World, WorldExt};

pub fn create_makeshift_dagger(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Item::new(0, try_basic_weapon!(6, 1, 1)))
        .with(Sprite::new("placeholder"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

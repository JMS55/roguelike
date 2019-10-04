mod class1;

pub use class1::*;

use crate::data::*;
use specs::{Builder, World, WorldExt};

pub fn create_player(world: &mut World) {
    world
        .create_entity()
        .with(Name("Player"))
        .with(Player::new())
        .with(Position::new(0, 0))
        .with(Attackable::new(20))
        .with(Sprite::new("red"))
        .build();
}

pub fn create_staircase(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Staircase"))
        .with(Staircase {})
        .with(Position::new(x, y))
        .with(Sprite::new("pink"))
        .build();
}

pub fn create_wall(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Wall"))
        .with(Position::new(x, y))
        .with(Sprite::new("blue"))
        .build();
}

mod layer1;

pub use layer1::*;

use crate::data::*;
use rand::Rng;
use rand_pcg::Pcg64;
use specs::{Builder, World, WorldExt};

pub fn create_player(world: &mut World) {
    world
        .create_entity()
        .with(Name("Player"))
        .with(Player::new())
        .with(Position::new(0, 0))
        .with(Attackable::new(30, 0, false))
        .with(Sprite::new("player"))
        .build();
}

pub fn create_staircase(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Staircase"))
        .with(Staircase {})
        .with(position)
        .with(Sprite::new("staircase"))
        .build();
}

pub fn create_spawner(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Spawner"))
        .with(Spawner::new())
        .with(position)
        .with(Intangible {})
        .build();
}

pub fn create_wall(position: Position, world: &mut World, rng: &mut Pcg64) {
    world
        .create_entity()
        .with(Name("Wall"))
        .with(position)
        .with(Sprite::new(if rng.gen_ratio(1, 4) {
            "wall_mossy"
        } else {
            "wall"
        }))
        .build();
}

pub fn create_floor(position: Position, world: &mut World) {
    world
        .create_entity()
        .with(Name("Floor"))
        .with(position)
        .with(Intangible {})
        .with(Sprite {
            id: "floor",
            in_foreground: false,
        })
        .build();
}

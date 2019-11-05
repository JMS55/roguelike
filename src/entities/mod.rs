mod layer1;

pub use layer1::*;

use crate::data::*;
use rand::Rng;
use rand_pcg::Pcg64;
use specs::{Builder, Entity, World, WorldExt};

pub fn create_player(world: &mut World) -> Entity {
    let player = Player::new(world);
    world
        .create_entity()
        .with(Name::new("Player", false))
        .with(player)
        .with(Position::new(0, 0))
        .with(Attackable::new(30, 0, None, false))
        .with(Sprite::new("player"))
        .build()
}

pub fn create_staircase(position: Position, world: &mut World) -> Entity {
    world
        .create_entity()
        .with(Name::new("Staircase", false))
        .with(Staircase {})
        .with(position)
        .with(Sprite::new("staircase"))
        .build()
}

pub fn create_spawner(position: Position, world: &mut World) -> Entity {
    world
        .create_entity()
        .with(Name::new("Spawner", false))
        .with(Spawner::new())
        .with(position)
        .with(Intangible {})
        .build()
}

pub fn create_wall(position: Position, world: &mut World, rng: &mut Pcg64) -> Entity {
    world
        .create_entity()
        .with(Name::new("Wall", false))
        .with(position)
        .with(Sprite::new(if rng.gen_ratio(1, 4) {
            "wall_mossy"
        } else {
            "wall"
        }))
        .build()
}

pub fn create_floor(position: Position, world: &mut World) -> Entity {
    world
        .create_entity()
        .with(Name::new("Floor", false))
        .with(position)
        .with(Intangible {})
        .with(Sprite {
            id: "floor",
            double_sized: false,
            in_foreground: false,
        })
        .build()
}

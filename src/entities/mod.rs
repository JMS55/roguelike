mod class1;

pub use class1::*;

use crate::data::*;
use rand::Rng;
use rand_pcg::Pcg64;
use specs::{Builder, Entity, World, WorldExt};

pub fn create_player(world: &mut World) {
    world
        .create_entity()
        .with(Name("Player"))
        .with(Player::new())
        .with(Position::new(0, 0))
        .with(Attackable::new(20))
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

fn replace_with_staircase_on_death(ai_entity: Entity, _: Option<Entity>, world: &mut World) {
    let ai_position = {
        let position_data = world.read_storage::<Position>();
        *position_data.get(ai_entity).unwrap()
    };
    create_staircase(ai_position, world);
}

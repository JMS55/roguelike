use crate::data::{Intangible, Position, Rarity, Spawner, RNG};
use crate::entities::create_random_class1;
use rand::seq::SliceRandom;
use specs::{Join, World, WorldExt};
use std::collections::HashSet;

pub fn tick_spawners(world: &mut World) {
    let mut spawn_positions = Vec::new();
    let mut obstacles;
    {
        obstacles = {
            let position_data = world.read_storage::<Position>();
            let intangible_data = world.read_storage::<Intangible>();
            (&position_data, !&intangible_data)
                .join()
                .map(|(position, _)| (position.x, position.y))
                .collect::<HashSet<(i32, i32)>>()
        };

        let mut spawner_data = world.write_storage::<Spawner>();
        let position_data = world.read_storage::<Position>();
        for (spawner, spawner_position) in (&mut spawner_data, &position_data).join() {
            if spawner.tick() {
                spawn_positions.push(*spawner_position);
            }
        }
    }

    for spawn_position in spawn_positions {
        if !obstacles.contains(&(spawn_position.x, spawn_position.y)) {
            let rarity = {
                let rng = &mut world.fetch_mut::<RNG>().0;
                [
                    (Rarity::Common, 70),
                    (Rarity::Uncommon, 25),
                    (Rarity::Rare, 5),
                ]
                .choose_weighted(rng, |r| r.1)
                .unwrap()
                .0
            };
            obstacles.insert((spawn_position.x, spawn_position.y));
            create_random_class1(rarity, spawn_position.x, spawn_position.y, world);
        }
    }
}

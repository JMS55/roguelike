mod arcane_ooze;
mod danger_spider;
mod mimic;
mod phase_bat;
mod pungent_ooze;
mod pyro_snake;
mod skeleton_scout;
mod soul_spectre;
mod volatile_husk;

use crate::components::*;
use hecs::{Entity, World};
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::Pcg64;

pub fn create_random_enemy(
    position: PositionComponent,
    world: &mut World,
    rng: &mut Pcg64,
) -> Entity {
    let create_function = [
        arcane_ooze::create_arcane_ooze,
        danger_spider::create_danger_spider,
        mimic::create_mimic,
        phase_bat::create_phase_bat,
        pungent_ooze::create_pungent_ooze,
        pyro_snake::create_pyro_snake,
        skeleton_scout::create_skeleton_scout,
        soul_spectre::create_soul_spectre,
        volatile_husk::create_volatile_husk,
    ]
    .choose(rng)
    .unwrap();
    (create_function)(position, world)
}

pub fn create_staircase(position: PositionComponent, world: &mut World) -> Entity {
    world.spawn((
        NameComponent {
            name: "Staircase",
            concealed_name: "???",
            is_concealed: false,
        },
        position,
        SpriteComponent { id: "staircase" },
        StaircaseComponent {},
    ))
}

pub fn create_wall(position: PositionComponent, world: &mut World, rng: &mut Pcg64) -> Entity {
    world.spawn((
        NameComponent {
            name: "Wall",
            concealed_name: "???",
            is_concealed: false,
        },
        position,
        SpriteComponent {
            id: if rng.gen_ratio(1, 4) {
                "wall_mossy"
            } else {
                "wall"
            },
        },
    ))
}

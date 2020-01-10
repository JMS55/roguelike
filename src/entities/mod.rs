mod arcane_ooze;
mod danger_spider;
mod discordant_soul;
mod mimic;
mod phase_bat;
mod pungent_ooze;
mod pyro_snake;
mod skeleton_scout;
mod soul_spectre;
mod volatile_husk;

pub use discordant_soul::create_discordant_soul;
pub use mimic::MimicAI;

use crate::components::*;
use crate::game::Game;
use hecs::{Entity, World};
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::Pcg64;

pub fn create_random_enemy(position: PositionComponent, game: &mut Game) -> Entity {
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
    .choose(&mut game.rng)
    .unwrap();
    (create_function)(position, game)
}

pub fn create_player(ecs: &mut World, rng: &mut Pcg64) -> Entity {
    ecs.spawn((
        NameComponent {
            name: |_, _| "Player",
        },
        PositionComponent { x: 0, y: 0 },
        SpriteComponent {
            id: |_, _| "player",
        },
        CombatComponent::new(
            rng.gen_range(12, 31),
            rng.gen_range(4, 13),
            rng.gen_range(4, 13),
            rng.gen_range(4, 13),
            rng.gen_range(4, 13),
            Team::Player,
        ),
    ))
}

pub fn create_staircase(position: PositionComponent, ecs: &mut World) -> Entity {
    ecs.spawn((
        NameComponent {
            name: |_, _| "Staircase",
        },
        position,
        SpriteComponent {
            id: |_, _| "staircase",
        },
        StaircaseComponent {},
    ))
}

pub fn create_wall(position: PositionComponent, ecs: &mut World, rng: &mut Pcg64) -> Entity {
    ecs.spawn((
        NameComponent {
            name: |_, _| "Wall",
        },
        position,
        SpriteComponent {
            id: if rng.gen_ratio(1, 4) {
                |_, _| "wall_mossy"
            } else {
                |_, _| "wall"
            },
        },
    ))
}

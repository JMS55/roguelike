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

use crate::components::*;
use crate::game::{Game, RNG};
use hecs::{Entity, World};
use rand::seq::SliceRandom;
use rand::Rng;

pub fn create_random_enemy(position: PositionComponent, game: &mut Game) -> Entity {
    let create_function = [
        // arcane_ooze::create_arcane_ooze,
        danger_spider::create_danger_spider,
        // mimic::create_mimic,
        phase_bat::create_phase_bat,
        // pungent_ooze::create_pungent_ooze,
        // pyro_snake::create_pyro_snake,
        // skeleton_scout::create_skeleton_scout,
        // soul_spectre::create_soul_spectre,
        // volatile_husk::create_volatile_husk,
    ]
    .choose(&mut game.rng)
    .unwrap();
    (create_function)(position, game)
}

pub fn create_player(world: &mut World, rng: &mut RNG) -> Entity {
    let max_health = rng.gen_range(12, 31);
    world.spawn((
        NameComponent {
            name: "Player",
            concealed_name: "???",
            is_concealed: false,
        },
        PositionComponent { x: 0, y: 0 },
        SpriteComponent { id: "player" },
        PlayerComponent {
            facing_direction: PositionComponent { x: 0, y: 1 },
            inventory: [None; 16],
            turns_before_passive_healing: 10,
        },
        StatsComponent {
            current_health: max_health,
            max_health,
            strength: rng.gen_range(1, 13),
            luck: rng.gen_range(1, 13),
            agility: rng.gen_range(1, 13),
            focus: rng.gen_range(1, 13),
        },
        TeamComponent::Ally,
    ))
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

pub fn create_wall(position: PositionComponent, world: &mut World, rng: &mut RNG) -> Entity {
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

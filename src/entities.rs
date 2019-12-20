use crate::components::*;
use legion::entity::Entity;
use legion::world::World;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::Pcg64;

pub fn create_walls<'w, T: Iterator<Item = PositionComponent>>(
    positions: T,
    world: &'w mut World,
    rng: &mut Pcg64,
) -> &'w [Entity] {
    world.insert(
        (),
        positions.map(|position| {
            (
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
            )
        }),
    )
}

pub fn create_staircase(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Staircase",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent { id: "staircase" },
            StaircaseComponent {},
        )),
    )[0]
}

pub fn create_random_enemy(
    position: PositionComponent,
    world: &mut World,
    rng: &mut Pcg64,
) -> Entity {
    let create_function = [
        create_phase_bat,
        create_danger_spider,
        create_pungent_ooze,
        create_skeleton_scout,
        create_volatile_husk,
        create_arcane_ooze,
        create_soul_spectre,
        create_mimic,
        create_pyro_snake,
    ]
    .choose(rng)
    .unwrap();
    (create_function)(position, world)
}

pub fn create_phase_bat(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Phase Bat",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent { id: "phase_bat" },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_danger_spider(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Danger Spider",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent {
                id: "danger_spider",
            },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_pungent_ooze(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Pungent Ooze",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent { id: "pungent_ooze" },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_skeleton_scout(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Skeleton Scout",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent {
                id: "skeleton_scout",
            },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_volatile_husk(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Volatile Husk",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent {
                id: "volatile_husk",
            },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_arcane_ooze(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Arcane Ooze",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent { id: "arcane_ooze" },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_soul_spectre(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Soul Spectre",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent { id: "soul_spectre" },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_mimic(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Staircase",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent {
                id: "mimic_disguised",
            },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_pyro_snake(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Pyro Snake",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent { id: "pyro_snake" },
            TeamComponent::AI,
        )),
    )[0]
}

pub fn create_discordant_soul(position: PositionComponent, world: &mut World) -> Entity {
    world.insert(
        (),
        Some((
            NameComponent {
                name: "Discordant Soul",
                concealed_name: "???",
                is_concealed: false,
            },
            position,
            SpriteComponent {
                id: "discordant_soul",
            },
            TeamComponent::AI,
        )),
    )[0]
}

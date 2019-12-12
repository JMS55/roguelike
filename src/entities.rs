use crate::components::*;
use legion::entity::Entity;
use legion::world::World;
use rand::Rng;
use rand_pcg::Pcg64;

pub fn create_walls<'w, T: Iterator<Item = PositionComponent>>(
    positions: T,
    world: &'w mut World,
    rng: &mut Pcg64,
) -> &'w [Entity] {
    let sprite_id = if rng.gen_ratio(1, 4) {
        "wall_mossy"
    } else {
        "wall"
    };
    world.insert(
        (),
        positions.map(|position| {
            (
                NameComponent {
                    name: "Wall",
                    concealed_name: "Wall",
                    is_concealed: false,
                },
                position,
                SpriteComponent { id: sprite_id },
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
                concealed_name: "Staircase",
                is_concealed: false,
            },
            position,
            SpriteComponent { id: "staircase" },
            StaircaseComponent {},
        )),
    )[0]
}

use crate::attack::{player_can_attack, try_attack};
use crate::data::*;
use crate::movement::try_move;
use specs::{Builder, Entity, Join, World, WorldExt};

pub fn create_jump_saber(item_position: Option<Position>, world: &mut World) -> Entity {
    let mut e = world
        .create_entity()
        .with(Item::new(0, |world| {
            if let Some(target_entity) = player_can_attack(2, 2, world) {
                let player_entity = {
                    let (player_entity, player_facing_direction) = {
                        let entities = world.entities();
                        let player_data = world.read_storage::<Player>();
                        let (player_entity, player) =
                            (&entities, &player_data).join().next().unwrap();
                        (player_entity, player.facing_direction)
                    };
                    if try_move(player_entity, player_facing_direction, world).is_ok() {
                        Some(player_entity)
                    } else {
                        None
                    }
                };
                if let Some(player_entity) = player_entity {
                    try_attack(8, 1, 1, player_entity, target_entity, world).map(|_| ())
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        }))
        .with(Sprite::new("placeholder"));
    if let Some(item_position) = item_position {
        e = e.with(item_position);
    }
    e.build()
}

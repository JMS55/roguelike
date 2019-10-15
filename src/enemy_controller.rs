use crate::data::{Player, Position, AI};
use specs::{Entity, Join, World, WorldExt};

pub fn enemy_controller_system(world: &mut World) {
    let player_position = {
        let player_data = world.read_storage::<Player>();
        let position_data = world.read_storage::<Position>();
        (&player_data, &position_data)
            .join()
            .map(|(_, position)| *position)
            .next()
            .unwrap()
    };

    let mut ai_list = {
        let ai_data = world.read_storage::<AI>();
        let position_data = world.read_storage::<Position>();
        let entities = world.entities();
        (&ai_data, &position_data, &entities)
            .join()
            .map(|(ai, position, entity)| (*ai, *position, entity))
            .collect::<Vec<(AI, Position, Entity)>>()
    };
    ai_list.sort_unstable_by(|(_, p1, _), (_, p2, _)| {
        p1.distance_from(*p2)
            .cmp(&p2.distance_from(player_position))
    });

    for (ai, _, ai_entity) in &ai_list {
        (ai.run)(*ai_entity, world);
    }
}

use crate::attack::damage;
use crate::data::*;
use specs::{Entity, Join, World, WorldExt};

pub fn end_of_turn(world: &mut World) {
    let entities_to_process = {
        let entities = world.entities();
        let player_data = world.read_storage::<Player>();
        let attackable_data = world.read_storage::<Attackable>();

        if *world.fetch::<GameState>() == GameState::PlayerTurn {
            vec![(&entities, &player_data).join().next().unwrap().0]
        } else {
            (&entities, &attackable_data)
                .join()
                .map(|(entity, _)| entity)
                .collect::<Vec<Entity>>()
        }
    };

    for entity in entities_to_process {
        let mut apply_blight_damage = false;

        {
            let mut attackable_data = world.write_storage::<Attackable>();
            let entity_attackable = attackable_data.get_mut(entity).unwrap();

            if entity_attackable.cant_attack_turns > 0 {
                entity_attackable.cant_attack_turns -= 1;
            }

            if entity_attackable.cant_move_turns > 0 {
                entity_attackable.cant_move_turns -= 1;
            }

            if entity_attackable.blight_stacks > 0 {
                entity_attackable.blight_stacks -= 1;
                apply_blight_damage = true;
            }
        }

        if apply_blight_damage {
            {
                let mut message_log = world.fetch_mut::<MessageLog>();
                let name_data = world.read_storage::<Name>();
                let entity_name = name_data.get(entity).unwrap();
                message_log.new_message(
                    format!("{} took 1 damage from blight", entity_name.get_text()),
                    MessageColor::Green,
                    MessageDisplayLength::Short,
                );
            }
            damage(1, false, false, None, entity, world);
        }
    }
}

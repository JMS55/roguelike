use crate::data::Attackable;
use specs::{Join, World, WorldExt};

pub fn end_of_turn(world: &mut World) {
    let mut attackable_data = world.write_storage::<Attackable>();
    for attackable in (&mut attackable_data).join() {
        if attackable.cant_attack_turns > 0 {
            attackable.cant_attack_turns -= 1;
        }
    }
}

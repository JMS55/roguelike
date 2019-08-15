use crate::components::{HealthComponent, QueuedAttack};
use specs::{Entities, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage};

pub struct AttackSystem {}

impl AttackSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'s> System<'s> for AttackSystem {
    type SystemData = (
        ReadStorage<'s, QueuedAttack>,
        WriteStorage<'s, HealthComponent>,
        Entities<'s>,
        Read<'s, LazyUpdate>,
    );

    fn run(
        &mut self,
        (queued_attack_data, mut health_data, entities, lazy_update): Self::SystemData,
    ) {
        for (attack_info, attacking_entity) in (&queued_attack_data, &entities).join() {
            let target_entity_health = health_data.get_mut(attack_info.target_entity).unwrap();
            target_entity_health.current_health -= 1;
            if target_entity_health.current_health == 0 {
                entities.delete(attack_info.target_entity).unwrap();
            }
            lazy_update.remove::<QueuedAttack>(attacking_entity);
        }
    }
}

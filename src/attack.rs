use crate::data::{Health, MessageColor, MessageLog, Name};
use specs::{Entity, World, WorldExt};

pub fn attack(damage: u32, attacker: Entity, target: Entity, world: &mut World) {
    let mut health_data = world.write_storage::<Health>();
    let name_data = world.read_storage::<Name>();
    let mut message_log = world.fetch_mut::<MessageLog>();

    let target_health = &mut health_data.get_mut(target).unwrap().current_health;
    *target_health = target_health.checked_sub(damage).unwrap_or(0);

    let attacker_name = name_data.get(attacker).unwrap();
    let target_name = name_data.get(target).unwrap();
    message_log.new_message(
        format!(
            "{} attacked {} for {} damage",
            attacker_name.0, target_name.0, damage,
        ),
        MessageColor::White,
    );
}

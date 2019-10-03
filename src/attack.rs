use crate::data::{
    HealAttackerOnDeath, Health, MessageColor, MessageDisplayLength, MessageLog, Name,
};
use specs::{Entity, World, WorldExt};

pub fn attack(damage: u32, attacker: Entity, target: Entity, world: &mut World) {
    let mut target_is_dead = false;

    {
        let mut health_data = world.write_storage::<Health>();
        let heal_attacker_on_death_data = world.read_storage::<HealAttackerOnDeath>();
        let name_data = world.read_storage::<Name>();
        let mut message_log = world.fetch_mut::<MessageLog>();
        let entities = world.entities();

        if !entities.is_alive(target) {
            return;
        }

        let attacker_name = name_data.get(attacker).unwrap();
        let target_name = name_data.get(target).unwrap();
        message_log.new_message(
            format!(
                "{} attacked {} for {} damage",
                attacker_name.0, target_name.0, damage,
            ),
            MessageColor::White,
            MessageDisplayLength::Short,
        );

        let target_health = &mut health_data.get_mut(target).unwrap().current_health;
        *target_health = target_health.checked_sub(damage).unwrap_or(0);

        if *target_health == 0 {
            target_is_dead = true;

            if let Some(heal_attacker_on_death) = heal_attacker_on_death_data.get(target) {
                if let Some(attacker_health) = health_data.get_mut(attacker) {
                    attacker_health.current_health = match heal_attacker_on_death {
                        HealAttackerOnDeath::Full => attacker_health.max_health,
                        HealAttackerOnDeath::Amount(heal_amount) => {
                            let mut new_health = attacker_health.current_health + heal_amount;
                            if new_health > attacker_health.max_health {
                                new_health = attacker_health.max_health;
                            }
                            new_health
                        }
                        HealAttackerOnDeath::MaxPercentage(percentage) => {
                            let mut new_health = attacker_health.current_health
                                + (attacker_health.max_health as f32 * percentage) as u32;
                            if new_health > attacker_health.max_health {
                                new_health = attacker_health.max_health;
                            }
                            new_health
                        }
                        HealAttackerOnDeath::CurrentPercentage(percentage) => {
                            let mut new_health = attacker_health.current_health
                                + (attacker_health.current_health as f32 * percentage) as u32;
                            if new_health > attacker_health.max_health {
                                new_health = attacker_health.max_health;
                            }
                            new_health
                        }
                    };
                }
            }
        }
    }

    if target_is_dead {
        world.delete_entity(target).unwrap();
    }
}

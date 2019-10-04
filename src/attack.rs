use crate::data::*;
use specs::{Entity, World, WorldExt};

pub fn attack(
    mut damage: Damage,
    range: Range,
    attacker: Entity,
    target: Entity,
    world: &mut World,
) {
    let mut target_is_dead = false;

    {
        let mut attackable_data = world.write_storage::<Attackable>();
        let heal_attacker_on_death_data = world.read_storage::<HealAttackerOnDeath>();
        let name_data = world.read_storage::<Name>();
        let mut message_log = world.fetch_mut::<MessageLog>();
        let entities = world.entities();

        if !entities.is_alive(target) {
            return;
        }

        let attacker_has_oozing_buff = { attackable_data.get(attacker).unwrap().has_oozing_buff };
        let mut target_attackable = attackable_data.get_mut(target).unwrap();
        if attacker_has_oozing_buff {
            damage.0 += target_attackable.oozed_debuff_stacks;
        }
        if range.0 == 1 {
            target_attackable.oozed_debuff_stacks += 1;
        }

        target_attackable.current_health = target_attackable
            .current_health
            .checked_sub(damage.0)
            .unwrap_or(0);

        if target_attackable.current_health == 0 {
            target_is_dead = true;

            if let Some(heal_attacker_on_death) = heal_attacker_on_death_data.get(target) {
                if let Some(attacker_attackable) = attackable_data.get_mut(attacker) {
                    attacker_attackable.current_health = match heal_attacker_on_death {
                        HealAttackerOnDeath::Full => attacker_attackable.max_health,
                        HealAttackerOnDeath::Amount(heal_amount) => {
                            let mut new_health = attacker_attackable.current_health + heal_amount;
                            if new_health > attacker_attackable.max_health {
                                new_health = attacker_attackable.max_health;
                            }
                            new_health
                        }
                        HealAttackerOnDeath::MaxPercentage(percentage) => {
                            let mut new_health = attacker_attackable.current_health
                                + (attacker_attackable.max_health as f32 * percentage) as u32;
                            if new_health > attacker_attackable.max_health {
                                new_health = attacker_attackable.max_health;
                            }
                            new_health
                        }
                        HealAttackerOnDeath::CurrentPercentage(percentage) => {
                            let mut new_health = attacker_attackable.current_health
                                + (attacker_attackable.current_health as f32 * percentage) as u32;
                            if new_health > attacker_attackable.max_health {
                                new_health = attacker_attackable.max_health;
                            }
                            new_health
                        }
                    };
                }
            }
        }

        let attacker_name = name_data.get(attacker).unwrap();
        let target_name = name_data.get(target).unwrap();
        message_log.new_message(
            format!(
                "{} attacked {} for {} damage",
                attacker_name.0, target_name.0, damage.0,
            ),
            MessageColor::White,
            MessageDisplayLength::Short,
        );
    }

    if target_is_dead {
        world.delete_entity(target).unwrap();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Damage(pub u32);

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Range(pub u32);

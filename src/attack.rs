use crate::data::*;
use crate::entities;
use specs::{Entity, Join, World, WorldExt};

/// Returns whether or not the target died and the amount of damage dealt
pub fn damage(
    mut damage: u32,
    is_melee: bool,
    attacker: Option<Entity>,
    target: Entity,
    world: &mut World,
) -> (bool, u32) {
    let target_died = {
        let mut attackable_data = world.write_storage::<Attackable>();

        let target_attackable = attackable_data.get(target).unwrap();
        if target_attackable.current_health == 0 {
            return (true, 0);
        }

        if let Some(attacker) = attacker {
            let attacker_attackable = attackable_data.get(attacker).unwrap().clone();
            let target_attackable = attackable_data.get_mut(target).unwrap();

            if attacker_attackable.is_oozing {
                damage += target_attackable.oozed_stacks;
                if is_melee {
                    target_attackable.oozed_stacks += 1;
                }
            }
        }

        {
            let target_attackable = attackable_data.get(target).unwrap();
            let (lower_spawn_times_health_threshold, lower_spawn_times_by_turns) =
                target_attackable.lower_spawn_times;
            if lower_spawn_times_health_threshold != 0.0 && lower_spawn_times_by_turns != 0 {
                if target_attackable.current_health as f32 / target_attackable.max_health as f32
                    <= lower_spawn_times_health_threshold
                {
                    let mut spawner_data = world.write_storage::<Spawner>();
                    for spawner in (&mut spawner_data).join() {
                        spawner.turns_per_spawn = spawner
                            .turns_per_spawn
                            .checked_sub(lower_spawn_times_by_turns)
                            .unwrap_or(0);
                        if spawner.turns_per_spawn < 10 {
                            spawner.turns_per_spawn = 10;
                        }
                    }
                    let mut message_log = world.fetch_mut::<MessageLog>();
                    message_log.new_message(
                        "The air around you feels more... dangerous...",
                        MessageColor::White,
                        MessageDisplayLength::Short,
                    );
                }
            }
        }

        let target_attackable = attackable_data.get_mut(target).unwrap();
        target_attackable.current_health = target_attackable
            .current_health
            .checked_sub(damage)
            .unwrap_or(0);

        target_attackable.current_health == 0
    };

    if target_died {
        let (blast_damage, blast_radius) = {
            let attackable_data = world.read_storage::<Attackable>();
            let target_attackable = attackable_data.get(target).unwrap();
            target_attackable.explode_on_death
        };
        if blast_damage != 0 && blast_radius != 0 {
            {
                let mut message_log = world.fetch_mut::<MessageLog>();
                let name_data = world.read_storage::<Name>();
                let target_name = name_data.get(target).unwrap().0;
                message_log.new_message(
                    format!("{} exploded!", target_name),
                    MessageColor::White,
                    MessageDisplayLength::Medium,
                );
            }

            let targets = {
                let attackable_data = world.read_storage::<Attackable>();
                let position_data = world.read_storage::<Position>();
                let entities = world.entities();
                let target_position = position_data.get(target).unwrap();
                (&entities, &position_data, &attackable_data)
                    .join()
                    .filter(|(_, position, _)| {
                        position.x - target_position.x <= blast_radius as i16
                            && position.y - target_position.y <= blast_radius as i16
                    })
                    .map(|(entity, _, _)| entity)
                    .collect::<Vec<Entity>>()
            };
            for target in targets {
                self::damage(blast_damage, false, Some(target), target, world);
            }
        }

        let target_is_boss = {
            let attackable_data = world.read_storage::<Attackable>();
            let target_attackable = attackable_data.get(target).unwrap();
            target_attackable.is_boss
        };
        if target_is_boss {
            {
                let mut attackable_data = world.write_storage::<Attackable>();
                let player_data = world.read_storage::<Player>();
                let player_attackable = (&mut attackable_data, &player_data)
                    .join()
                    .next()
                    .unwrap()
                    .0;
                player_attackable.max_health += 5;
                player_attackable.current_health = player_attackable.max_health;
            }
            {
                let target_position = {
                    let position_data = world.read_storage::<Position>();
                    *position_data.get(target).unwrap()
                };
                entities::create_staircase(target_position, world);
            }
        }

        {
            let mut player_data = world.write_storage::<Player>();
            let attackable_data = world.read_storage::<Attackable>();
            let mut player = (&mut player_data).join().next().unwrap();
            let target_attackable = attackable_data.get(target).unwrap();
            player.crystals += target_attackable.crystals_dropped_on_death;
        }

        world.delete_entity(target).unwrap();
    }

    (target_died, damage)
}

/// Ok(bool) means the attack went through and whether or not the target died from it
/// Err(()) means the attacker was unable to attack
pub fn try_attack(
    base_damage: u32,
    minimum_range: u32,
    maximum_range: u32,
    attacker: Entity,
    target: Entity,
    world: &mut World,
) -> Result<bool, ()> {
    if can_attack(minimum_range, maximum_range, attacker, target, world) {
        let (attacker_name, target_name) = {
            let name_data = world.read_storage::<Name>();
            (
                *name_data.get(attacker).unwrap(),
                *name_data.get(target).unwrap(),
            )
        };

        let gap = {
            let position_data = world.read_storage::<Position>();
            let attacker_position = position_data.get(attacker).unwrap();
            let target_position = position_data.get(target).unwrap();
            let mut gap = (attacker_position.x - target_position.x).abs() as u32;
            let y_gap = (attacker_position.y - target_position.y).abs() as u32;
            if y_gap > gap {
                gap = y_gap;
            }
            gap
        };

        let (target_died, damage_dealt) =
            damage(base_damage, gap == 1, Some(attacker), target, world);

        let mut message_log = world.fetch_mut::<MessageLog>();
        message_log.new_message(
            format!(
                "{} attacked {} for {} damage",
                attacker_name.0, target_name.0, damage_dealt,
            ),
            MessageColor::White,
            MessageDisplayLength::Short,
        );

        Ok(target_died)
    } else {
        Err(())
    }
}

pub fn can_attack(
    minimum_range: u32,
    maximum_range: u32,
    attacker: Entity,
    target: Entity,
    world: &World,
) -> bool {
    let position_data = world.read_storage::<Position>();
    let attacker_position = position_data.get(attacker).unwrap();
    let target_position = position_data.get(target).unwrap();

    let x_gap = (attacker_position.x - target_position.x).abs() as u32;
    let y_gap = (attacker_position.y - target_position.y).abs() as u32;
    let is_straight_path = x_gap == 0 || y_gap == 0;
    let gap = x_gap + y_gap;

    is_straight_path && gap >= minimum_range && gap <= maximum_range
}

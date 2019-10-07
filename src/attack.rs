use crate::data::*;
use crate::movement::try_turn;
use specs::{Entity, World, WorldExt};

/// Returns whether or not the target died and the amount of damage dealt
pub fn damage(
    mut damage: u32,
    is_melee: bool,
    attacker: Entity,
    target: Entity,
    world: &mut World,
) -> (bool, u32) {
    let target_died = {
        // Can't modify attacker_attackable here. A clone is needed to statisfy the borrow checker.
        let mut attackable_data = world.write_storage::<Attackable>();
        let attacker_attackable = attackable_data.get(attacker).unwrap().clone();
        let mut target_attackable = attackable_data.get_mut(target).unwrap();

        if attacker_attackable.has_oozing_buff {
            damage += target_attackable.oozed_debuff_stacks;
        }
        if is_melee {
            target_attackable.oozed_debuff_stacks += 1;
        }

        target_attackable.current_health = target_attackable
            .current_health
            .checked_sub(damage)
            .unwrap_or(0);

        target_attackable.current_health == 0
    };

    if target_died {
        let target_on_death = {
            let attackable_data = world.write_storage::<Attackable>();
            attackable_data.get(target).unwrap().on_death
        };
        if let Some(target_on_death) = target_on_death {
            (target_on_death)(target, attacker, world);
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
                name_data.get(attacker).unwrap().clone(),
                name_data.get(target).unwrap().clone(),
            )
        };

        let (direction_to_move, gap) = {
            let position_data = world.read_storage::<Position>();
            let attacker_position = position_data.get(attacker).unwrap();
            let target_position = position_data.get(target).unwrap();

            let mut direction_to_move = Direction::Up;
            let change_in_x = attacker_position.x - target_position.x;
            let change_in_y = attacker_position.y - target_position.y;
            if change_in_x < 0 {
                direction_to_move = Direction::Left;
            }
            if change_in_x > 0 {
                direction_to_move = Direction::Right;
            }
            if change_in_y < 0 {
                direction_to_move = Direction::Down;
            }

            let mut gap = (attacker_position.x - target_position.x).abs() as u32;
            let y_gap = (attacker_position.y - target_position.y).abs() as u32;
            if y_gap > gap {
                gap = y_gap;
            }

            (direction_to_move, gap)
        };

        try_turn(attacker, direction_to_move, world)?;
        let (target_died, damage_dealt) = damage(base_damage, gap == 1, attacker, target, world);

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
    world: &mut World,
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

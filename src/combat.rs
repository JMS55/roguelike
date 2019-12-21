use crate::components::*;
use crate::game::Game;
use legion::entity::Entity;
use legion::filter::filter_fns;
use legion::query::{IntoQuery, Read};
use std::collections::{HashMap, HashSet};

pub fn attack(
    attacker: Entity,
    damage_amount: u16,
    attack_offsets: &'static [PositionComponent],
    game: &mut Game,
) {
    let attacker_position = *game
        .world
        .get_component::<PositionComponent>(attacker)
        .unwrap();
    let attacker_team = *game.world.get_component::<TeamComponent>(attacker).unwrap();
    let target_positions = <(Read<PositionComponent>, Read<TeamComponent>)>::query()
        .filter(filter_fns::component::<StatsComponent>())
        .iter_entities_immutable(&game.world)
        .filter_map(|(entity, (position, team))| {
            if *team != attacker_team {
                Some((*position, entity))
            } else {
                None
            }
        })
        .collect::<HashMap<PositionComponent, Entity>>();

    for (a, b, c, d) in &[(1, 0, 0, 1), (0, -1, 1, 0), (-1, 0, 0, -1), (0, 1, -1, 0)] {
        let mut offset_found_target = false;
        for offset in attack_offsets {
            let attack_position = PositionComponent {
                x: attacker_position.x + a * offset.x + b * offset.y,
                y: attacker_position.y + c * offset.x + d * offset.y,
            };
            if let Some(target) = target_positions.get(&attack_position) {
                damage(*target, damage_amount, game);
                offset_found_target = true;
            }
        }
        if offset_found_target {
            break;
        }
    }
}

pub fn can_attack(
    attacker: Entity,
    attack_offsets: &'static [PositionComponent],
    game: &Game,
) -> bool {
    let attacker_position = *game
        .world
        .get_component::<PositionComponent>(attacker)
        .unwrap();
    let attacker_team = *game.world.get_component::<TeamComponent>(attacker).unwrap();
    let target_positions = <(Read<PositionComponent>, Read<TeamComponent>)>::query()
        .filter(filter_fns::component::<StatsComponent>())
        .iter_immutable(&game.world)
        .filter_map(|(position, team)| {
            if *team != attacker_team {
                Some(*position)
            } else {
                None
            }
        })
        .collect::<HashSet<PositionComponent>>();

    for (a, b, c, d) in &[(1, 0, 0, 1), (0, -1, 1, 0), (-1, 0, 0, -1), (0, 1, -1, 0)] {
        for offset in attack_offsets {
            let attack_position = PositionComponent {
                x: attacker_position.x + a * offset.x + b * offset.y,
                y: attacker_position.y + c * offset.x + d * offset.y,
            };
            if target_positions.contains(&attack_position) {
                return true;
            }
        }
    }
    false
}

// Returns whether the target died
pub fn damage(target: Entity, damage_amount: u16, game: &mut Game) -> bool {
    let did_target_die = {
        let mut target_stats = game
            .world
            .get_component_mut::<StatsComponent>(target)
            .unwrap();
        target_stats.current_health = target_stats.current_health.saturating_sub(damage_amount);
        target_stats.current_health == 0
    };
    if did_target_die {
        game.world.delete(target);
    }
    did_target_die
}

pub fn heal(target: Entity, heal_amount: u16, game: &mut Game) {
    let mut target_stats = game
        .world
        .get_component_mut::<StatsComponent>(target)
        .unwrap();
    target_stats.current_health = target_stats
        .max_health
        .min(target_stats.current_health + heal_amount);
}

use crate::components::*;
use crate::game::{DamageInfo, DamageType, Game};
use hecs::Entity;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

pub fn create_phase_bat(position: PositionComponent, game: &mut Game) -> Entity {
    game.ecs.spawn((
        NameComponent {
            name: |_, _| "Phase Bat",
        },
        position,
        SpriteComponent {
            id: |_, _| "phase_bat",
        },
        AIComponent {
            ai: Box::new(PhaseBatAI {
                chase_target: None,
                patrol_goal: None,
            }),
        },
        CombatComponent::new(
            game.rng.gen_range(10, 17),
            game.rng.gen_range(4, 13),
            game.rng.gen_range(4, 13),
            game.rng.gen_range(4, 13),
            game.rng.gen_range(4, 13),
            Team::Enemy,
        ),
    ))
}

#[derive(Clone)]
struct PhaseBatAI {
    chase_target: Option<Entity>,
    patrol_goal: Option<PositionComponent>,
}

impl AI for PhaseBatAI {
    fn run(&mut self, this_entity: Entity, game: &mut Game) {
        let this_position = *game.ecs.get::<PositionComponent>(this_entity).unwrap();

        // Ensure chase_target is valid
        if let Some(chase_target) = self.chase_target {
            if !Self::is_chase_target_still_valid(this_entity, chase_target, game) {
                self.chase_target = None;
            }
        }
        if self.chase_target == None {
            self.chase_target = Self::find_chase_target(this_position, game);
        }

        // If no patrol goal, or already at it, choose a new one
        if self.patrol_goal == None || Some(this_position) == self.patrol_goal {
            loop {
                let room = game.rooms.choose(&mut game.rng).unwrap();
                self.patrol_goal = Some(PositionComponent {
                    x: game.rng.gen_range(
                        room.center.x - room.x_radius as i32,
                        room.center.x + room.x_radius as i32 + 1,
                    ),
                    y: game.rng.gen_range(
                        room.center.y - room.y_radius as i32,
                        room.center.y + room.y_radius as i32 + 1,
                    ),
                });
                if self.patrol_goal != Some(this_position) {
                    break;
                }
            }
        }

        // Attack, otherwise move
        let attack_result = Self::try_attack(this_entity, self.chase_target, game);
        if !game.ecs.contains(this_entity) {
            return;
        }
        if attack_result {
            // If anything was hit by the attack, increase this entity's agility by 2
            game.ecs
                .get_mut::<CombatComponent>(this_entity)
                .unwrap()
                .increase_base_agility(2);
        } else {
            match self.chase_target {
                Some(chase_target) => {
                    let chase_target_position =
                        *game.ecs.get::<PositionComponent>(chase_target).unwrap();
                    Self::move_towards(
                        this_entity,
                        chase_target_position,
                        |p| {
                            (p.x - chase_target_position.x).abs()
                                + (p.y - chase_target_position.y).abs()
                                == 1
                        },
                        game,
                    );
                }
                None => {
                    let patrol_goal = self.patrol_goal.unwrap();
                    if !Self::move_towards(this_entity, patrol_goal, |p| p == patrol_goal, game) {
                        // If unable to move towards patrol_goal (No path), remove it
                        self.patrol_goal = None;
                    }
                }
            }
        }
    }

    fn clone(&self) -> Box<dyn AI> {
        Box::new(std::clone::Clone::clone(self))
    }
}

impl PhaseBatAI {
    fn is_chase_target_still_valid(this_entity: Entity, chase_target: Entity, game: &Game) -> bool {
        // Remove chase target if it's no longer alive
        if !game.ecs.contains(chase_target) {
            return false;
        }

        // Remove chase target if it's more than 4 tiles away by path length
        let this_position = *game.ecs.get::<PositionComponent>(this_entity).unwrap();
        let chase_target_position = *game.ecs.get::<PositionComponent>(chase_target).unwrap();
        let obstacles = game
            .ecs
            .query::<&PositionComponent>()
            .iter()
            .map(|(_, position)| *position)
            .collect::<HashSet<PositionComponent>>();
        let mut frontier = VecDeque::new();
        let mut previously_visited = HashMap::new();
        frontier.push_back(this_position);
        previously_visited.insert(this_position, 0);

        while let Some(current) = frontier.pop_front() {
            if current == chase_target_position {
                if previously_visited[&current] > 3 {
                    return false;
                }
            }

            for neighbor in &current.neighbors() {
                if !previously_visited.contains_key(&neighbor) && !obstacles.contains(&neighbor) {
                    previously_visited.insert(*neighbor, previously_visited[&current] + 1);
                    frontier.push_back(*neighbor);
                }
            }
        }

        true
    }

    // Try to find an entity not on team Enemy and at most 3 tiles away by path length
    fn find_chase_target(this_position: PositionComponent, game: &Game) -> Option<Entity> {
        let obstacles = game
            .ecs
            .query::<&PositionComponent>()
            .iter()
            .map(|(_, position)| *position)
            .collect::<HashSet<PositionComponent>>();
        let mut targets = HashMap::new();
        for (entity, (position, combat)) in game
            .ecs
            .query::<(&PositionComponent, &CombatComponent)>()
            .iter()
        {
            if combat.team != Team::Enemy {
                for neighbor in &position.neighbors() {
                    targets.insert(*neighbor, entity);
                }
            }
        }
        let mut frontier = VecDeque::new();
        let mut previously_visited = HashMap::new();
        frontier.push_back(this_position);
        previously_visited.insert(this_position, 0);

        while let Some(current) = frontier.pop_front() {
            if previously_visited[&current] > 2 {
                return None;
            }

            if let Some(entity) = targets.get(&current) {
                return Some(*entity);
            }

            for neighbor in &current.neighbors() {
                if !previously_visited.contains_key(&neighbor) && !obstacles.contains(&neighbor) {
                    previously_visited.insert(*neighbor, previously_visited[&current] + 1);
                    frontier.push_back(*neighbor);
                }
            }
        }

        None
    }

    // Returns whether the attack went through or not
    fn try_attack(this_entity: Entity, chase_target: Option<Entity>, game: &mut Game) -> bool {
        let this_position = *game.ecs.get::<PositionComponent>(this_entity).unwrap();
        let this_combat = *game.ecs.get::<CombatComponent>(this_entity).unwrap();

        let mut target = None;
        for offset in &[
            PositionComponent { x: -1, y: 0 },
            PositionComponent { x: 0, y: -1 },
            PositionComponent { x: 1, y: 0 },
            PositionComponent { x: 0, y: 1 },
        ] {
            target = game
                .ecs
                .query::<(&PositionComponent, &CombatComponent)>()
                .iter()
                .find_map(|(entity, (position, combat))| {
                    if *position == this_position + *offset && combat.team != Team::Enemy {
                        Some(entity)
                    } else {
                        None
                    }
                });
            if chase_target.is_some() && target == chase_target {
                break;
            }
        }

        if let Some(target) = target {
            game.damage_entity(DamageInfo {
                target,
                damage_amount: this_combat.get_agility(),
                damage_type: DamageType::Agility,
                variance: true,
            });
            true
        } else {
            false
        }
    }

    fn move_towards<F: Fn(PositionComponent) -> bool>(
        this_entity: Entity,
        a_star_guide: PositionComponent,
        reached_goal: F,
        game: &mut Game,
    ) -> bool {
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        struct FrontierNode {
            position: PositionComponent,
            priority: u32,
        }
        impl Ord for FrontierNode {
            fn cmp(&self, other: &Self) -> Ordering {
                self.priority.cmp(&other.priority).reverse()
            }
        }
        impl PartialOrd for FrontierNode {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let this_position = *game.ecs.get::<PositionComponent>(this_entity).unwrap();
        let obstacles = game
            .ecs
            .query::<&PositionComponent>()
            .iter()
            .map(|(_, position)| *position)
            .collect::<HashSet<PositionComponent>>();
        let mut frontier = BinaryHeap::new();
        let mut cost_so_far = HashMap::new();
        let mut came_from = HashMap::new();
        frontier.push(FrontierNode {
            position: this_position,
            priority: 0,
        });
        cost_so_far.insert(this_position, 0);
        came_from.insert(this_position, this_position);

        let mut goal = None;
        while let Some(current) = frontier.pop() {
            if (reached_goal)(current.position) {
                goal = Some(current.position);
                break;
            }

            for neighbor in &current.position.neighbors() {
                let new_cost = cost_so_far[&current.position] + 1;
                if (!cost_so_far.contains_key(&neighbor) || new_cost < cost_so_far[&neighbor])
                    && !obstacles.contains(&neighbor)
                {
                    cost_so_far.insert(*neighbor, new_cost);
                    frontier.push(FrontierNode {
                        position: *neighbor,
                        priority: new_cost
                            + (a_star_guide.x - current.position.x)
                                .abs()
                                .max((a_star_guide.y - current.position.y).abs())
                                as u32,
                    });
                    came_from.insert(*neighbor, current.position);
                }
            }
        }

        if let Some(goal) = goal {
            let mut current = goal;
            while came_from[&current] != this_position {
                current = came_from[&current];
            }
            *game.ecs.get_mut::<PositionComponent>(this_entity).unwrap() = current;
            true
        } else {
            false
        }
    }
}

use crate::components::*;
use crate::entities;
use crate::game::Game;
use hecs::Entity;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

pub fn create_danger_spider(position: PositionComponent, game: &mut Game) -> Entity {
    let max_health = game.rng.gen_range(10, 17);
    game.world.spawn((
        NameComponent {
            name: "Danger Spider",
            concealed_name: "???",
            is_concealed: false,
        },
        position,
        SpriteComponent {
            id: "danger_spider",
        },
        AIComponent {
            ai: Box::new(DangerSpiderAI {
                chase_target: None,
                patrol_goal: None,
                spawned_help_yet: false,
            }),
        },
        StatsComponent {
            current_health: max_health,
            max_health,
            strength: game.rng.gen_range(4, 8),
            luck: game.rng.gen_range(5, 8),
            agility: game.rng.gen_range(6, 9),
            focus: game.rng.gen_range(1, 12),
        },
        TeamComponent::AI,
    ))
}

#[derive(Clone)]
struct DangerSpiderAI {
    chase_target: Option<Entity>,
    patrol_goal: Option<PositionComponent>,
    spawned_help_yet: bool,
}

impl AI for DangerSpiderAI {
    fn run(&mut self, this_entity: Entity, game: &mut Game) {
        let this_position = *game.world.get::<PositionComponent>(this_entity).unwrap();

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
                    x: game.dungeon_generation_rng.gen_range(
                        room.center.x - room.x_radius as i32,
                        room.center.x + room.x_radius as i32 + 1,
                    ),
                    y: game.dungeon_generation_rng.gen_range(
                        room.center.y - room.y_radius as i32,
                        room.center.y + room.y_radius as i32 + 1,
                    ),
                });
                if self.patrol_goal != Some(this_position) {
                    break;
                }
            }
        }

        // Spawn another enemy if at 30% health or below, there is space for it, and this hasn't been done before. Ends this function early.
        let this_stats = *game.world.get::<StatsComponent>(this_entity).unwrap();
        if (this_stats.current_health as f64 / this_stats.max_health as f64) <= 0.3
            && !self.spawned_help_yet
        {
            let obstacles = game
                .world
                .query::<&PositionComponent>()
                .iter()
                .map(|(_, position)| *position)
                .collect::<HashSet<PositionComponent>>();
            for offset in &[
                PositionComponent { x: 0, y: 1 },
                PositionComponent { x: 1, y: 0 },
                PositionComponent { x: 0, y: -1 },
                PositionComponent { x: -1, y: 0 },
            ] {
                let helper_spawn_position = this_position + *offset;
                if !obstacles.contains(&helper_spawn_position) {
                    entities::create_random_enemy(helper_spawn_position, game);
                    self.spawned_help_yet = true;
                    return;
                }
            }
        }

        // Attack, otherwise move
        if !Self::try_attack(this_entity, self.chase_target, game) {
            match self.chase_target {
                Some(chase_target) => {
                    let chase_target_position =
                        *game.world.get::<PositionComponent>(chase_target).unwrap();
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

impl DangerSpiderAI {
    fn is_chase_target_still_valid(this_entity: Entity, chase_target: Entity, game: &Game) -> bool {
        // Remove chase target if it's no longer alive
        if !game.world.contains(chase_target) {
            return false;
        }

        // Remove chase target if it's more than 4 tiles away by path length
        let this_position = *game.world.get::<PositionComponent>(this_entity).unwrap();
        let chase_target_position = *game.world.get::<PositionComponent>(chase_target).unwrap();
        let obstacles = game
            .world
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
                    dbg!(frontier.len(), previously_visited.len());

                    return false;
                }
            }

            for neighbor in &neighbors(current) {
                if !previously_visited.contains_key(&neighbor) && !obstacles.contains(&neighbor) {
                    previously_visited.insert(*neighbor, previously_visited[&current] + 1);
                    frontier.push_back(*neighbor);
                }
            }
        }

        true
    }

    // Try to find an entity with TeamComponent::Ally and at most 3 tiles away by path length
    fn find_chase_target(this_position: PositionComponent, game: &Game) -> Option<Entity> {
        let obstacles = game
            .world
            .query::<&PositionComponent>()
            .iter()
            .map(|(_, position)| *position)
            .collect::<HashSet<PositionComponent>>();
        let mut targets = HashMap::new();
        for (entity, (position, team)) in game
            .world
            .query::<(&PositionComponent, &TeamComponent)>()
            .iter()
        {
            if team == &TeamComponent::Ally {
                for neighbor in &neighbors(*position) {
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

            for neighbor in &neighbors(current) {
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
        let this_position = *game.world.get::<PositionComponent>(this_entity).unwrap();
        let this_stats = *game.world.get::<StatsComponent>(this_entity).unwrap();

        let mut target = None;
        for offset in &[
            PositionComponent { x: -1, y: 0 },
            PositionComponent { x: 0, y: -1 },
            PositionComponent { x: 1, y: 0 },
            PositionComponent { x: 0, y: 1 },
        ] {
            target = game
                .world
                .query::<(&PositionComponent, &TeamComponent)>()
                .with::<StatsComponent>()
                .iter()
                .find_map(|(entity, (position, team))| {
                    if *position == this_position + *offset && *team == TeamComponent::Ally {
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
            let mut target_stats = game.world.get_mut::<StatsComponent>(target).unwrap();
            let attack_missed = game.rng.gen_bool(target_stats.agility as f64 / 100.0);
            if !attack_missed {
                let minimum_damge = (this_stats.strength as f64 / 2.0
                    + this_stats.focus as f64 / 1.5)
                    .round() as u32;
                let damage = game.rng.gen_range(minimum_damge, minimum_damge + 6);
                target_stats.current_health = target_stats.current_health.saturating_sub(damage);

                if target_stats.current_health == 0 && target != game.player_entity {
                    drop(target_stats);
                    game.world.despawn(target).unwrap();
                }
            }
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

        let this_position = *game.world.get::<PositionComponent>(this_entity).unwrap();
        let obstacles = game
            .world
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
            *game
                .world
                .get_mut::<PositionComponent>(this_entity)
                .unwrap() = current;
            true
        } else {
            false
        }
    }
}

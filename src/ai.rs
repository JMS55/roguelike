use crate::combat::*;
use crate::components::*;
use crate::game::Game;
use crate::movement::*;
use legion::entity::Entity;
use legion::filter::filter_fns;
use legion::query::{IntoQuery, Read};
use rand::seq::SliceRandom;
use rand::Rng;

pub trait AI: Send + Sync + 'static {
    fn run(&mut self, game: &mut Game, this_entity: Entity);
    fn clone(&self) -> Box<dyn AI>;
}

#[derive(Clone)]
pub struct StationaryAI {
    pub attack_offsets: &'static [PositionComponent],
    pub get_damage_amount: fn(Entity, &Game) -> u16, // Entity is this AI's entity
    pub target: Option<Entity>,
}

impl AI for StationaryAI {
    fn run(&mut self, game: &mut Game, this_entity: Entity) {
        let this_position = *game
            .world
            .get_component::<PositionComponent>(this_entity)
            .unwrap();

        if let Some(target) = self.target {
            if !game.world.is_alive(target) {
                self.target = None;
            }
        }

        match self.target {
            Some(target) => {
                if can_attack(this_entity, self.attack_offsets, game) {
                    attack(
                        this_entity,
                        (self.get_damage_amount)(this_entity, game),
                        self.attack_offsets,
                        game,
                    );
                } else {
                    let target_position = *game
                        .world
                        .get_component::<PositionComponent>(target)
                        .unwrap();
                    let x_distance = (this_position.x - target_position.x).abs();
                    let y_distance = (this_position.y - target_position.y).abs();
                    if x_distance >= 8 && y_distance >= 8 {
                        self.target = None;
                    } else {
                        let _ = try_move_towards(this_entity, target_position, game);
                    }
                }
            }

            None => {
                self.target = <(Read<PositionComponent>, Read<TeamComponent>)>::query()
                    .filter(filter_fns::component::<StatsComponent>())
                    .iter_entities_immutable(&game.world)
                    .find_map(|(entity, (position, team))| {
                        let x_distance = (position.x - this_position.x).abs();
                        let y_distance = (position.y - this_position.y).abs();
                        if x_distance <= 4 && y_distance <= 4 && *team == TeamComponent::Ally {
                            Some(entity)
                        } else {
                            None
                        }
                    });
            }
        }
    }

    fn clone(&self) -> Box<dyn AI> {
        Box::new(std::clone::Clone::clone(self))
    }
}

#[derive(Clone)]
pub struct PatrollingAI {
    pub attack_offsets: &'static [PositionComponent],
    pub get_damage_amount: fn(Entity, &Game) -> u16, // Entity is this AI's entity
    pub target: Option<Entity>,
    pub patrol_goal: Option<PositionComponent>,
}

impl AI for PatrollingAI {
    fn run(&mut self, game: &mut Game, this_entity: Entity) {
        let this_position = *game
            .world
            .get_component::<PositionComponent>(this_entity)
            .unwrap();

        if let Some(target) = self.target {
            if !game.world.is_alive(target) {
                self.target = None;
            }
        }

        if self.patrol_goal == None {
            let room = game.rooms.choose(&mut game.rng).unwrap();
            self.patrol_goal = Some(PositionComponent {
                x: game.dungeon_generation_rng.gen_range(
                    room.center.x - room.x_radius as i16,
                    room.center.x + room.x_radius as i16 + 1,
                ),
                y: game.dungeon_generation_rng.gen_range(
                    room.center.y - room.y_radius as i16,
                    room.center.y + room.y_radius as i16 + 1,
                ),
            });
        }

        match self.target {
            Some(target) => {
                if can_attack(this_entity, self.attack_offsets, game) {
                    attack(
                        this_entity,
                        (self.get_damage_amount)(this_entity, game),
                        self.attack_offsets,
                        game,
                    );
                } else {
                    let target_position = *game
                        .world
                        .get_component::<PositionComponent>(target)
                        .unwrap();
                    let x_distance = (this_position.x - target_position.x).abs();
                    let y_distance = (this_position.y - target_position.y).abs();
                    if x_distance >= 8 && y_distance >= 8 {
                        self.target = None;
                    } else {
                        let _ = try_move_towards(this_entity, target_position, game);
                    }
                }
            }

            None => {
                self.target = <(Read<PositionComponent>, Read<TeamComponent>)>::query()
                    .filter(filter_fns::component::<StatsComponent>())
                    .iter_entities_immutable(&game.world)
                    .find_map(|(entity, (position, team))| {
                        let x_distance = (position.x - this_position.x).abs();
                        let y_distance = (position.y - this_position.y).abs();
                        if x_distance <= 4 && y_distance <= 4 && *team == TeamComponent::Ally {
                            Some(entity)
                        } else {
                            None
                        }
                    });
                if self.target == None {
                    let move_result =
                        try_move_towards(this_entity, self.patrol_goal.unwrap(), game);
                    if move_result.is_err() {
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

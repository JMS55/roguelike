use crate::components::*;
use crate::generate_dungeon::generate_dungeon;
use legion::entity::Entity;
use legion::world::World;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::time::Instant;

pub struct Game {
    pub world: World,
    pub player_entity: Entity,
    pub floor_number: u32,
    pub rng: Pcg64,
    pub dungeon_generation_rng: Pcg64,
    pub message_log: Vec<Message>,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut rng = Pcg64::from_entropy();
        let floor_number = 0;
        let dungeon_generation_rng = Pcg64::from_entropy();
        let message_log = Vec::with_capacity(100);

        let max_health = rng.gen_range(12, 31);
        let player_entity = world.insert(
            (),
            Some((
                NameComponent {
                    name: "Player",
                    concealed_name: "Player",
                    is_concealed: false,
                },
                PositionComponent { x: 0, y: 0 },
                SpriteComponent {
                    id: "player",
                    in_foreground: true,
                },
                PlayerComponent {
                    facing_direction: (true, true),
                    inventory: [None; 16],
                    turns_before_passive_healing: 10,
                },
                CombatComponent {
                    current_health: max_health,
                    max_health,
                    strength: rng.gen_range(1, 13),
                    luck: rng.gen_range(1, 13),
                    agility: rng.gen_range(1, 13),
                    focus: rng.gen_range(1, 13),
                },
                TeamComponent::Ally,
            )),
        )[0];

        let mut this = Self {
            world,
            player_entity,
            floor_number,
            rng,
            dungeon_generation_rng,
            message_log,
        };

        generate_dungeon(&mut this);

        this
    }

    pub fn recent_messages(&self) -> Vec<Message> {
        unimplemented!("TODO: Start at end of list and go backwards until a message that is too old is reached");
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Message {
    pub text: String,
    pub color: MessageColor,
    pub time_created: Instant,
}

impl Message {
    pub fn new(text: String, color: MessageColor) -> Self {
        Self {
            text: text,
            color: color,
            time_created: Instant::now(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MessageColor {
    White,
}

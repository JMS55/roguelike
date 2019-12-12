use crate::game::Game;
use legion::entity::Entity;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NameComponent {
    pub name: &'static str,
    pub concealed_name: &'static str,
    pub is_concealed: bool,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PositionComponent {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct IntangibleComponent {}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SpriteComponent {
    pub id: &'static str,
    pub in_foreground: bool,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CombatComponent {
    pub current_health: u16,
    pub max_health: u16,
    pub strength: u16,
    pub luck: u16,
    pub agility: u16,
    pub focus: u16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PlayerComponent {
    pub facing_direction: (bool, bool),
    pub inventory: [Option<Entity>; 16],
    pub turns_before_passive_healing: u16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct StaircaseComponent {}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum TeamComponent {
    Ally,
    AI,
}

pub struct AIComponent {
    pub ai: Box<dyn AI>,
}

pub trait AI {
    fn run(&mut self, game: &mut Game, this_entity: Entity);
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct StationaryAI {
    pub target: Option<Entity>,
}

impl AI for StationaryAI {
    fn run(&mut self, game: &mut Game, this_entity: Entity) {}
}

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub struct PatrollingAI {
    pub target: Option<Entity>,
    pub path: Vec<PositionComponent>,
}

impl AI for PatrollingAI {
    fn run(&mut self, game: &mut Game, this_entity: Entity) {}
}

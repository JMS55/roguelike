use crate::game::Game;
use crate::movement::Direction;
use hecs::Entity;

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
pub struct SpriteComponent {
    pub id: &'static str,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct StatsComponent {
    pub current_health: u16,
    pub max_health: u16,
    pub strength: u16,
    pub luck: u16,
    pub agility: u16,
    pub focus: u16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum TeamComponent {
    Ally,
    AI,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PlayerComponent {
    pub facing_direction: Direction,
    pub inventory: [Option<Entity>; 16],
    pub turns_before_passive_healing: u16,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct StaircaseComponent {}

pub struct AIComponent {
    pub ai: Box<dyn AI>,
}

pub trait AI: Send + Sync + 'static {
    fn run(&mut self, this_entity: Entity, game: &mut Game);
    fn clone(&self) -> Box<dyn AI>;
}

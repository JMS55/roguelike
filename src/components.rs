use crate::game::Game;
use hecs::Entity;
use std::ops::Add;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NameComponent {
    pub name: &'static str,
    pub concealed_name: &'static str,
    pub is_concealed: bool,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PositionComponent {
    pub x: i32,
    pub y: i32,
}

impl PositionComponent {
    pub fn neighbors(&self) -> [PositionComponent; 8] {
        [
            *self + PositionComponent { x: -1, y: 1 },
            *self + PositionComponent { x: 0, y: 1 },
            *self + PositionComponent { x: 1, y: 1 },
            *self + PositionComponent { x: 1, y: 0 },
            *self + PositionComponent { x: 1, y: -1 },
            *self + PositionComponent { x: 0, y: -1 },
            *self + PositionComponent { x: -1, y: -1 },
            *self + PositionComponent { x: -1, y: 0 },
        ]
    }
}

impl Add for PositionComponent {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SpriteComponent {
    pub id: &'static str,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct StatsComponent {
    pub current_health: u32,
    pub max_health: u32,
    pub strength: u32,
    pub luck: u32,
    pub agility: u32,
    pub focus: u32,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum TeamComponent {
    Ally,
    AI,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PlayerComponent {
    pub facing_direction: PositionComponent,
    pub inventory: [Option<Entity>; 16],
    pub turns_before_passive_healing: u32,
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BurnComponent {
    pub damage_per_turn: u32,
    pub turns_left: u32,
}

use crate::game::Game;
use hecs::Entity;
use std::ops::Add;

#[derive(Copy, Clone)]
pub struct NameComponent {
    pub name: fn(Entity, &Game) -> String,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct PositionComponent {
    pub x: i32,
    pub y: i32,
}

impl PositionComponent {
    pub fn neighbors(&self) -> [Self; 8] {
        [
            *self + Self { x: -1, y: 1 },
            *self + Self { x: 0, y: 1 },
            *self + Self { x: 1, y: 1 },
            *self + Self { x: 1, y: 0 },
            *self + Self { x: 1, y: -1 },
            *self + Self { x: 0, y: -1 },
            *self + Self { x: -1, y: -1 },
            *self + Self { x: -1, y: 0 },
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
pub struct CombatComponent {
    pub current_health: u32,
    pub max_health: u32,
    strength: u32,
    focus: u32,
    agility: u32,
    luck: u32,
    pub team: Team,

    pub strength_buff: (u32, u32), // Amount, Turns left
    pub focus_buff: (u32, u32),    // Amount, Turns left
    pub agility_buff: (u32, u32),  // Amount, Turns left
    pub luck_buff: (u32, u32),     // Amount, Turns left
    pub magic_immune_buff: bool,

    pub strength_debuff: (u32, u32), // Amount, Turns left
    pub focus_debuff: (u32, u32),    // Amount, Turns left
    pub agility_debuff: (u32, u32),  // Amount, Turns left
    pub luck_debuff: (u32, u32),     // Amount, Turns left
    pub burn_debuff: (u32, u32),     // Damage per turn, Turns left
}

impl CombatComponent {
    pub fn new(
        max_health: u32,
        strength: u32,
        focus: u32,
        agility: u32,
        luck: u32,
        team: Team,
    ) -> Self {
        Self {
            current_health: max_health,
            max_health,
            strength,
            focus,
            agility,
            luck,
            team,

            strength_buff: (0, 0),
            focus_buff: (0, 0),
            agility_buff: (0, 0),
            luck_buff: (0, 0),
            magic_immune_buff: false,

            strength_debuff: (0, 0),
            focus_debuff: (0, 0),
            agility_debuff: (0, 0),
            luck_debuff: (0, 0),
            burn_debuff: (0, 0),
        }
    }

    pub fn get_strength(&self) -> u32 {
        (self.strength + self.strength_buff.0)
            .checked_sub(self.strength_debuff.0)
            .unwrap_or(1)
            .min(100)
    }

    pub fn get_focus(&self) -> u32 {
        (self.focus + self.focus_buff.0)
            .checked_sub(self.focus_debuff.0)
            .unwrap_or(1)
            .min(100)
    }

    pub fn get_agility(&self) -> u32 {
        (self.agility + self.agility_buff.0)
            .checked_sub(self.agility_debuff.0)
            .unwrap_or(1)
            .min(100)
    }

    pub fn get_luck(&self) -> u32 {
        (self.luck + self.luck_buff.0)
            .checked_sub(self.luck_debuff.0)
            .unwrap_or(1)
            .min(100)
    }

    pub fn increase_base_strength(&mut self, amount: u32) {
        self.strength = 100.min(self.strength + amount);
    }

    pub fn increase_base_focus(&mut self, amount: u32) {
        self.focus = 100.min(self.focus + amount);
    }

    pub fn increase_base_agility(&mut self, amount: u32) {
        self.agility = 100.min(self.agility + amount);
    }

    pub fn increase_base_luck(&mut self, amount: u32) {
        self.luck = 100.min(self.luck + amount);
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
    Neutral,
}

pub struct AIComponent {
    pub ai: Box<dyn AI>,
}

pub trait AI: Send + Sync + 'static {
    fn run(&mut self, this_entity: Entity, game: &mut Game);
    fn clone(&self) -> Box<dyn AI>;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct StaircaseComponent {}

use specs::storage::BTreeStorage;
use specs::{Component, Entity};
use specs_derive::Component;

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct PlayerComponent {
    pub facing_direction: Direction,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct PositionComponent {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct SpriteComponent {
    pub id: &'static str,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct HealthComponent {
    pub current_health: u32,
    pub max_health: u32,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct QueuedAttack {
    pub target_entity: Entity,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

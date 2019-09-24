use specs::storage::BTreeStorage;
use specs::{Component, Entity};
use specs_derive::Component;

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct PlayerComponent {
    pub crystals: u32,
    pub turns_taken: u32,
}

impl PlayerComponent {
    pub fn new(crystals: u32) -> Self {
        Self {
            crystals,
            turns_taken: 0,
        }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct PositionComponent {
    pub x: i32,
    pub y: i32,
    pub facing_direction: Direction,
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
pub struct StaircaseComponent {}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct AIAttackPlayerComponent {}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct QueuedAttack {
    pub target_entity: Entity,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
#[storage(BTreeStorage)]
pub struct QueuedMovement {
    pub goal_x: i32,
    pub goal_y: i32,
    pub movement_type: MovementType,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum MovementType {
    StandOn,
    StandNextTo,
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct IsPlayerTurn(pub bool);

#[derive(Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct ShouldAdvanceFloor(pub bool);

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

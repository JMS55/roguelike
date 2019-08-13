use specs::storage::BTreeStorage;
use specs::Component;
use specs_derive::Component;

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct PlayerComponent {}

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

use crate::components::*;
use crate::game::Game;
use hecs::{Entity, World};

pub fn create_arcane_ooze(position: PositionComponent, world: &mut World) -> Entity {
    todo!();
}

#[derive(Clone)]
struct ArcaneOozeAI {}

impl AI for ArcaneOozeAI {
    fn run(&mut self, this_entity: Entity, game: &mut Game) {
        todo!();
    }

    fn clone(&self) -> Box<dyn AI> {
        Box::new(std::clone::Clone::clone(self))
    }
}

use crate::components::*;
use crate::game::Game;
use hecs::Entity;

pub fn create_mimic(position: PositionComponent, game: &mut Game) -> Entity {
    todo!();
}

#[derive(Clone)]
struct MimicAI {}

impl AI for MimicAI {
    fn run(&mut self, this_entity: Entity, game: &mut Game) {
        todo!();
    }

    fn clone(&self) -> Box<dyn AI> {
        Box::new(std::clone::Clone::clone(self))
    }
}

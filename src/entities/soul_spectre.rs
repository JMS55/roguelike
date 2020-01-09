use crate::components::*;
use crate::game::Game;
use hecs::Entity;

pub fn create_soul_spectre(position: PositionComponent, game: &mut Game) -> Entity {
    todo!();
}

#[derive(Clone)]
struct SoulSpectreAI {}

impl AI for SoulSpectreAI {
    fn run(&mut self, this_entity: Entity, game: &mut Game) {
        todo!();
    }

    fn clone(&self) -> Box<dyn AI> {
        Box::new(std::clone::Clone::clone(self))
    }
}

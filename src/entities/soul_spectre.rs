use crate::components::*;
use crate::game::Game;
use hecs::{Entity, World};

pub fn create_soul_spectre(position: PositionComponent, world: &mut World) -> Entity {
    todo!();
}

fn create_discordant_soul(position: PositionComponent, world: &mut World) -> Entity {
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

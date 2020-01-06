use crate::components::*;
use crate::game::Game;
use hecs::{Entity, World};

fn create_discordant_soul(position: PositionComponent, world: &mut World) -> Entity {
    todo!();
}

#[derive(Clone)]
struct DiscordantSoulAI {}

impl AI for DiscordantSoulAI {
    fn run(&mut self, this_entity: Entity, game: &mut Game) {
        todo!();
    }

    fn clone(&self) -> Box<dyn AI> {
        Box::new(std::clone::Clone::clone(self))
    }
}

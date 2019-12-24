use crate::components::*;
use crate::game::Game;
use hecs::{Entity, World};

pub fn create_danger_spider(position: PositionComponent, world: &mut World) -> Entity {
    todo!();
}

#[derive(Clone)]
struct DangerSpiderAI {}

impl AI for DangerSpiderAI {
    fn run(&mut self, this_entity: Entity, game: &mut Game) {
        todo!();
    }

    fn clone(&self) -> Box<dyn AI> {
        Box::new(std::clone::Clone::clone(self))
    }
}

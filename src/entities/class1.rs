use crate::attack::attack;
use crate::data::*;
use crate::movement::*;
use rand::Rng;
use specs::{Builder, Join, World, WorldExt};

pub fn create_phase_bat(x: i32, y: i32, world: &mut World) {
    world
        .create_entity()
        .with(Name("Phase Bat"))
        .with(AI::new(|ai_entity, world| {
            let (ai_position, player_entity, player_position) = {
                let position_data = world.read_storage::<Position>();
                let player_data = world.read_storage::<Player>();
                let entities = world.entities();

                let ai_position = position_data.get(ai_entity).unwrap();
                let (player_entity, _, player_position) = (&entities, &player_data, &position_data)
                    .join()
                    .next()
                    .unwrap();

                (*ai_position, player_entity, *player_position)
            };

            if ai_position.is_next_to(&player_position) {
                attack(3, ai_entity, player_entity, world);
                if {
                    let mut rng_data = world.write_storage::<RNG>();
                    let ai_rng = rng_data.get_mut(ai_entity).unwrap();
                    ai_rng.0.gen_ratio(1, 10)
                } {
                    attack(2, ai_entity, player_entity, world);
                }
            } else {
                let path = pathfind(
                    ai_position.x,
                    ai_position.y,
                    player_position.x,
                    player_position.y,
                    world,
                );
                if let Some((new_x, new_y)) = path.get(0) {
                    let direction = match (new_x - ai_position.x, new_y - ai_position.y) {
                        (0, 1) => Direction::Up,
                        (0, -1) => Direction::Down,
                        (-1, 0) => Direction::Left,
                        (1, 0) => Direction::Right,
                        _ => unreachable!(),
                    };
                    let _ = try_move(ai_entity, direction, world);
                }
            }
        }))
        .with(Position::new(x, y))
        .with(Health::new(6))
        .with(Sprite::new("green"))
        .with(RNG::new())
        .build();
}

impl Position {
    fn is_next_to(&self, other: &Self) -> bool {
        match (self.x - other.x, self.y - other.y) {
            (1, 0) => true,
            (-1, 0) => true,
            (0, 1) => true,
            (0, -1) => true,
            _ => false,
        }
    }
}

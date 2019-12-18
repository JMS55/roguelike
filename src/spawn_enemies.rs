use crate::components::PositionComponent;
use crate::entities;
use crate::game::Game;
use legion::query::{IntoQuery, Read};
use rand::Rng;

pub fn spawn_enemies(game: &mut Game) {
    for i in 1..game.dungeon_generation_rng.gen_range(7, 10) {
        // Choose random room and position
        if let Some(enemy_room) = game.rooms.get(i) {
            for _ in 0..30 {
                let enemy_position = PositionComponent {
                    x: game.dungeon_generation_rng.gen_range(
                        enemy_room.center.x - enemy_room.x_radius as i16,
                        enemy_room.center.x + enemy_room.x_radius as i16 + 1,
                    ),
                    y: game.dungeon_generation_rng.gen_range(
                        enemy_room.center.y - enemy_room.y_radius as i16,
                        enemy_room.center.y + enemy_room.y_radius as i16 + 1,
                    ),
                };

                // Try to place an enemy there unless the space is not empty
                if !Read::<PositionComponent>::query()
                    .iter_immutable(&game.world)
                    .any(|position| enemy_position == *position)
                {
                    entities::create_random_enemy(
                        enemy_position,
                        &mut game.world,
                        &mut game.dungeon_generation_rng,
                    );
                    break;
                }
            }
        }
    }
}

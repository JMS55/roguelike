use crate::components::PositionComponent;
use crate::entities;
use crate::game::Game;
use rand::Rng;
use std::collections::HashSet;

pub fn spawn_enemies(game: &mut Game) {
    let mut obstacles = game
        .world
        .query::<&PositionComponent>()
        .iter()
        .map(|(_, position)| *position)
        .collect::<HashSet<PositionComponent>>();

    for i in 1..game.dungeon_generation_rng.gen_range(7, 10) {
        // Choose a random position within a random room
        if let Some(enemy_room) = game.rooms.get(i) {
            for _ in 0..30 {
                let enemy_position = PositionComponent {
                    x: game.dungeon_generation_rng.gen_range(
                        enemy_room.center.x - enemy_room.x_radius as i32,
                        enemy_room.center.x + enemy_room.x_radius as i32 + 1,
                    ),
                    y: game.dungeon_generation_rng.gen_range(
                        enemy_room.center.y - enemy_room.y_radius as i32,
                        enemy_room.center.y + enemy_room.y_radius as i32 + 1,
                    ),
                };

                // Place an enemy there if the space is unoccupied
                if !obstacles.contains(&enemy_position) {
                    entities::create_random_enemy(enemy_position, game);
                    obstacles.insert(enemy_position);
                    break;
                }
            }
        }
    }
}

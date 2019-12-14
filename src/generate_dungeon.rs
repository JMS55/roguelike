use crate::components::PositionComponent;
use crate::entities;
use crate::game::{Game, Message, MessageColor};
use rand::Rng;
use std::collections::HashSet;

pub fn generate_dungeon(game: &mut Game) {
    game.rooms.clear();
    game.floor_positions.clear();

    // Place rooms
    let starting_room = Room {
        center: PositionComponent { x: 0, y: 0 },
        x_radius: 3,
        y_radius: 3,
    };
    game.rooms.push(starting_room);
    'room_placing_loop: for _ in 0..200 {
        let room = Room {
            center: PositionComponent {
                x: game.dungeon_generation_rng.gen_range(-30, 31),
                y: game.dungeon_generation_rng.gen_range(-30, 31),
            },
            x_radius: game.dungeon_generation_rng.gen_range(2, 8),
            y_radius: game.dungeon_generation_rng.gen_range(2, 8),
        };
        for other_room in &game.rooms {
            let required_gap = game.dungeon_generation_rng.gen_range(3, 10);
            let x_gap = (room.center.x - other_room.center.x).abs()
                - room.x_radius as i16
                - other_room.x_radius as i16
                - 3;
            let y_gap = (room.center.y - other_room.center.y).abs()
                - room.y_radius as i16
                - other_room.y_radius as i16
                - 3;
            let actual_gap = x_gap.max(y_gap);
            if actual_gap < required_gap && actual_gap != -1 {
                continue 'room_placing_loop;
            }
        }
        game.rooms.push(room);
    }

    // Place corridors
    for (start_room_index, start_room) in game.rooms.iter().enumerate() {
        let mut end_room_index = game.dungeon_generation_rng.gen_range(0, game.rooms.len());
        while end_room_index == start_room_index {
            end_room_index = game.dungeon_generation_rng.gen_range(0, game.rooms.len());
        }
        let end_room = &game.rooms[end_room_index];
        let start_x = game.dungeon_generation_rng.gen_range(
            start_room.center.x - start_room.x_radius as i16,
            start_room.center.x + start_room.x_radius as i16 + 1,
        );
        let start_y = game.dungeon_generation_rng.gen_range(
            start_room.center.y - start_room.y_radius as i16,
            start_room.center.y + start_room.y_radius as i16 + 1,
        );
        let end_x = game.dungeon_generation_rng.gen_range(
            end_room.center.x - end_room.x_radius as i16,
            end_room.center.x + end_room.x_radius as i16 + 1,
        );
        let end_y = game.dungeon_generation_rng.gen_range(
            end_room.center.y - end_room.y_radius as i16,
            end_room.center.y + end_room.y_radius as i16 + 1,
        );
        for x in start_x.min(end_x)..start_x.max(end_x) {
            game.floor_positions
                .insert(PositionComponent { x, y: start_y });
        }
        for y in start_y.min(end_y)..=start_y.max(end_y) {
            game.floor_positions
                .insert(PositionComponent { x: end_x, y });
        }
    }

    // Get list of all wall positions
    let mut wall_positions = HashSet::with_capacity(1600);
    for room in &game.rooms {
        let x_radius = room.x_radius as i16;
        let y_radius = room.y_radius as i16;
        for x in -(x_radius + 1)..=(x_radius + 1) {
            wall_positions.insert(PositionComponent {
                x: room.center.x + x,
                y: room.center.y + y_radius + 1,
            });
            wall_positions.insert(PositionComponent {
                x: room.center.x + x,
                y: room.center.y - y_radius - 1,
            });
        }
        for y in -y_radius..=y_radius {
            wall_positions.insert(PositionComponent {
                x: room.center.x + x_radius + 1,
                y: room.center.y + y,
            });
            wall_positions.insert(PositionComponent {
                x: room.center.x - x_radius - 1,
                y: room.center.y + y,
            });
        }
    }
    for corridor_position in &game.floor_positions {
        'neighbor_loop: for (x, y) in &corridor_position.get_neighbors() {
            for room in &game.rooms {
                let x_radius = room.x_radius as i16;
                let y_radius = room.y_radius as i16;
                let x_range = (room.center.x - x_radius - 1)..=(room.center.x + x_radius + 1);
                let y_range = (room.center.y - y_radius - 1)..=(room.center.y + y_radius + 1);
                if x_range.contains(x) && y_range.contains(y) {
                    continue 'neighbor_loop;
                }
            }
            wall_positions.insert(PositionComponent { x: *x, y: *y });
        }
    }

    // Update list of floor tiles with floor tiles from rooms
    for room in &game.rooms {
        let x_radius = room.x_radius as i16;
        let y_radius = room.y_radius as i16;
        for x in -x_radius..=x_radius {
            for y in -y_radius..=y_radius {
                game.floor_positions.insert(PositionComponent {
                    x: room.center.x + x,
                    y: room.center.y + y,
                });
            }
        }
    }

    // Create wall entities
    entities::create_walls(
        wall_positions.difference(&game.floor_positions).cloned(),
        &mut game.world,
        &mut game.dungeon_generation_rng,
    );

    // Create staircase entity
    let staircase_room = &game.rooms[1];
    let staircase_x = game.dungeon_generation_rng.gen_range(
        staircase_room.center.x - staircase_room.x_radius as i16 + 1,
        staircase_room.center.x + staircase_room.x_radius as i16,
    );
    let staircase_y = game.dungeon_generation_rng.gen_range(
        staircase_room.center.y - staircase_room.y_radius as i16 + 1,
        staircase_room.center.y + staircase_room.y_radius as i16,
    );
    entities::create_staircase(
        PositionComponent {
            x: staircase_x,
            y: staircase_y,
        },
        &mut game.world,
    );

    // Display message
    game.floor_number += 1;
    game.message_log.push(Message::new(
        format!("Entering floor {}", game.floor_number),
        MessageColor::White,
    ));
}

pub struct Room {
    pub center: PositionComponent,
    pub x_radius: u16,
    pub y_radius: u16,
}

impl PositionComponent {
    fn get_neighbors(self) -> [(i16, i16); 8] {
        [
            (self.x + 1, self.y),
            (self.x - 1, self.y),
            (self.x, self.y + 1),
            (self.x, self.y - 1),
            (self.x + 1, self.y + 1),
            (self.x + 1, self.y - 1),
            (self.x - 1, self.y + 1),
            (self.x - 1, self.y - 1),
        ]
    }
}

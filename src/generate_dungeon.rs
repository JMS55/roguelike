use crate::components::*;
use crate::game::{Game, Message, MessageColor};
use rand::Rng;
use std::collections::HashSet;

pub fn generate_dungeon(game: &mut Game) {
    // Place rooms
    let mut rooms = Vec::with_capacity(41);
    let starting_room = Room {
        center: PositionComponent { x: 0, y: 0 },
        x_radius: 3,
        y_radius: 3,
    };
    rooms.push(starting_room);
    'room_placing_loop: for _ in 0..200 {
        let room = Room {
            center: PositionComponent {
                x: game.dungeon_generation_rng.gen_range(-30, 31),
                y: game.dungeon_generation_rng.gen_range(-30, 31),
            },
            x_radius: game.dungeon_generation_rng.gen_range(2, 8),
            y_radius: game.dungeon_generation_rng.gen_range(2, 8),
        };
        for other_room in &rooms {
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
        rooms.push(room);
    }

    // Place corridors
    let mut corridor_positions = HashSet::with_capacity((rooms.len() / 2) * 12);
    for (start_room_index, start_room) in rooms.iter().enumerate() {
        let mut end_room_index = game.dungeon_generation_rng.gen_range(0, rooms.len());
        while end_room_index == start_room_index {
            end_room_index = game.dungeon_generation_rng.gen_range(0, rooms.len());
        }
        let end_room = &rooms[end_room_index];
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
            corridor_positions.insert(PositionComponent { x, y: start_y });
        }
        for y in start_y.min(end_y)..=start_y.max(end_y) {
            corridor_positions.insert(PositionComponent { x: end_x, y });
        }
    }

    // Get list of all wall positions
    let mut wall_positions =
        HashSet::with_capacity(rooms.len() * 36 + corridor_positions.len() * 3);
    for room in &rooms {
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
    for corridor_position in &corridor_positions {
        'neighbor_loop: for (x, y) in &corridor_position.get_neighbors() {
            for room in &rooms {
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

    // Create floor entities
    for room in &rooms {
        let x_radius = room.x_radius as i16;
        let y_radius = room.y_radius as i16;
        for x in -x_radius..=x_radius {
            for y in -y_radius..=y_radius {
                create_floor(
                    PositionComponent {
                        x: room.center.x + x,
                        y: room.center.y + y,
                    },
                    game,
                );
            }
        }
    }
    for corridor_position in &corridor_positions {
        create_floor(*corridor_position, game);
    }

    // Create wall entities
    for wall_position in wall_positions.difference(&corridor_positions) {
        create_wall(*wall_position, game);
    }

    // Create staircase entity
    let staircase_room = &rooms[1];
    let staircase_x = game.dungeon_generation_rng.gen_range(
        staircase_room.center.x - staircase_room.x_radius as i16 + 1,
        staircase_room.center.x + staircase_room.x_radius as i16,
    );
    let staircase_y = game.dungeon_generation_rng.gen_range(
        staircase_room.center.y - staircase_room.y_radius as i16 + 1,
        staircase_room.center.y + staircase_room.y_radius as i16,
    );
    create_staircase(
        PositionComponent {
            x: staircase_x,
            y: staircase_y,
        },
        game,
    );

    // Display message
    game.floor_number += 1;
    game.message_log.push(Message::new(
        format!("Entering floor {}", game.floor_number),
        MessageColor::White,
    ));
}

struct Room {
    center: PositionComponent,
    x_radius: u16,
    y_radius: u16,
}

fn create_floor(position: PositionComponent, game: &mut Game) {
    game.world.insert(
        (),
        Some((
            NameComponent {
                name: "Floor",
                concealed_name: "Floor",
                is_concealed: false,
            },
            position,
            IntangibleComponent {},
            SpriteComponent {
                id: "floor",
                in_foreground: false,
            },
        )),
    );
}

fn create_wall(position: PositionComponent, game: &mut Game) {
    let sprite_id = if game.rng.gen_ratio(1, 4) {
        "wall_mossy"
    } else {
        "wall"
    };
    game.world.insert(
        (),
        Some((
            NameComponent {
                name: "Wall",
                concealed_name: "Wall",
                is_concealed: false,
            },
            position,
            SpriteComponent {
                id: sprite_id,
                in_foreground: true,
            },
        )),
    );
}

fn create_staircase(position: PositionComponent, game: &mut Game) {
    game.world.insert(
        (),
        Some((
            NameComponent {
                name: "Staircase",
                concealed_name: "Staircase",
                is_concealed: false,
            },
            position,
            SpriteComponent {
                id: "staircase",
                in_foreground: true,
            },
            StaircaseComponent {},
        )),
    );
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

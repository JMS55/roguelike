use crate::data::{MessageColor, MessageDisplayLength, MessageLog, Player, Position};
use crate::entities;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use specs::{Join, World, WorldExt};
use std::collections::HashSet;

pub struct GenerateDungeonSystem {
    next_floor: u32,
    rng: Pcg64,
}

impl GenerateDungeonSystem {
    pub fn new() -> Self {
        Self {
            next_floor: 1,
            rng: Pcg64::from_entropy(),
        }
    }
}

impl GenerateDungeonSystem {
    pub fn run(&mut self, world: &mut World) {
        let mut non_player_entities = Vec::new();
        {
            let entities = world.entities();
            let mut player_data = world.write_storage::<Player>();
            let mut position_data = world.write_storage::<Position>();
            let (player_entity, player, player_position) =
                (&entities, &mut player_data, &mut position_data)
                    .join()
                    .next()
                    .unwrap();
            *player_position = Position::new(0, 0);
            player.turns_taken = 0;

            for entity in (&entities).join() {
                if entity != player_entity {
                    non_player_entities.push(entity);
                }
            }
        }
        for entity in non_player_entities {
            world.delete_entity(entity).unwrap();
        }

        world.fetch_mut::<MessageLog>().new_message(
            format!("Entering floor {}", self.next_floor),
            MessageColor::White,
            MessageDisplayLength::Medium,
        );
        self.next_floor += 1;

        let mut rooms = Vec::with_capacity(41);
        let starting_room = Room {
            center_x: 0,
            center_y: 0,
            x_radius: 3,
            y_radius: 3,
        };
        rooms.push(starting_room);
        'room_placing_loop: for _ in 0..200 {
            let room = Room {
                center_x: self.rng.gen_range(-30, 31),
                center_y: self.rng.gen_range(-30, 31),
                x_radius: self.rng.gen_range(2, 8),
                y_radius: self.rng.gen_range(2, 8),
            };
            for other_room in &rooms {
                let required_gap = self.rng.gen_range(3, 10);
                let x_gap = (room.center_x - other_room.center_x).abs()
                    - room.x_radius as i32
                    - other_room.x_radius as i32
                    - 3;
                let y_gap = (room.center_y - other_room.center_y).abs()
                    - room.y_radius as i32
                    - other_room.y_radius as i32
                    - 3;
                let actual_gap = x_gap.max(y_gap);
                if actual_gap < required_gap && actual_gap != -1 {
                    continue 'room_placing_loop;
                }
            }
            rooms.push(room);
        }

        let mut corridor_positions = HashSet::with_capacity((rooms.len() / 2) * 12);
        for (start_room_index, start_room) in rooms.iter().enumerate() {
            let mut end_room_index = self.rng.gen_range(0, rooms.len());
            while end_room_index == start_room_index {
                end_room_index = self.rng.gen_range(0, rooms.len());
            }
            let end_room = &rooms[end_room_index];
            let start_x = self.rng.gen_range(
                start_room.center_x - start_room.x_radius as i32,
                start_room.center_x + start_room.x_radius as i32 + 1,
            );
            let start_y = self.rng.gen_range(
                start_room.center_y - start_room.y_radius as i32,
                start_room.center_y + start_room.y_radius as i32 + 1,
            );
            let end_x = self.rng.gen_range(
                end_room.center_x - end_room.x_radius as i32,
                end_room.center_x + end_room.x_radius as i32 + 1,
            );
            let end_y = self.rng.gen_range(
                end_room.center_y - end_room.y_radius as i32,
                end_room.center_y + end_room.y_radius as i32 + 1,
            );
            for x in start_x.min(end_x)..start_x.max(end_x) {
                corridor_positions.insert((x, start_y));
            }
            for y in start_y.min(end_y)..=start_y.max(end_y) {
                corridor_positions.insert((end_x, y));
            }
        }

        let mut room_wall_positions = HashSet::with_capacity(rooms.len() * 36);
        for room in &rooms {
            let x_radius = room.x_radius as i32;
            let y_radius = room.y_radius as i32;
            for x in -(x_radius + 1)..=(x_radius + 1) {
                room_wall_positions.insert((room.center_x + x, room.center_y + y_radius + 1));
                room_wall_positions.insert((room.center_x + x, room.center_y - y_radius - 1));
            }
            for y in -y_radius..=y_radius {
                room_wall_positions.insert((room.center_x + x_radius + 1, room.center_y + y));
                room_wall_positions.insert((room.center_x - x_radius - 1, room.center_y + y));
            }
        }

        let mut corridor_wall_positions = HashSet::with_capacity(corridor_positions.len() * 3);
        for (x, y) in &corridor_positions {
            'neighbor_loop: for (x, y) in &get_neighbors(*x, *y) {
                for room in &rooms {
                    let x_radius = room.x_radius as i32;
                    let y_radius = room.y_radius as i32;
                    let x_range = (room.center_x - x_radius - 1)..=(room.center_x + x_radius + 1);
                    let y_range = (room.center_y - y_radius - 1)..=(room.center_y + y_radius + 1);
                    if x_range.contains(x) && y_range.contains(y) {
                        continue 'neighbor_loop;
                    }
                }
                corridor_wall_positions.insert((*x, *y));
            }
        }

        let wall_positions = room_wall_positions
            .union(&corridor_wall_positions)
            .cloned()
            .collect::<HashSet<(i32, i32)>>();
        let wall_positions = wall_positions
            .difference(&corridor_positions)
            .cloned()
            .collect::<HashSet<(i32, i32)>>();
        for (x, y) in &wall_positions {
            entities::create_wall(*x, *y, world);
        }

        let staircase_room = &rooms[1];
        let staircase_x = self.rng.gen_range(
            staircase_room.center_x - staircase_room.x_radius as i32 + 1,
            staircase_room.center_x + staircase_room.x_radius as i32,
        );
        let staircase_y = self.rng.gen_range(
            staircase_room.center_y - staircase_room.y_radius as i32 + 1,
            staircase_room.center_y + staircase_room.y_radius as i32,
        );
        entities::create_staircase(staircase_x, staircase_y, world);

        for room in &rooms {
            if self.rng.gen_ratio(1, 4) {
                let x = self.rng.gen_range(
                    room.center_x - room.x_radius as i32,
                    room.center_x + room.x_radius as i32 + 1,
                );
                let y = self.rng.gen_range(
                    room.center_y - room.y_radius as i32,
                    room.center_y + room.y_radius as i32 + 1,
                );
                if x != staircase_x && y != staircase_y {
                    entities::create_spawner(x, y, world);
                }
            }
        }
    }
}

struct Room {
    center_x: i32,
    center_y: i32,
    x_radius: u32,
    y_radius: u32,
}

fn get_neighbors(x: i32, y: i32) -> [(i32, i32); 8] {
    [
        (x + 1, y),
        (x - 1, y),
        (x, y + 1),
        (x, y - 1),
        (x + 1, y + 1),
        (x + 1, y - 1),
        (x - 1, y + 1),
        (x - 1, y - 1),
    ]
}

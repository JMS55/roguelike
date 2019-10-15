use crate::data::{
    Direction, MessageColor, MessageDisplayLength, MessageLog, Player, Position, Rarity,
};
use crate::entities;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use specs::{Join, World, WorldExt};
use std::collections::HashSet;

pub struct GenerateDungeonSystem {
    next_floor: u32,
    next_boss_floor: u32,
    rng: Pcg64,
}

impl GenerateDungeonSystem {
    pub fn new() -> Self {
        let mut rng = Pcg64::from_entropy();
        Self {
            next_floor: 1,
            next_boss_floor: rng.gen_range(8, 11),
            rng,
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
            player.facing_direction = Direction::Up;
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

        if self.next_floor != self.next_boss_floor {
            let mut rooms = Vec::with_capacity(41);
            let starting_room = Room {
                center: Position::new(0, 0),
                x_radius: 3,
                y_radius: 3,
            };
            rooms.push(starting_room);
            'room_placing_loop: for _ in 0..200 {
                let room = Room {
                    center: Position::new(self.rng.gen_range(-30, 31), self.rng.gen_range(-30, 31)),
                    x_radius: self.rng.gen_range(2, 8),
                    y_radius: self.rng.gen_range(2, 8),
                };
                for other_room in &rooms {
                    let required_gap = self.rng.gen_range(3, 10);
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

            let mut corridor_positions = HashSet::with_capacity((rooms.len() / 2) * 12);
            for (start_room_index, start_room) in rooms.iter().enumerate() {
                let mut end_room_index = self.rng.gen_range(0, rooms.len());
                while end_room_index == start_room_index {
                    end_room_index = self.rng.gen_range(0, rooms.len());
                }
                let end_room = &rooms[end_room_index];
                let start_x = self.rng.gen_range(
                    start_room.center.x - start_room.x_radius as i16,
                    start_room.center.x + start_room.x_radius as i16 + 1,
                );
                let start_y = self.rng.gen_range(
                    start_room.center.y - start_room.y_radius as i16,
                    start_room.center.y + start_room.y_radius as i16 + 1,
                );
                let end_x = self.rng.gen_range(
                    end_room.center.x - end_room.x_radius as i16,
                    end_room.center.x + end_room.x_radius as i16 + 1,
                );
                let end_y = self.rng.gen_range(
                    end_room.center.y - end_room.y_radius as i16,
                    end_room.center.y + end_room.y_radius as i16 + 1,
                );
                for x in start_x.min(end_x)..start_x.max(end_x) {
                    corridor_positions.insert(Position::new(x, start_y));
                }
                for y in start_y.min(end_y)..=start_y.max(end_y) {
                    corridor_positions.insert(Position::new(end_x, y));
                }
            }

            for room in &rooms {
                let x_radius = room.x_radius as i16;
                let y_radius = room.y_radius as i16;
                for x in -x_radius..=x_radius {
                    for y in -y_radius..=y_radius {
                        entities::create_floor(
                            Position::new(room.center.x + x, room.center.y + y),
                            world,
                        );
                    }
                }
            }
            for corridor_position in &corridor_positions {
                entities::create_floor(*corridor_position, world);
            }

            let mut room_wall_positions = HashSet::with_capacity(rooms.len() * 36);
            for room in &rooms {
                let x_radius = room.x_radius as i16;
                let y_radius = room.y_radius as i16;
                for x in -(x_radius + 1)..=(x_radius + 1) {
                    room_wall_positions.insert(Position::new(
                        room.center.x + x,
                        room.center.y + y_radius + 1,
                    ));
                    room_wall_positions.insert(Position::new(
                        room.center.x + x,
                        room.center.y - y_radius - 1,
                    ));
                }
                for y in -y_radius..=y_radius {
                    room_wall_positions.insert(Position::new(
                        room.center.x + x_radius + 1,
                        room.center.y + y,
                    ));
                    room_wall_positions.insert(Position::new(
                        room.center.x - x_radius - 1,
                        room.center.y + y,
                    ));
                }
            }

            let mut corridor_wall_positions = HashSet::with_capacity(corridor_positions.len() * 3);
            for corridor_position in &corridor_positions {
                'neighbor_loop: for (x, y) in &corridor_position.get_neighbors() {
                    for room in &rooms {
                        let x_radius = room.x_radius as i16;
                        let y_radius = room.y_radius as i16;
                        let x_range =
                            (room.center.x - x_radius - 1)..=(room.center.x + x_radius + 1);
                        let y_range =
                            (room.center.y - y_radius - 1)..=(room.center.y + y_radius + 1);
                        if x_range.contains(x) && y_range.contains(y) {
                            continue 'neighbor_loop;
                        }
                    }
                    corridor_wall_positions.insert(Position::new(*x, *y));
                }
            }

            let wall_positions = room_wall_positions
                .union(&corridor_wall_positions)
                .cloned()
                .collect::<HashSet<Position>>();
            let wall_positions = wall_positions
                .difference(&corridor_positions)
                .cloned()
                .collect::<HashSet<Position>>();
            for wall_position in &wall_positions {
                entities::create_wall(*wall_position, world, &mut self.rng);
            }

            let staircase_room = &rooms[1];
            let staircase_x = self.rng.gen_range(
                staircase_room.center.x - staircase_room.x_radius as i16 + 1,
                staircase_room.center.x + staircase_room.x_radius as i16,
            );
            let staircase_y = self.rng.gen_range(
                staircase_room.center.y - staircase_room.y_radius as i16 + 1,
                staircase_room.center.y + staircase_room.y_radius as i16,
            );
            entities::create_staircase(Position::new(staircase_x, staircase_y), world);

            for room in &rooms {
                if self.rng.gen_ratio(1, 4) {
                    let x = self.rng.gen_range(
                        room.center.x - room.x_radius as i16,
                        room.center.x + room.x_radius as i16 + 1,
                    );
                    let y = self.rng.gen_range(
                        room.center.y - room.y_radius as i16,
                        room.center.y + room.y_radius as i16 + 1,
                    );
                    if x != staircase_x && y != staircase_y {
                        entities::create_spawner(Position::new(x, y), world);
                    }
                }
            }
        } else {
            let semicircle_positions = [
                (1, 10),
                (2, 10),
                (3, 10),
                (3, 9),
                (4, 9),
                (5, 9),
                (5, 8),
                (6, 8),
                (6, 7),
                (7, 7),
                (7, 6),
                (7, 5),
                (8, 5),
                (8, 4),
                (8, 3),
                (8, 2),
                (8, 1),
                (8, 0),
                (8, -1),
                (7, -1),
                (7, -2),
                (7, -3),
                (6, -3),
                (6, -4),
                (5, -4),
                (5, -5),
                (4, -5),
                (3, -5),
                (3, -6),
                (2, -6),
                (1, -6),
            ];
            entities::create_wall(Position::new(0, 10), world, &mut self.rng);
            entities::create_wall(Position::new(0, -6), world, &mut self.rng);
            for (wall_x, wall_y) in &semicircle_positions {
                entities::create_wall(Position::new(*wall_x, *wall_y), world, &mut self.rng);
                entities::create_wall(Position::new(-*wall_x, *wall_y), world, &mut self.rng);
            }

            for floor_y in -5..=9 {
                entities::create_floor(Position::new(-1, floor_y), world);
                entities::create_floor(Position::new(-2, floor_y), world);
                entities::create_floor(Position::new(0, floor_y), world);
                entities::create_floor(Position::new(1, floor_y), world);
                entities::create_floor(Position::new(2, floor_y), world);
            }
            for floor_y in -4..=8 {
                entities::create_floor(Position::new(-3, floor_y), world);
                entities::create_floor(Position::new(-4, floor_y), world);
                entities::create_floor(Position::new(3, floor_y), world);
                entities::create_floor(Position::new(4, floor_y), world);
            }
            for floor_y in -3..=7 {
                entities::create_floor(Position::new(-5, floor_y), world);
                entities::create_floor(Position::new(5, floor_y), world);
            }
            for floor_y in -2..=6 {
                entities::create_floor(Position::new(-6, floor_y), world);
                entities::create_floor(Position::new(6, floor_y), world);
            }
            for floor_y in 0..=4 {
                entities::create_floor(Position::new(-7, floor_y), world);
                entities::create_floor(Position::new(7, floor_y), world);
            }

            entities::create_random_class1(Rarity::Epic, Position::new(0, 4), world);

            self.next_boss_floor = self.rng.gen_range(8, 11) + self.next_floor;
        }

        world.fetch_mut::<MessageLog>().new_message(
            format!("Entering floor {}", self.next_floor),
            MessageColor::White,
            MessageDisplayLength::Medium,
        );

        self.next_floor += 1;
    }
}

struct Room {
    center: Position,
    x_radius: u16,
    y_radius: u16,
}

impl Position {
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

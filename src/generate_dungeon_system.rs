use crate::components::{Direction, PositionComponent, SpriteComponent};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use specs::{Builder, LazyUpdate, Read, System, WorldExt};
use std::collections::HashSet;

pub struct GenerateDungeonSystem {
    rng: Pcg64,
}

impl GenerateDungeonSystem {
    pub fn new() -> Self {
        Self {
            rng: Pcg64::from_entropy(),
        }
    }
}

impl<'s> System<'s> for GenerateDungeonSystem {
    type SystemData = Read<'s, LazyUpdate>;

    fn run(&mut self, lazy_update: Self::SystemData) {
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
                let required_gap = self.rng.gen_range(1, 6);
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
        for start_room in &rooms {
            let end_room = &rooms[self.rng.gen_range(0, rooms.len())];
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

        for (x, y) in room_wall_positions
            .union(&corridor_wall_positions)
            .cloned()
            .collect::<HashSet<(i32, i32)>>()
            .difference(&corridor_positions)
        {
            create_wall(*x, *y, &lazy_update);
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

fn create_wall(x: i32, y: i32, lazy_update: &Read<LazyUpdate>) {
    lazy_update.exec_mut(move |world| {
        world
            .create_entity()
            .with(PositionComponent {
                x,
                y,
                facing_direction: Direction::Right,
            })
            .with(SpriteComponent { id: "blue" })
            .build();
    });
}

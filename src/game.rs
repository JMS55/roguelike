use crate::components::*;
use crate::entities;
use hecs::{Entity, World};
use noise::{NoiseFn, OpenSimplex};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use sdl2::keyboard::{KeyboardState, Scancode};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

pub struct Game {
    pub ecs: World,
    actor_queue: VecDeque<Entity>,

    pub player_entity: Entity,
    pub player_facing_direction: PositionComponent,
    player_inventory: [Option<Entity>; 16],
    player_turns_until_passive_healing: u32,
    last_input_time: Instant,

    pub rooms: Vec<Room>,
    floor_positions: HashSet<PositionComponent>,
    floor_number: u32,

    pub rng: Pcg64,
    dungeon_generation_rng: Pcg64,

    pub animation_queue: VecDeque<Box<dyn Animation>>,
    noise_generator: OpenSimplex,
    time_game_started: Instant,
}

impl Game {
    pub fn new(dungeon_generation_seed: String, rng_seed: String) -> Self {
        let mut dungeon_hasher = DefaultHasher::new();
        dungeon_generation_seed.hash(&mut dungeon_hasher);
        let dungeon_generation_seed = dungeon_hasher.finish();

        let mut rng_hasher = DefaultHasher::new();
        rng_seed.hash(&mut rng_hasher);
        let rng_seed = rng_hasher.finish();

        let mut ecs = World::new();
        let mut rng = Pcg64::seed_from_u64(rng_seed);
        let player_entity = entities::create_player(&mut ecs, &mut rng);

        let mut game = Self {
            ecs,
            actor_queue: VecDeque::new(),

            player_entity,
            player_facing_direction: PositionComponent { x: 0, y: 1 },
            player_inventory: [None; 16],
            player_turns_until_passive_healing: 10,
            last_input_time: Instant::now(),

            rooms: Vec::new(),
            floor_positions: HashSet::new(),
            floor_number: 0,

            rng,
            dungeon_generation_rng: Pcg64::seed_from_u64(dungeon_generation_seed),

            animation_queue: VecDeque::new(),
            noise_generator: OpenSimplex::new(),
            time_game_started: Instant::now(),
        };

        game.next_floor();
        game
    }

    pub fn run(&mut self, keyboard: &KeyboardState) {
        if self
            .ecs
            .get::<CombatComponent>(self.player_entity)
            .unwrap()
            .current_health
            != 0
            && self.animation_queue.is_empty()
        {
            // If actor queue is empty, add player and then all AI entities
            if self.actor_queue.is_empty() {
                self.actor_queue.extend(
                    self.ecs
                        .query::<()>()
                        .with::<AIComponent>()
                        .iter()
                        .map(|(entity, _)| entity),
                );
                self.actor_queue.push_back(self.player_entity);
            }

            // Run next actor
            let entity = *self.actor_queue.front().unwrap();
            if entity == self.player_entity {
                if self.last_input_time.elapsed() >= Duration::from_millis(120) {
                    let mut player_acted = false;

                    let mut keystate = (
                        0,
                        0,
                        keyboard.is_scancode_pressed(Scancode::LShift)
                            || keyboard.is_scancode_pressed(Scancode::RShift),
                    );
                    if keyboard.is_scancode_pressed(Scancode::W)
                        || keyboard.is_scancode_pressed(Scancode::Up)
                    {
                        keystate.1 = 1;
                    }
                    if keyboard.is_scancode_pressed(Scancode::A)
                        || keyboard.is_scancode_pressed(Scancode::Left)
                    {
                        keystate.0 = -1;
                    }
                    if keyboard.is_scancode_pressed(Scancode::S)
                        || keyboard.is_scancode_pressed(Scancode::Down)
                    {
                        keystate.1 = -1;
                    }
                    if keyboard.is_scancode_pressed(Scancode::D)
                        || keyboard.is_scancode_pressed(Scancode::Right)
                    {
                        keystate.0 = 1;
                    }
                    if keystate.0 != 0 || keystate.1 != 0 {
                        self.player_facing_direction = PositionComponent {
                            x: keystate.0,
                            y: keystate.1,
                        };
                        if !keystate.2 {
                            let player_position = *self
                                .ecs
                                .get::<PositionComponent>(self.player_entity)
                                .unwrap();
                            let attempted_move_position = player_position
                                + PositionComponent {
                                    x: keystate.0,
                                    y: keystate.1,
                                };
                            if self
                                .ecs
                                .query::<&PositionComponent>()
                                .iter()
                                .all(|(_, position)| *position != attempted_move_position)
                            {
                                *self
                                    .ecs
                                    .get_mut::<PositionComponent>(self.player_entity)
                                    .unwrap() = attempted_move_position;
                                player_acted = true;
                            }
                        }
                        self.last_input_time = Instant::now();
                    }
                    if keyboard.is_scancode_pressed(Scancode::E) {
                        player_acted = true;
                        self.last_input_time = Instant::now();
                    }
                    if keyboard.is_scancode_pressed(Scancode::Q)
                        && (self.player_facing_direction.x == 0
                            || self.player_facing_direction.y == 0)
                    {
                        let player_position = *self
                            .ecs
                            .get::<PositionComponent>(self.player_entity)
                            .unwrap();
                        let interacting_with_position =
                            player_position + self.player_facing_direction;

                        // Staircase interaction
                        if self
                            .ecs
                            .query::<&PositionComponent>()
                            .with::<StaircaseComponent>()
                            .iter()
                            .any(|(_, position)| *position == interacting_with_position)
                        {
                            {
                                let mut player_combat = self
                                    .ecs
                                    .get_mut::<CombatComponent>(self.player_entity)
                                    .unwrap();

                                // Remove all buffs
                                player_combat.strength_buff = (0, 0);
                                player_combat.focus_buff = (0, 0);
                                player_combat.agility_buff = (0, 0);
                                player_combat.luck_buff = (0, 0);
                                player_combat.magic_immune_buff = false;
                                player_combat.explode_on_death_buff = (0, 0);

                                // Remove all debuffs
                                player_combat.strength_debuff = (0, 0);
                                player_combat.focus_debuff = (0, 0);
                                player_combat.agility_debuff = (0, 0);
                                player_combat.luck_debuff = (0, 0);
                                player_combat.burn_debuff = (0, 0);

                                // Heal the player by 20% of their max health
                                player_combat.current_health = player_combat.current_health
                                    + (player_combat.max_health as f64 * 0.2).round() as u32;
                                player_combat.current_health =
                                    player_combat.current_health.min(player_combat.max_health);
                            }

                            self.next_floor();
                        }

                        // Disguised Mimic interaction
                        let mimic_entity = self
                            .ecs
                            .query::<(&MimicComponent, &PositionComponent)>()
                            .iter()
                            .find_map(|(entity, (mimic_component, position))| {
                                if *position == interacting_with_position
                                    && mimic_component.disguised
                                {
                                    Some(entity)
                                } else {
                                    None
                                }
                            });
                        if let Some(mimic_entity) = mimic_entity {
                            // Undisguise Mimic
                            self.ecs
                                .get_mut::<MimicComponent>(mimic_entity)
                                .unwrap()
                                .disguised = false;
                            // Mimic starts chasing player
                            self.ecs
                                .insert_one(
                                    mimic_entity,
                                    AIComponent {
                                        ai: Box::new(entities::MimicAI {
                                            chase_target: Some(self.player_entity),
                                        }),
                                    },
                                )
                                .unwrap();

                            player_acted = true;
                        }

                        self.last_input_time = Instant::now();
                    }

                    if player_acted {
                        self.entity_end_of_turn(entity);
                        self.actor_queue.pop_front();
                    }
                }
            } else {
                // If entity is still alive
                if let Ok(mut ai) = self
                    .ecs
                    .get_mut::<AIComponent>(entity)
                    // Clone the AI to prevent borrow errors
                    .map(|ai_component| ai_component.ai.clone())
                {
                    // Run the enitiy's AI. This mutates the copy we made.
                    ai.run(entity, self);
                    // Overwrite the old AI with the copy we made (does nothing if the entity is dead)
                    let _ = self.ecs.insert_one(entity, AIComponent { ai });
                    self.entity_end_of_turn(entity);
                }
                self.actor_queue.pop_front();
            }
        }
    }

    pub fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        textures: &mut HashMap<String, Texture>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
    ) {
        let player_position = *self
            .ecs
            .get::<PositionComponent>(self.player_entity)
            .unwrap();
        let player_combat = *self.ecs.get::<CombatComponent>(self.player_entity).unwrap();

        // Render background
        if !cfg!(debug_assertions) {
            // Determine color modifier
            let (red_modifier, green_modifier, blue_modifier) =
                if (player_combat.current_health as f64 / player_combat.max_health as f64) < 0.3 {
                    (1.7, 0.2, 0.4)
                } else {
                    (1.0, 1.0, 1.8)
                };

            // Create pixel data
            let mut pixel_data: [u8; 480 * 480 * 3] = [0; 480 * 480 * 3];
            let time_since_game_started = self.time_game_started.elapsed().as_secs_f64();
            let mut i = 0;
            for y in 0..480 {
                for x in 0..480 {
                    let mut n = self.noise_generator.get([
                        x as f64 / 96.0,
                        y as f64 / 96.0,
                        time_since_game_started,
                    ]);
                    // If tile is neighboring player then draw the background slightly lighter
                    let m = if x >= 192 && x <= 288 && y >= 192 && y <= 288 {
                        40.0
                    } else {
                        16.0
                    };
                    n = (n + 1.0) * m;
                    pixel_data[i] = (n * red_modifier).round() as u8;
                    pixel_data[i + 1] = (n * green_modifier).round() as u8;
                    pixel_data[i + 2] = (n * blue_modifier).round() as u8;
                    i += 3;
                }
            }

            // Copy pixel data to canvas
            let mut texture = texture_creator
                .create_texture_static(PixelFormatEnum::RGB24, 480, 480)
                .unwrap();
            texture.update(None, &pixel_data, 480 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
        }

        // Render floor tiles
        {
            for position in self
                .floor_positions
                .iter()
                .map(|position| PositionComponent {
                    x: position.x - player_position.x + 7,
                    y: player_position.y - position.y + 7,
                })
                .filter(|position| (0..15).contains(&position.x))
                .filter(|position| (0..15).contains(&position.y))
            {
                let dest_rect =
                    Rect::new((position.x * 32) as i32, (position.y * 32) as i32, 32, 32);
                canvas.copy(&textures["floor"], None, dest_rect).unwrap();
            }
        }

        // Render entities
        {
            for (entity, sprite, position) in self
                .ecs
                .query::<(&SpriteComponent, &PositionComponent)>()
                .iter()
                .map(|(entity, (sprite, position))| {
                    (
                        entity,
                        sprite,
                        PositionComponent {
                            x: position.x - player_position.x + 7,
                            y: player_position.y - position.y + 7,
                        },
                    )
                })
                .filter(|(_, _, position)| (0..15).contains(&position.x))
                .filter(|(_, _, position)| (0..15).contains(&position.y))
                .filter(|(entity, _, _)| {
                    if let Some(animation) = self.animation_queue.front() {
                        !animation.entities_not_to_render().contains(&entity)
                    } else {
                        true
                    }
                })
            {
                let dest_rect =
                    Rect::new((position.x * 32) as i32, (position.y * 32) as i32, 32, 32);
                canvas
                    .copy(
                        &textures
                            .get((sprite.id)(entity, self))
                            .unwrap_or(&textures["placeholder"]),
                        None,
                        dest_rect,
                    )
                    .unwrap();
            }
        }

        // Render player facing_direction indicator
        {
            let texture_id =
                if self.player_facing_direction.x == 0 || self.player_facing_direction.y == 0 {
                    "direction_indicator"
                } else {
                    "direction_indicator_diagonal"
                };
            let dest_rect = Rect::new(224, 224, 32, 32);
            let rotation = match self.player_facing_direction {
                PositionComponent { x: 0, y: 1 } => 90.0,
                PositionComponent { x: 0, y: -1 } => 270.0,
                PositionComponent { x: -1, y: 0 } => 0.0,
                PositionComponent { x: 1, y: 0 } => 180.0,
                PositionComponent { x: -1, y: 1 } => 0.0,
                PositionComponent { x: -1, y: -1 } => 270.0,
                PositionComponent { x: 1, y: -1 } => 180.0,
                PositionComponent { x: 1, y: 1 } => 90.0,
                _ => unreachable!(),
            };
            canvas
                .copy_ex(
                    &textures[texture_id],
                    None,
                    dest_rect,
                    rotation,
                    None,
                    false,
                    false,
                )
                .unwrap();
        }

        // Render animation
        {
            if let Some(animation) = self.animation_queue.front_mut() {
                animation.render(canvas, textures, texture_creator, font);
                if animation.is_complete() {
                    self.animation_queue.pop_front();
                }
            }
        }
    }

    pub fn damage_entity(&mut self, damage_info: DamageInfo) {
        let mut target_combat = self
            .ecs
            .get_mut::<CombatComponent>(damage_info.target)
            .unwrap();
        let mut attack_hit = true;
        let mut critical_hit = false;
        if damage_info.variance {
            critical_hit = self.rng.gen_bool(target_combat.get_luck() as f64 / 200.0);
            if !critical_hit {
                attack_hit = self
                    .rng
                    .gen_bool(1.0 - target_combat.get_luck() as f64 / 100.0);
            }
        }

        if attack_hit {
            // Deal damage
            let damage_negated = match damage_info.damage_type {
                DamageType::None => 0,
                DamageType::Strength => (target_combat.get_strength() as f64 / 2.0).round() as u32,
                DamageType::Focus => {
                    if target_combat.magic_immune_buff {
                        damage_info.damage_amount
                    } else {
                        (target_combat.get_focus() as f64 / 2.0).round() as u32
                    }
                }
                DamageType::Agility => (target_combat.get_agility() as f64 / 2.0).round() as u32,
            };
            let mut damage = damage_info.damage_amount.saturating_sub(damage_negated);
            if damage_info.variance {
                if critical_hit {
                    damage = (damage as f64 * 1.5).round() as u32;
                } else {
                    damage = self.rng.gen_range(damage, damage + 5);
                }
            }
            target_combat.current_health = target_combat.current_health.saturating_sub(damage);

            // Add damage animation
            let player_position = *self
                .ecs
                .get::<PositionComponent>(self.player_entity)
                .unwrap();
            let target_position = *self
                .ecs
                .get::<PositionComponent>(damage_info.target)
                .unwrap();
            let render_position = PositionComponent {
                x: target_position.x - player_position.x + 7,
                y: player_position.y - target_position.y + 7,
            };
            if (0..15).contains(&render_position.x) && (0..15).contains(&render_position.y) {
                let target_sprite = self.ecs.get::<SpriteComponent>(damage_info.target).unwrap();
                self.animation_queue.push_back(Box::new(DamageAnimation {
                    time_started: None,
                    damaged_entity: damage_info.target,
                    entity_position: render_position,
                    entity_sprite: (target_sprite.id)(damage_info.target, self),
                    damage_dealt: damage,
                    critical_hit,
                }));
            }

            // Undisguise Mimic
            if let Ok(mut mimic_component) = self.ecs.get_mut::<MimicComponent>(damage_info.target)
            {
                mimic_component.disguised = false;
            }

            // On death
            if target_combat.current_health == 0 {
                let (explosion_damage, explosion_radius) = target_combat.explode_on_death_buff;
                drop(target_combat);

                // Delete entity if they are not the player
                if damage_info.target != self.player_entity {
                    self.ecs.despawn(damage_info.target).unwrap();
                }

                // Handle explode on death buff
                if explosion_radius != 0 {
                    let explosion_center = *self
                        .ecs
                        .get::<PositionComponent>(damage_info.target)
                        .unwrap();
                    let entities_hit_by_explosion = self
                        .ecs
                        .query::<&PositionComponent>()
                        .with::<CombatComponent>()
                        .iter()
                        .filter_map(|(entity, position)| {
                            if (position.x - explosion_center.x)
                                .abs()
                                .max((position.y - explosion_center.y).abs())
                                as u32
                                <= explosion_radius
                            {
                                Some(entity)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<Entity>>();
                    for target in entities_hit_by_explosion {
                        self.damage_entity(DamageInfo {
                            target,
                            damage_amount: explosion_damage,
                            damage_type: DamageType::None,
                            variance: false,
                        });
                    }
                }
            }
        } else {
            // Add miss animation
            let player_position = *self
                .ecs
                .get::<PositionComponent>(self.player_entity)
                .unwrap();
            let target_position = *self
                .ecs
                .get::<PositionComponent>(damage_info.target)
                .unwrap();
            let render_position = PositionComponent {
                x: target_position.x - player_position.x + 7,
                y: player_position.y - target_position.y + 7,
            };
            if (0..15).contains(&render_position.x) && (0..15).contains(&render_position.y) {
                self.animation_queue.push_back(Box::new(MissAnimation {
                    time_started: None,
                    entity_position: render_position,
                }));
            }
        }
    }

    fn next_floor(&mut self) {
        // Cleanup from previous floor
        *self
            .ecs
            .get_mut::<PositionComponent>(self.player_entity)
            .unwrap() = PositionComponent { x: 0, y: 0 };
        let mut entities_to_delete = Vec::new();
        'entity_loop: for (entity, _) in self.ecs.iter() {
            if entity == self.player_entity {
                continue 'entity_loop;
            }
            for inventory_entity in &self.player_inventory {
                if Some(entity) == *inventory_entity {
                    continue 'entity_loop;
                }
            }
            entities_to_delete.push(entity);
        }
        for entity in entities_to_delete {
            self.ecs.despawn(entity).unwrap();
        }
        self.rooms.clear();
        self.floor_positions.clear();

        // Place rooms
        let starting_room = Room {
            center: PositionComponent { x: 0, y: 0 },
            x_radius: 3,
            y_radius: 3,
        };
        self.rooms.push(starting_room);
        'room_placing_loop: for _ in 0..200 {
            let room = Room {
                center: PositionComponent {
                    x: self.dungeon_generation_rng.gen_range(-30, 31),
                    y: self.dungeon_generation_rng.gen_range(-30, 31),
                },
                x_radius: self.dungeon_generation_rng.gen_range(2, 8),
                y_radius: self.dungeon_generation_rng.gen_range(2, 8),
            };
            for other_room in &self.rooms {
                let required_gap = self.dungeon_generation_rng.gen_range(3, 10);
                let x_gap = (room.center.x - other_room.center.x).abs()
                    - room.x_radius as i32
                    - other_room.x_radius as i32
                    - 3;
                let y_gap = (room.center.y - other_room.center.y).abs()
                    - room.y_radius as i32
                    - other_room.y_radius as i32
                    - 3;
                let actual_gap = x_gap.max(y_gap);
                if actual_gap < required_gap && actual_gap != -1 {
                    continue 'room_placing_loop;
                }
            }
            self.rooms.push(room);
        }

        // Place corridors
        for (start_room_index, start_room) in self.rooms.iter().enumerate() {
            let mut end_room_index = self.dungeon_generation_rng.gen_range(0, self.rooms.len());
            while end_room_index == start_room_index {
                end_room_index = self.dungeon_generation_rng.gen_range(0, self.rooms.len());
            }
            let end_room = &self.rooms[end_room_index];
            let start_x = self.dungeon_generation_rng.gen_range(
                start_room.center.x - start_room.x_radius as i32,
                start_room.center.x + start_room.x_radius as i32 + 1,
            );
            let start_y = self.dungeon_generation_rng.gen_range(
                start_room.center.y - start_room.y_radius as i32,
                start_room.center.y + start_room.y_radius as i32 + 1,
            );
            let end_x = self.dungeon_generation_rng.gen_range(
                end_room.center.x - end_room.x_radius as i32,
                end_room.center.x + end_room.x_radius as i32 + 1,
            );
            let end_y = self.dungeon_generation_rng.gen_range(
                end_room.center.y - end_room.y_radius as i32,
                end_room.center.y + end_room.y_radius as i32 + 1,
            );
            for x in start_x.min(end_x)..start_x.max(end_x) {
                self.floor_positions
                    .insert(PositionComponent { x, y: start_y });
            }
            for y in start_y.min(end_y)..=start_y.max(end_y) {
                self.floor_positions
                    .insert(PositionComponent { x: end_x, y });
            }
        }

        // Get list of all wall positions
        let mut wall_positions = HashSet::with_capacity(1600);
        for room in &self.rooms {
            let x_radius = room.x_radius as i32;
            let y_radius = room.y_radius as i32;
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
        for corridor_position in &self.floor_positions {
            'neighbor_loop: for neighbor in &corridor_position.neighbors() {
                for room in &self.rooms {
                    let x_radius = room.x_radius as i32;
                    let y_radius = room.y_radius as i32;
                    let x_range = (room.center.x - x_radius - 1)..=(room.center.x + x_radius + 1);
                    let y_range = (room.center.y - y_radius - 1)..=(room.center.y + y_radius + 1);
                    if x_range.contains(&neighbor.x) && y_range.contains(&neighbor.y) {
                        continue 'neighbor_loop;
                    }
                }
                wall_positions.insert(PositionComponent {
                    x: neighbor.x,
                    y: neighbor.y,
                });
            }
        }

        // Update list of floor tiles with floor tiles from rooms
        for room in &self.rooms {
            let x_radius = room.x_radius as i32;
            let y_radius = room.y_radius as i32;
            for x in -x_radius..=x_radius {
                for y in -y_radius..=y_radius {
                    self.floor_positions.insert(PositionComponent {
                        x: room.center.x + x,
                        y: room.center.y + y,
                    });
                }
            }
        }

        // Create wall entities
        for wall_position in wall_positions.difference(&self.floor_positions) {
            entities::create_wall(
                *wall_position,
                &mut self.ecs,
                &mut self.dungeon_generation_rng,
            );
        }

        // Create staircase entity
        let staircase_room = &self.rooms[1];
        let staircase_x = self.dungeon_generation_rng.gen_range(
            staircase_room.center.x - staircase_room.x_radius as i32 + 1,
            staircase_room.center.x + staircase_room.x_radius as i32,
        );
        let staircase_y = self.dungeon_generation_rng.gen_range(
            staircase_room.center.y - staircase_room.y_radius as i32 + 1,
            staircase_room.center.y + staircase_room.y_radius as i32,
        );
        entities::create_staircase(
            PositionComponent {
                x: staircase_x,
                y: staircase_y,
            },
            &mut self.ecs,
        );

        self.floor_number += 1;
        self.animation_queue.push_back(Box::new(NextFloorAnimation {
            time_started: None,
            floor_number: self.floor_number,
        }));

        self.spawn_enemies();
    }

    fn spawn_enemies(&mut self) {
        let mut obstacles = self
            .ecs
            .query::<&PositionComponent>()
            .iter()
            .map(|(_, position)| *position)
            .collect::<HashSet<PositionComponent>>();

        for i in 1..self.dungeon_generation_rng.gen_range(7, 10) {
            // Choose a random position within a random room
            if let Some(enemy_room) = self.rooms.get(i) {
                for _ in 0..30 {
                    let enemy_position = PositionComponent {
                        x: self.dungeon_generation_rng.gen_range(
                            enemy_room.center.x - enemy_room.x_radius as i32,
                            enemy_room.center.x + enemy_room.x_radius as i32 + 1,
                        ),
                        y: self.dungeon_generation_rng.gen_range(
                            enemy_room.center.y - enemy_room.y_radius as i32,
                            enemy_room.center.y + enemy_room.y_radius as i32 + 1,
                        ),
                    };

                    // Place an enemy there if the space is unoccupied
                    if !obstacles.contains(&enemy_position) {
                        entities::create_random_enemy(enemy_position, self);
                        obstacles.insert(enemy_position);
                        break;
                    }
                }
            }
        }
    }

    fn entity_end_of_turn(&mut self, entity: Entity) {
        let mut entity_combat = self.ecs.get_mut::<CombatComponent>(entity).unwrap();

        if entity == self.player_entity {
            // Heal player by 2 health every 10 turns
            self.player_turns_until_passive_healing -= 1;
            if self.player_turns_until_passive_healing == 0 {
                entity_combat.current_health = entity_combat
                    .max_health
                    .min(entity_combat.current_health + 2);
                self.player_turns_until_passive_healing = 10;
            }
        }

        // Apply burn debuff damage
        let burn_damage = entity_combat.burn_debuff.0;
        drop(entity_combat);
        if burn_damage != 0 {
            self.damage_entity(DamageInfo {
                target: entity,
                damage_amount: burn_damage,
                damage_type: DamageType::None,
                variance: false,
            });
            self.ecs
                .get_mut::<CombatComponent>(entity)
                .unwrap()
                .burn_debuff
                .1 -= 1;
        }

        let mut entity_combat = self.ecs.get_mut::<CombatComponent>(entity).unwrap();

        // Tick buff timers
        entity_combat.strength_buff.1 = entity_combat.strength_buff.1.saturating_sub(1);
        if entity_combat.strength_buff.1 == 0 {
            entity_combat.strength_buff.0 = 0;
        }
        entity_combat.focus_buff.1 = entity_combat.focus_buff.1.saturating_sub(1);
        if entity_combat.focus_buff.1 == 0 {
            entity_combat.focus_buff.0 = 0;
        }
        entity_combat.agility_buff.1 = entity_combat.agility_buff.1.saturating_sub(1);
        if entity_combat.agility_buff.1 == 0 {
            entity_combat.agility_buff.0 = 0;
        }
        entity_combat.luck_buff.1 = entity_combat.luck_buff.1.saturating_sub(1);
        if entity_combat.luck_buff.1 == 0 {
            entity_combat.luck_buff.0 = 0;
        }

        // Tick debuff timers
        entity_combat.strength_debuff.1 = entity_combat.strength_debuff.1.saturating_sub(1);
        if entity_combat.strength_debuff.1 == 0 {
            entity_combat.strength_debuff.0 = 0;
        }
        entity_combat.focus_debuff.1 = entity_combat.focus_debuff.1.saturating_sub(1);
        if entity_combat.focus_debuff.1 == 0 {
            entity_combat.focus_debuff.0 = 0;
        }
        entity_combat.agility_debuff.1 = entity_combat.agility_debuff.1.saturating_sub(1);
        if entity_combat.agility_debuff.1 == 0 {
            entity_combat.agility_debuff.0 = 0;
        }
        entity_combat.luck_debuff.1 = entity_combat.luck_debuff.1.saturating_sub(1);
        if entity_combat.luck_debuff.1 == 0 {
            entity_combat.luck_debuff.0 = 0;
        }
        entity_combat.burn_debuff.1 = entity_combat.burn_debuff.1.saturating_sub(1);
        if entity_combat.burn_debuff.1 == 0 {
            entity_combat.burn_debuff.0 = 0;
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct DamageInfo {
    pub target: Entity,
    pub damage_amount: u32,
    pub damage_type: DamageType,
    pub variance: bool,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum DamageType {
    None,
    Strength,
    Focus,
    Agility,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Room {
    pub center: PositionComponent,
    pub x_radius: u32,
    pub y_radius: u32,
}

pub trait Animation {
    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        textures: &mut HashMap<String, Texture>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
    );
    fn entities_not_to_render(&self) -> HashSet<Entity>;
    fn is_complete(&self) -> bool;
}

struct DamageAnimation {
    time_started: Option<Instant>,
    damaged_entity: Entity,
    entity_position: PositionComponent,
    entity_sprite: &'static str,
    damage_dealt: u32,
    critical_hit: bool,
}

impl Animation for DamageAnimation {
    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        textures: &mut HashMap<String, Texture>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
    ) {
        if self.time_started == None {
            self.time_started = Some(Instant::now());
        }

        let frame_number =
            ((self.time_started.unwrap().elapsed().as_millis().min(400) as f64 / 400.0) * 4.0)
                .floor() as i32;
        let opacity = if frame_number % 2 == 0 { 127 } else { 255 };
        textures
            .entry(self.entity_sprite.to_owned())
            .and_modify(|texture| texture.set_alpha_mod(opacity));
        canvas
            .copy(
                &textures[self.entity_sprite],
                None,
                Rect::new(
                    self.entity_position.x * 32,
                    self.entity_position.y * 32,
                    32,
                    32,
                ),
            )
            .unwrap();
        textures
            .entry(self.entity_sprite.to_owned())
            .and_modify(|texture| texture.set_alpha_mod(255));

        if self.critical_hit {
            let frame_number =
                ((self.time_started.unwrap().elapsed().as_millis().min(400) as f64 / 400.0) * 6.0)
                    .floor() as i32;
            canvas
                .copy(
                    &textures["critical_hit_animation"],
                    Rect::new(24 * frame_number, 0, 24, 24),
                    Rect::new(
                        self.entity_position.x * 32 - 32,
                        self.entity_position.y * 32 - 32,
                        96,
                        96,
                    ),
                )
                .unwrap();
        }

        let surface = font
            .render(&format!("-{}hp", self.damage_dealt))
            .blended(Color::RGBA(0, 0, 0, 255))
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let texture_info = texture.query();
        let dest_rect = Rect::new(
            self.entity_position.x * 32 + (32 - texture_info.width as i32) / 2 + 1,
            self.entity_position.y * 32 - 20 + 2,
            texture_info.width,
            texture_info.height,
        );
        canvas.copy(&texture, None, dest_rect).unwrap();
        let surface = font
            .render(&format!("-{}hp", self.damage_dealt))
            .blended(Color::RGBA(255, 255, 255, 255))
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let texture_info = texture.query();
        let dest_rect = Rect::new(
            self.entity_position.x * 32 + (32 - texture_info.width as i32) / 2 - 1,
            self.entity_position.y * 32 - 20,
            texture_info.width,
            texture_info.height,
        );
        canvas.copy(&texture, None, dest_rect).unwrap();
    }

    fn entities_not_to_render(&self) -> HashSet<Entity> {
        let mut entities_not_to_render = HashSet::with_capacity(1);
        entities_not_to_render.insert(self.damaged_entity);
        entities_not_to_render
    }

    fn is_complete(&self) -> bool {
        self.time_started.unwrap().elapsed() >= Duration::from_millis(400)
    }
}

struct MissAnimation {
    time_started: Option<Instant>,
    entity_position: PositionComponent,
}

impl Animation for MissAnimation {
    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        _: &mut HashMap<String, Texture>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
    ) {
        if self.time_started == None {
            self.time_started = Some(Instant::now());
        }

        let surface = font
            .render("Dodged!")
            .blended(Color::RGBA(254, 174, 52, 255))
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let texture_info = texture.query();
        let dest_rect = Rect::new(
            self.entity_position.x * 32 + (32 - texture_info.width as i32) / 2 - 1,
            self.entity_position.y * 32 - 20,
            texture_info.width,
            texture_info.height,
        );
        canvas.copy(&texture, None, dest_rect).unwrap();
        let surface = font
            .render("Dodged!")
            .blended(Color::RGBA(140, 123, 39, 255))
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let texture_info = texture.query();
        let dest_rect = Rect::new(
            self.entity_position.x * 32 + (32 - texture_info.width as i32) / 2 + 1,
            self.entity_position.y * 32 - 20 + 2,
            texture_info.width,
            texture_info.height,
        );
        canvas.copy(&texture, None, dest_rect).unwrap();
    }

    fn entities_not_to_render(&self) -> HashSet<Entity> {
        HashSet::with_capacity(0)
    }

    fn is_complete(&self) -> bool {
        self.time_started.unwrap().elapsed() >= Duration::from_millis(400)
    }
}

struct NextFloorAnimation {
    time_started: Option<Instant>,
    floor_number: u32,
}

impl Animation for NextFloorAnimation {
    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        _: &mut HashMap<String, Texture>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
    ) {
        if self.time_started == None {
            self.time_started = Some(Instant::now());
        }

        let t = self.time_started.unwrap().elapsed().as_secs_f64().min(1.0);
        let opacity = if t > 0.4 {
            (((1.0 - t) * 255.0).round() as u8).max(1)
        } else {
            255
        };

        let surface = font
            .render(&format!("Floor {}", self.floor_number))
            .blended(Color::RGBA(0, 0, 0, opacity))
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let mut texture_info = texture.query();
        texture_info.width *= 3;
        texture_info.height *= 3;
        let dest_rect = Rect::new(
            (480 - texture_info.width as i32) / 2 + 1,
            (480 - texture_info.height as i32) / 4 + 2,
            texture_info.width,
            texture_info.height,
        );
        canvas.copy(&texture, None, dest_rect).unwrap();
        let surface = font
            .render(&format!("Floor {}", self.floor_number))
            .blended(Color::RGBA(255, 255, 255, opacity))
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let mut texture_info = texture.query();
        texture_info.width *= 3;
        texture_info.height *= 3;
        let dest_rect = Rect::new(
            (480 - texture_info.width as i32) / 2 - 1,
            (480 - texture_info.height as i32) / 4,
            texture_info.width,
            texture_info.height,
        );
        canvas.copy(&texture, None, dest_rect).unwrap();
    }

    fn entities_not_to_render(&self) -> HashSet<Entity> {
        HashSet::with_capacity(0)
    }

    fn is_complete(&self) -> bool {
        self.time_started.unwrap().elapsed() >= Duration::from_secs(1)
    }
}

use crate::data::*;
use crate::generate_dungeon::GenerateDungeonSystem;
use crate::movement::try_move;
use specs::{Join, World, WorldExt};

pub struct PlayerControllerSystem {
    pub action: PlayerAction,
}

impl PlayerControllerSystem {
    pub fn new() -> Self {
        Self {
            action: PlayerAction::None,
        }
    }

    pub fn run(
        &mut self,
        generate_dungeon_system: &mut GenerateDungeonSystem,
        world: &mut World,
    ) -> PlayerActed {
        let (player, player_entity) = {
            let player_data = world.read_storage::<Player>();
            let entities = world.entities();
            let (player, player_entity) = (&player_data, &entities).join().next().unwrap();
            (*player, player_entity)
        };

        let player_acted = match self.action {
            PlayerAction::None => false,
            PlayerAction::Pass => true,
            PlayerAction::Interact => {
                let mut end_turn = false;

                if let Some(item_entity) = {
                    let entities = world.entities();
                    let position_data = world.read_storage::<Position>();
                    let item_data = world.read_storage::<Item>();
                    let player_position = position_data.get(player_entity).unwrap();
                    let new_position = player_position.offset_by(player.facing_direction);
                    (&entities, &position_data, &item_data)
                        .join()
                        .find(|(_, position, _)| {
                            position == &&new_position
                                && ((player_position.x - position.x != 0)
                                    != (player_position.y - position.y != 0))
                        })
                        .map(|(entity, _, _)| entity)
                } {
                    let mut message_log = world.fetch_mut::<MessageLog>();
                    let mut player_data = world.write_storage::<Player>();
                    let mut position_data = world.write_storage::<Position>();
                    let name_data = world.read_storage::<Name>();
                    let player = player_data.get_mut(player_entity).unwrap();
                    for item_slot in player.inventory.iter_mut() {
                        if *item_slot == None {
                            position_data.remove(item_entity);
                            *item_slot = Some(item_entity);
                            let item_name = name_data.get(item_entity).unwrap();
                            message_log.new_message(
                                format!("You picked up: {}", item_name.get_text()),
                                MessageColor::White,
                                MessageDisplayLength::Medium,
                            );
                            end_turn = true;
                            break;
                        }
                    }
                };

                let is_facing_staircase = {
                    let position_data = world.read_storage::<Position>();
                    let staircase_data = world.read_storage::<Staircase>();
                    let player_position = position_data.get(player_entity).unwrap();
                    let new_position = player_position.offset_by(player.facing_direction);
                    (&position_data, &staircase_data)
                        .join()
                        .any(|(position, _)| {
                            position == &new_position
                                && ((player_position.x - position.x != 0)
                                    != (player_position.y - position.y != 0))
                        })
                };
                if is_facing_staircase {
                    {
                        let mut attackable_data = world.write_storage::<Attackable>();
                        let player_attackable = attackable_data.get_mut(player_entity).unwrap();

                        player_attackable.current_health +=
                            (player_attackable.max_health as f64 * 0.2).round() as u32;
                        if player_attackable.current_health > player_attackable.max_health {
                            player_attackable.current_health = player_attackable.max_health;
                        }

                        player_attackable.oozed_stacks = 0;
                        player_attackable.cant_attack_turns = 0;
                        player_attackable.cant_move_turns = 0;
                        player_attackable.blight_stacks = 0;
                    }
                    generate_dungeon_system.run(world);
                    end_turn = true;
                }

                end_turn
            }
            PlayerAction::Turn(direction) => {
                let mut player_data = world.write_storage::<Player>();
                player_data.get_mut(player_entity).unwrap().facing_direction = direction;
                false
            }
            PlayerAction::Move(direction) => {
                {
                    let mut player_data = world.write_storage::<Player>();
                    player_data.get_mut(player_entity).unwrap().facing_direction = direction;
                }
                if try_move(player_entity, direction, world).is_ok() {
                    true
                } else {
                    self.action = PlayerAction::UseItem(ItemSlot::One);
                    false
                }
            }
            PlayerAction::UseItem(item_slot) => {
                let inventory_index = match item_slot {
                    ItemSlot::One => 0,
                    ItemSlot::Two => 1,
                    ItemSlot::Three => 2,
                    ItemSlot::Four => 3,
                };
                if let Some(item_entity) = player.inventory[inventory_index] {
                    let item = {
                        let item_data = world.read_storage::<Item>();
                        *item_data.get(item_entity).unwrap()
                    };
                    let item_result = (item.try_use)(item_entity, world);
                    if item_result.should_end_turn {
                        let mut player_data = world.write_storage::<Player>();
                        let player = player_data.get_mut(player_entity).unwrap();
                        player.crystals = player
                            .crystals
                            .checked_sub(item.crystals_per_use)
                            .unwrap_or(0);
                    }
                    if item_result.should_consume_item {
                        let _ = world.delete_entity(item_entity);
                        let mut player_data = world.write_storage::<Player>();
                        let player = player_data.get_mut(player_entity).unwrap();
                        player.inventory[inventory_index] = None;
                    }
                    item_result.should_end_turn
                } else {
                    false
                }
            }
        };

        if player_acted {
            self.action = PlayerAction::None;
            {
                let mut player_data = world.write_storage::<Player>();
                let mut player = player_data.get_mut(player_entity).unwrap();
                player.turns_taken += 1;

                if player.heal_turns_left == 0 {
                    let mut attackable_data = world.write_storage::<Attackable>();
                    let mut player_attackable = attackable_data.get_mut(player_entity).unwrap();
                    player_attackable.current_health += 1;
                    if player_attackable.current_health > player_attackable.max_health {
                        player_attackable.current_health = player_attackable.max_health;
                    }
                    player.heal_turns_left = 10;
                } else {
                    player.heal_turns_left -= 1;
                }
            }
            world.insert(GameState::EnemyTurn);
        }
        PlayerActed(player_acted)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum PlayerAction {
    None,
    Pass,
    Interact,
    Turn(Direction),
    Move(Direction),
    UseItem(ItemSlot),
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct PlayerActed(pub bool);

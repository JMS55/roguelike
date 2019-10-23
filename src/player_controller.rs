use crate::data::{Attackable, Direction, GameState, Item, ItemSlot, Player, Position, Staircase};
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
                    }
                    generate_dungeon_system.run(world);
                }
                is_facing_staircase
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
                try_move(player_entity, direction, world).is_ok()
            }
            PlayerAction::UseItem(item_slot) => {
                let inventory_index = match item_slot {
                    ItemSlot::One => 0,
                    ItemSlot::Two => 1,
                    ItemSlot::Three => 2,
                    ItemSlot::Four => 3,
                };
                if let Some(item) = player.inventory[inventory_index] {
                    let item = {
                        let item_data = world.read_storage::<Item>();
                        *item_data.get(item).unwrap()
                    };
                    let attack_succeeded = (item.try_use)(world).is_ok();
                    if attack_succeeded {
                        let mut player_data = world.write_storage::<Player>();
                        let player = player_data.get_mut(player_entity).unwrap();
                        player.crystals = player
                            .crystals
                            .checked_sub(item.crystals_per_use)
                            .unwrap_or(0);
                    }
                    attack_succeeded
                } else {
                    false
                }
            }
        };
        self.action = PlayerAction::None;

        if player_acted {
            {
                let mut player_data = world.write_storage::<Player>();
                player_data.get_mut(player_entity).unwrap().turns_taken += 1;
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

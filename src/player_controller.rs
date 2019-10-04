use crate::data::{Direction, GameState, Player, Position, Staircase};
use crate::generate_dungeon::GenerateDungeonSystem;
use crate::movement;
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
        let player_entity = {
            let entities = world.entities();
            let player_data = world.read_storage::<Player>();
            (&entities, &player_data).join().next().unwrap().0
        };

        let player_acted = match self.action {
            PlayerAction::None => false,
            PlayerAction::Pass => true,
            PlayerAction::Interact => {
                let is_facing_staircase = {
                    let position_data = world.read_storage::<Position>();
                    let staircase_data = world.read_storage::<Staircase>();
                    let player_position = position_data.get(player_entity).unwrap();
                    let (new_x, new_y) = match player_position.facing_direction {
                        Direction::Up => (player_position.x, player_position.y + 1),
                        Direction::Down => (player_position.x, player_position.y - 1),
                        Direction::Left => (player_position.x - 1, player_position.y),
                        Direction::Right => (player_position.x + 1, player_position.y),
                    };
                    (&position_data, &staircase_data)
                        .join()
                        .any(|(position, _)| position.x == new_x && position.y == new_y)
                };
                if is_facing_staircase {
                    generate_dungeon_system.run(world);
                }
                is_facing_staircase
            }
            PlayerAction::Turn(direction) => {
                let _ = movement::try_turn(player_entity, direction, world);
                false
            }
            PlayerAction::Move(direction) => {
                movement::try_move(player_entity, direction, world).is_ok()
            }
        };

        if player_acted {
            {
                let mut player_data = world.write_storage::<Player>();
                player_data.get_mut(player_entity).unwrap().turns_taken += 1;
            }
            world.insert(GameState::EnemyTurn);
        }

        self.action = PlayerAction::None;
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
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct PlayerActed(pub bool);

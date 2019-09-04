use crate::components::PlayerComponent;
use specs::{Join, System, WriteStorage};

pub struct DrainCrystalsSystem {
    previous_turns_taken: u32,
}

impl DrainCrystalsSystem {
    pub fn new() -> Self {
        Self {
            previous_turns_taken: 0,
        }
    }
}

impl<'s> System<'s> for DrainCrystalsSystem {
    type SystemData = WriteStorage<'s, PlayerComponent>;

    fn run(&mut self, mut player_data: Self::SystemData) {
        let player = (&mut player_data).join().next().unwrap();
        if self.previous_turns_taken != player.turns_taken {
            let crystals_to_subtract = match player.turns_taken {
                0..=500 => 0,
                501..=601 => 2,
                602..=702 => 5,
                703..=903 => 10,
                _ => 20,
            };
            player.crystals = player
                .crystals
                .checked_sub(crystals_to_subtract)
                .unwrap_or(0);
            self.previous_turns_taken += 1;
        }
    }
}

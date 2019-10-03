use crate::data::{MessageColor, MessageLog, Player};
use specs::{Join, World, WorldExt};

pub fn drain_crystals_system(world: &mut World) {
    let mut player_data = world.write_storage::<Player>();
    let mut player = (&mut player_data).join().next().unwrap();
    let mut message_log = world.fetch_mut::<MessageLog>();

    if let Some((message, color)) = match player.turns_taken {
            500 => {
                Some(("You feel a sense of... unease. Perhaps you should consider leaving soon...", MessageColor::White))
            }
            600 => Some(("The sense of danger grows. Fatigue starts to overcome your body. You must leave before it's too late!", MessageColor::Orange)),
            700 => Some(("YOUR INSTINCTS SCREAM TO RUN. YOUR BODY GROWS HEAVY WITH DESPAIR.", MessageColor::Red)),
            _ => None,
        } {
            message_log.new_message(message, color);
        }

    let crystals_to_subtract = match player.turns_taken {
        0..=499 => 0,
        500..=599 => 2,
        600..=699 => 5,
        _ => 10,
    };
    player.crystals = player
        .crystals
        .checked_sub(crystals_to_subtract)
        .unwrap_or(0);
}

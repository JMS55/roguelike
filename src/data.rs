use crate::items;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use specs::storage::BTreeStorage;
use specs::{Component, Entity, World};
use specs_derive::Component;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Name {
    pub text: &'static str,
    pub concealed: bool,
}

impl Name {
    pub fn new(text: &'static str, concealed: bool) -> Self {
        Self { text, concealed }
    }

    pub fn get_text(&self) -> &'static str {
        if self.concealed {
            "???"
        } else {
            self.text
        }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn distance_from(self, other: Self) -> u32 {
        (other.x - self.x).abs() as u32 + (other.y - self.y).abs() as u32
    }

    pub fn offset_by(self, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self::new(self.x, self.y + 1),
            Direction::Down => Self::new(self.x, self.y - 1),
            Direction::Left => Self::new(self.x - 1, self.y),
            Direction::Right => Self::new(self.x + 1, self.y),
            Direction::UpLeft => Self::new(self.x - 1, self.y + 1),
            Direction::DownLeft => Self::new(self.x - 1, self.y - 1),
            Direction::DownRight => Self::new(self.x + 1, self.y - 1),
            Direction::UpRight => Self::new(self.x + 1, self.y + 1),
        }
    }

    pub fn neighbors(self, obstacles: &HashSet<Self>) -> Vec<Position> {
        let mut neighbors = Vec::with_capacity(4);
        for (x_offset, y_offset) in &[
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let mut new_position = self;
            new_position.x += x_offset;
            new_position.y += y_offset;
            if !obstacles.contains(&new_position) {
                neighbors.push(new_position);
            }
        }
        neighbors
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Sprite {
    pub id: &'static str,
    pub double_sized: bool,
    pub in_foreground: bool,
}

impl Sprite {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            double_sized: false,
            in_foreground: true,
        }
    }
}

#[derive(Component, Clone)]
#[storage(BTreeStorage)]
pub struct Attackable {
    pub current_health: u32,
    pub max_health: u32,

    pub is_boss: bool,
    pub crystals_dropped_on_death: u32,
    pub item_dropped_on_death: Option<fn(Option<Position>, &mut World) -> Entity>,

    pub is_oozing: bool,
    pub explode_on_death: (u32, u32),  // Damage, Radius
    pub lower_spawn_times: (f32, u32), // Health percent threshold, Turns to lower by
    pub is_magic_immune: bool,

    pub oozed_stacks: u32,
    pub cant_attack_turns: u32,
    pub cant_move_turns: u32,
    pub blight_stacks: u32,
}

impl Attackable {
    pub fn new(
        max_health: u32,
        crystals_dropped_on_death: u32,
        item_dropped_on_death: Option<fn(Option<Position>, &mut World) -> Entity>,
        is_boss: bool,
    ) -> Self {
        Self {
            current_health: max_health,
            max_health,

            is_boss,
            crystals_dropped_on_death,
            item_dropped_on_death,

            is_oozing: false,
            explode_on_death: (0, 0),
            lower_spawn_times: (0.0, 0),
            is_magic_immune: false,

            oozed_stacks: 0,
            cant_attack_turns: 0,
            cant_move_turns: 0,
            blight_stacks: 0,
        }
    }
}

#[derive(Component, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct AI {
    pub run: fn(Entity, &mut World),
}

impl AI {
    pub fn new(run: fn(Entity, &mut World)) -> Self {
        Self { run }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Counter(pub u32);

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Intangible {}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Player {
    pub facing_direction: Direction,
    pub crystals: u32,
    pub inventory: [Option<Entity>; 16],
    pub turns_taken: u32,
    pub heal_turns_left: u32,
}

impl Player {
    pub fn new(world: &mut World) -> Self {
        let mut inventory = [None; 16];
        inventory[0] = Some(items::create_makeshift_dagger(None, world));
        Self {
            facing_direction: Direction::Up,
            crystals: 200,
            inventory,
            turns_taken: 0,
            heal_turns_left: 10,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum ItemSlot {
    One,
    Two,
    Three,
    Four,
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Staircase {}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Spawner {
    turns_since_last_spawn: u32,
    pub turns_per_spawn: u32,
    pub spawn_concealed: bool,
}

impl Spawner {
    pub fn new() -> Self {
        let turns_per_spawn = 30;
        Self {
            turns_since_last_spawn: turns_per_spawn,
            turns_per_spawn,
            spawn_concealed: false,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.turns_since_last_spawn += 1;
        if self.turns_since_last_spawn >= self.turns_per_spawn {
            self.turns_since_last_spawn = 0;
            true
        } else {
            false
        }
    }
}

#[derive(Component, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Item {
    pub crystals_per_use: u32,
    pub try_use: fn(Entity, &mut World) -> ItemResult,
}

impl Item {
    pub fn new(crystals_per_use: u32, try_use: fn(Entity, &mut World) -> ItemResult) -> Self {
        Self {
            crystals_per_use,
            try_use,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct ItemResult {
    pub should_end_turn: bool,
    pub should_consume_item: bool,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum GameState {
    NewGame,
    PlayerTurn,
    EnemyTurn,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct MessageLog {
    messages: Vec<Message>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn new_message<T: Into<String>>(
        &mut self,
        message: T,
        color: MessageColor,
        display_length: MessageDisplayLength,
    ) {
        self.messages.push(Message {
            text: message.into(),
            color,
            display_length,
            time_created: Instant::now(),
        });
    }

    pub fn recent_messages(&mut self) -> impl Iterator<Item = &Message> {
        self.messages
            .retain(|message| message.time_created.elapsed() <= message.display_length.duration());
        self.messages.iter().rev()
    }

    pub fn empty(&mut self) {
        self.messages.clear();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Message {
    pub text: String,
    pub color: MessageColor,
    pub display_length: MessageDisplayLength,
    pub time_created: Instant,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum MessageColor {
    White,
    Orange,
    Red,
    Green,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum MessageDisplayLength {
    Short,
    Medium,
    Long,
}

impl MessageDisplayLength {
    pub fn duration(self) -> Duration {
        Duration::from_secs(match self {
            MessageDisplayLength::Short => 3,
            MessageDisplayLength::Medium => 4,
            MessageDisplayLength::Long => 6,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct ScrollInfo {
    pub scroll_of_shadows_sprite: &'static str,
    pub scroll_of_shadows_identified: bool,
    pub scroll_of_displacement_sprite: &'static str,
    pub scroll_of_displacement_identified: bool,
    pub scroll_of_entanglement_sprite: &'static str,
    pub scroll_of_entanglement_identified: bool,
    pub scroll_of_lightning_sprite: &'static str,
    pub scroll_of_lightning_identified: bool,
}

impl ScrollInfo {
    pub fn new(rng: &mut RNG) -> Self {
        let mut colors = vec![
            "scroll_red",
            "scroll_orange",
            "scroll_yellow",
            "scroll_green",
            "scroll_cyan",
            "scroll_blue",
            "scroll_purple",
            "scroll_black",
        ];
        colors.shuffle(&mut rng.0);
        Self {
            scroll_of_shadows_sprite: colors.pop().unwrap(),
            scroll_of_shadows_identified: false,
            scroll_of_displacement_sprite: colors.pop().unwrap(),
            scroll_of_displacement_identified: false,
            scroll_of_entanglement_sprite: colors.pop().unwrap(),
            scroll_of_entanglement_identified: false,
            scroll_of_lightning_sprite: colors.pop().unwrap(),
            scroll_of_lightning_identified: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RNG(pub Pcg64);

impl RNG {
    pub fn new() -> Self {
        Self(Pcg64::from_entropy())
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    DownLeft,
    DownRight,
    UpRight,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

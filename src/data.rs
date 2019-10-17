use rand::SeedableRng;
use rand_pcg::Pcg64;
use specs::storage::BTreeStorage;
use specs::{Component, Entity, World};
use specs_derive::Component;
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Name(pub &'static str);

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
    pub in_foreground: bool,
}

impl Sprite {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            in_foreground: true,
        }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
#[storage(BTreeStorage)]
pub struct Attackable {
    pub current_health: u32,
    pub max_health: u32,

    pub is_boss: bool,
    pub is_oozing: bool,
    pub explode_on_death: (u32, u32),

    pub oozed_stacks: u32,
}

impl Attackable {
    pub fn new(max_health: u32, is_boss: bool) -> Self {
        Self {
            current_health: max_health,
            max_health,

            is_boss,
            is_oozing: false,
            explode_on_death: (0, 0),

            oozed_stacks: 0,
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
pub struct AICounter(pub u32);

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Intangible {}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Player {
    pub facing_direction: Direction,
    pub crystals: u32,
    pub turns_taken: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            facing_direction: Direction::Up,
            crystals: 500,
            turns_taken: 0,
        }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Staircase {}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Spawner {
    turns_since_last_spawn: u32,
    turns_per_spawn: u32,
}

impl Spawner {
    pub fn new() -> Self {
        let turns_per_spawn = 30;
        Self {
            turns_since_last_spawn: turns_per_spawn,
            turns_per_spawn,
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

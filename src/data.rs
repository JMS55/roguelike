use rand::SeedableRng;
use rand_pcg::Pcg64;
use specs::storage::BTreeStorage;
use specs::{Component, Entity, World};
use specs_derive::Component;
use std::time::{Duration, Instant};

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Name(pub &'static str);

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub facing_direction: Direction,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            facing_direction: Direction::Right,
        }
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Sprite {
    pub id: &'static str,
}

impl Sprite {
    pub fn new(id: &'static str) -> Self {
        Self { id }
    }
}

#[derive(Component, Clone)]
#[storage(BTreeStorage)]
pub struct Attackable {
    pub current_health: u32,
    pub max_health: u32,
    pub on_death: Option<fn(Entity, Option<Entity>, &mut World)>,

    pub has_oozing_buff: bool,

    pub oozed_debuff_stacks: u32,
}

impl Attackable {
    pub fn new(max_health: u32) -> Self {
        Self {
            current_health: max_health,
            max_health,
            on_death: None,

            has_oozing_buff: false,

            oozed_debuff_stacks: 0,
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
pub struct Boolean(pub bool);

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Intangible {}

#[derive(Component, Debug, Hash, PartialEq, Eq, Copy, Clone)]
#[storage(BTreeStorage)]
pub struct Player {
    pub crystals: u32,
    pub turns_taken: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
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
    time_since_last_spawn: Duration,
    cooldown_time: Duration,
}

impl Spawner {
    pub fn new() -> Self {
        let cooldown_time = Duration::from_secs(20);
        Self {
            time_since_last_spawn: cooldown_time,
            cooldown_time,
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
    pub fn duration(&self) -> Duration {
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
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

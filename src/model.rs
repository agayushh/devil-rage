//! Shared types, tuning, and the on-disk save.

use std::fs;
use std::path::PathBuf;

pub const GRAVITY: f32 = 36.0;
pub const JUMP_V: f32 = 13.0;
pub const MOVE_SPEED: f32 = 9.0;
pub const ACCEL: f32 = 85.0;
pub const FRICTION: f32 = 110.0;
pub const ICE_ACCEL: f32 = 14.0;
pub const ICE_FRICTION: f32 = 3.2;
pub const MAX_FALL: f32 = 22.0;
pub const COYOTE: f32 = 0.09;
pub const JUMP_BUFFER: f32 = 0.10;
pub const SKIP_AFTER: u32 = 4;
pub const SAVE_VER: u32 = 1;

pub const KEY_TOTAL: u8 = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    None,
    Spike,
    Fall,
    Saw,
    Shot,
    Crush,
    Coin,
    FakeDoor,
    Clone,
    Chase,
    Bomb,
    Button,
}

impl Cause {
    pub fn quip(self, salt: u32) -> &'static str {
        let i = salt as usize;
        match self {
            Cause::Spike => ["the floor grew teeth", "that tile was hungry", "spikes file no warnings"][i % 3],
            Cause::Fall => ["the floor was a rumor", "gravity kept the receipt", "down is a door too"][i % 3],
            Cause::Saw => ["the saw liked your timing", "you auditioned for the blender", "round and round and dead"][i % 3],
            Cause::Shot => ["something in the air disagreed", "lead is not a collectible", "you lost the argument"][i % 3],
            Cause::Crush => ["the room got smaller", "personal space: denied", "a wall with ambition"][i % 3],
            Cause::Coin => ["that coin had a fuse", "greed is a hitbox", "it was never currency"][i % 3],
            Cause::FakeDoor => ["wrong door", "the exit pays rent to the spikes", "doors can lie with a straight face"][i % 3],
            Cause::Clone => ["your shadow went first", "you were not the one who died", "company policy: shared graves"][i % 3],
            Cause::Chase => ["hesitation is edible", "it does not get tired", "you stopped. it didn't"][i % 3],
            Cause::Bomb => ["sideways is still down", "the bomb had opinions", "catch of the day"][i % 3],
            Cause::Button => ["bold of you", "that button was a confession", "curiosity, billed in full"][i % 3],
            Cause::None => ["dead", "again", "the devil shrugs"][i % 3],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Normal,
    Small,
    Wide,
    Tall,
}

pub fn dims(p: Profile) -> (f32, f32) {
    match p {
        Profile::Normal => (0.56, 0.84),
        Profile::Small => (0.34, 0.55),
        Profile::Wide => (2.15, 0.84),
        Profile::Tall => (0.56, 1.62),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloneKind {
    Mirror,
    Opposite,
    Shadow,
    Dual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Air,
    Solid,
    Fake,
    Ghost,
    Flicker,
    Spike,
    Crumble,
    Ice,
    Spring,
    Super,
    OneWay,
    FakeDoor,
    Coin,
    Bomb,
    Key,
    Portal,
    Grow,
    Shrink,
    TallBtn,
    NormBtn,
    Pvp,
    GunR,
    GunL,
    BombGun,
}

impl Kind {
    pub fn blocks(self, flicker_on: bool) -> bool {
        match self {
            Kind::Solid
            | Kind::Ghost
            | Kind::Crumble
            | Kind::Ice
            | Kind::Spring
            | Kind::Super
            | Kind::GunR
            | Kind::GunL
            | Kind::BombGun => true,
            Kind::Flicker => flicker_on,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub kind: Kind,
    pub meta: i16,
    pub timer: f32,
}

impl Cell {
    pub fn air() -> Self {
        Self { kind: Kind::Air, meta: 0, timer: -1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Save {
    pub world: u8,
    pub door: u8,
    pub stage: u8,
    pub keys: Vec<u8>,
    pub secret: bool,
    pub secret_stage: u8,
    pub deaths: u32,
    pub skips: u32,
    pub hints: bool,
    pub bell: bool,
    pub ascii: bool,
    pub cleared: bool,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            world: 0,
            door: 0,
            stage: 0,
            keys: Vec::new(),
            secret: false,
            secret_stage: 0,
            deaths: 0,
            skips: 0,
            hints: true,
            bell: false,
            ascii: false,
            cleared: false,
        }
    }
}

impl Save {
    pub fn path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local/share/level-devil/save.txt")
        } else {
            PathBuf::from("level-devil-save.txt")
        }
    }

    pub fn load() -> Self {
        let path = Self::path();
        match fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn write(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(path, self.encode());
    }

    pub fn encode(&self) -> String {
        let keys: Vec<String> = self.keys.iter().map(|k| k.to_string()).collect();
        format!(
            "ver={SAVE_VER}\nworld={}\ndoor={}\nstage={}\nkeys={}\nsecret={}\nsecret_stage={}\ndeaths={}\nskips={}\nhints={}\nbell={}\nascii={}\ncleared={}\n",
            self.world,
            self.door,
            self.stage,
            keys.join(","),
            u8::from(self.secret),
            self.secret_stage,
            self.deaths,
            self.skips,
            u8::from(self.hints),
            u8::from(self.bell),
            u8::from(self.ascii),
            u8::from(self.cleared),
        )
    }

    pub fn parse(text: &str) -> Option<Self> {
        let mut s = Self::default();
        let mut ver = 0u32;
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            match k.trim() {
                "ver" => ver = v.trim().parse().ok()?,
                "world" => s.world = v.trim().parse().ok()?,
                "door" => s.door = v.trim().parse().ok()?,
                "stage" => s.stage = v.trim().parse().ok()?,
                "keys" => {
                    s.keys = if v.trim().is_empty() {
                        Vec::new()
                    } else {
                        v.trim().split(',').filter_map(|n| n.trim().parse().ok()).collect()
                    };
                }
                "secret" => s.secret = v.trim() == "1",
                "secret_stage" => s.secret_stage = v.trim().parse().ok()?,
                "deaths" => s.deaths = v.trim().parse().ok()?,
                "skips" => s.skips = v.trim().parse().ok()?,
                "hints" => s.hints = v.trim() != "0",
                "bell" => s.bell = v.trim() == "1",
                "ascii" => s.ascii = v.trim() == "1",
                "cleared" => s.cleared = v.trim() == "1",
                _ => {}
            }
        }
        if ver != SAVE_VER {
            return None;
        }
        s.keys.sort_unstable();
        s.keys.dedup();
        Some(s)
    }

    pub fn give_key(&mut self, id: u8) -> bool {
        if self.keys.contains(&id) {
            return false;
        }
        self.keys.push(id);
        self.keys.sort_unstable();
        if self.keys.len() >= KEY_TOTAL as usize {
            self.secret = true;
        }
        true
    }

    pub fn unlocked(&self, world: u8, door: u8) -> bool {
        if self.cleared || self.world >= 3 {
            return world < 3;
        }
        if world < self.world {
            return true;
        }
        world == self.world && door <= self.door
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Stage,
    Door,
    World,
    Done,
}

pub fn title_line(t: f32) -> &'static str {
    const LINES: &[&str] = &[
        "the door is lying",
        "walk like you mean it. then don't.",
        "coins remember your face",
        "gravity is a suggestion with teeth",
        "your shadow has worse ideas",
        "every floor filed a complaint",
    ];
    LINES[(t as usize / 3) % LINES.len()]
}

//! Screens and the play loop. Input is keys; the browser paints the result.

use std::collections::HashSet;

use crate::campaign::{self, SECRET, WORLDS};
use crate::model::*;
use crate::sim::{Sim, Stick, Sticks};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    A,
    D,
    W,
    S,
    Enter,
    Esc,
    Space,
    R,
    P,
    Q,
    K,
    J,
}

struct Keys {
    down: HashSet<Key>,
    edges: Vec<Key>,
}

impl Keys {
    fn new() -> Self {
        Self { down: HashSet::new(), edges: Vec::new() }
    }
    fn held(&self, key: Key) -> bool {
        self.down.contains(&key)
    }
    fn take_edges(&mut self) -> Vec<Key> {
        std::mem::take(&mut self.edges)
    }
    fn eat(&mut self) {
        self.edges.clear();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Intro,
    Run,
    Dead,
    Clear,
    Pause,
}

pub struct Play {
    pub world: u8,
    pub door: u8,
    pub stage: u8,
    pub follow: bool,
    pub secret: bool,
    pub deaths: u32,
    pub phase: Phase,
    pub phase_t: f32,
    pub pause_sel: usize,
    pub announce: String,
    pub sub: String,
    pub end_secret: bool,
    pub go_map: bool,
    pub go_end: bool,
}

pub enum Scene {
    Title { sel: usize },
    Settings { sel: usize },
    Map { world: usize, door: usize, secret: bool },
    Play(Play),
    Ending { secret: bool },
}

pub struct Game {
    pub save: Save,
    pub scene: Scene,
    pub sim: Option<Sim>,
    pub time: f32,
    pub flash: f32,
    pub shake: f32,
    pub toast: String,
    pub toast_t: f32,
    /// Set when the death bell should ring. The page clears it after playing.
    pub chime: bool,
    keys: Keys,
    code: Vec<Key>,
}

const CODE: [Key; 8] = [Key::Up, Key::Up, Key::Down, Key::Down, Key::Left, Key::Right, Key::Left, Key::Right];

impl Game {
    pub fn boot() -> Self {
        Self::from_save(Save::load())
    }

    pub fn from_save(save: Save) -> Self {
        Self {
            save,
            scene: Scene::Title { sel: 0 },
            keys: Keys::new(),
            sim: None,
            time: 0.0,
            flash: 0.0,
            shake: 0.0,
            toast: String::new(),
            toast_t: 0.0,
            chime: false,
            code: Vec::new(),
        }
    }

    pub fn on_key(&mut self, key: Key, down: bool, repeat: bool) {
        if down {
            let fresh = !self.keys.down.contains(&key);
            self.keys.down.insert(key);
            if fresh && !repeat {
                self.keys.edges.push(key);
            }
        } else {
            self.keys.down.remove(&key);
        }
    }

    pub fn tap(&mut self, key: Key) {
        self.keys.edges.push(key);
    }

    pub fn release_all(&mut self) {
        self.keys.down.clear();
    }

    pub fn pick_title(&mut self, sel: usize) {
        if let Scene::Title { sel: s } = &mut self.scene {
            *s = sel;
        }
        self.tap(Key::Enter);
    }

    pub fn pick_settings(&mut self, sel: usize) {
        if let Scene::Settings { sel: s } = &mut self.scene {
            *s = sel;
        }
        self.tap(Key::Enter);
    }

    pub fn pick_door(&mut self, world: usize, door: usize) {
        if let Scene::Map { world: w, door: d, secret } = &mut self.scene {
            *w = world;
            *d = door;
            *secret = false;
        }
        self.tap(Key::Enter);
    }

    pub fn pick_secret(&mut self) {
        if let Scene::Map { secret, .. } = &mut self.scene {
            *secret = true;
        }
        self.tap(Key::Enter);
    }

    pub fn pick_pause(&mut self, sel: usize) {
        if let Scene::Play(p) = &mut self.scene {
            p.pause_sel = sel;
        }
        self.tap(Key::Enter);
    }

    pub fn tick(&mut self, dt: f32) {
        self.time += dt;
        self.flash = (self.flash - dt).max(0.0);
        self.shake = (self.shake - dt).max(0.0);
        self.toast_t = (self.toast_t - dt).max(0.0);
        let edges = self.keys.take_edges();
        match &self.scene {
            Scene::Title { .. } => self.tick_title(&edges),
            Scene::Settings { .. } => self.tick_settings(&edges),
            Scene::Map { .. } => self.tick_map(&edges),
            Scene::Play(_) => self.tick_play(dt, &edges),
            Scene::Ending { .. } => {
                if pressed(&edges, &[Key::Enter, Key::Esc, Key::Space, Key::Q]) {
                    self.scene = Scene::Map { world: 0, door: 0, secret: false };
                }
            }
        }
    }

    pub fn title_items(&self) -> Vec<&'static str> {
        let fresh = self.save.world == 0
            && self.save.door == 0
            && self.save.stage == 0
            && !self.save.cleared
            && self.save.deaths == 0;
        if fresh {
            vec!["Play", "World map", "Settings"]
        } else {
            vec!["Continue", "World map", "Settings"]
        }
    }

    pub fn status_line(&self) -> String {
        match &self.scene {
            Scene::Title { sel } => {
                let items = self.title_items();
                format!("Level Devil. {}", items.get(*sel).copied().unwrap_or("Play"))
            }
            Scene::Settings { sel } => {
                let row = ["hints", "death bell", "plain shapes", "true door", "back"]
                    .get(*sel)
                    .copied()
                    .unwrap_or("");
                format!("Settings. {row}")
            }
            Scene::Map { world, door, secret } => {
                if *secret {
                    "World map. True door".into()
                } else {
                    let info = WORLDS.get(*world);
                    let name = info.map(|w| w.name).unwrap_or("World");
                    let tag = info.map(|w| w.tag).unwrap_or("");
                    let d = info.and_then(|w| w.doors.get(*door)).copied().unwrap_or("door");
                    format!("World map. {name}, {d}. {tag}")
                }
            }
            Scene::Ending { secret } => {
                if *secret { "True ending.".into() } else { "Ending.".into() }
            }
            Scene::Play(p) => {
                let name = self.sim.as_ref().map(|s| s.name).unwrap_or("Stage");
                match p.phase {
                    Phase::Intro => {
                        let hint = self.sim.as_ref().map(|s| s.hint).unwrap_or("");
                        format!("{name}. {hint}")
                    }
                    Phase::Run => name.into(),
                    Phase::Dead => {
                        let q = self
                            .sim
                            .as_ref()
                            .map(|s| s.cause.quip(self.save.deaths + p.deaths))
                            .unwrap_or("dead");
                        format!("Dead. {q}")
                    }
                    Phase::Clear => format!("{}. {}", p.announce, p.sub),
                    Phase::Pause => {
                        let o = ["resume", "retry", "map"].get(p.pause_sel).copied().unwrap_or("resume");
                        format!("Paused. {o}")
                    }
                }
            }
        }
    }

    fn tick_title(&mut self, edges: &[Key]) {
        let n = self.title_items().len();
        let Scene::Title { sel } = &mut self.scene else { return };
        if pressed(edges, &[Key::Up, Key::W, Key::K]) {
            *sel = (*sel + n - 1) % n;
        }
        if pressed(edges, &[Key::Down, Key::S, Key::J]) {
            *sel = (*sel + 1) % n;
        }
        if pressed(edges, &[Key::Enter, Key::Space]) {
            let choice_i = *sel;
            let items = self.title_items();
            let choice = items.get(choice_i).copied().unwrap_or("Play");
            match choice {
                "Play" | "Continue" => {
                    if self.save.cleared || self.save.world >= 3 {
                        self.scene = Scene::Map { world: 2, door: 0, secret: false };
                    } else {
                        let w = self.save.world;
                        let d = self.save.door;
                        let s = self.save.stage;
                        self.begin_stage(w, d, s, true, false);
                    }
                }
                "World map" => {
                    self.scene = Scene::Map { world: self.save.world.min(2) as usize, door: 0, secret: false };
                }
                "Settings" => self.scene = Scene::Settings { sel: 0 },
                _ => {}
            }
        }
    }

    fn tick_settings(&mut self, edges: &[Key]) {
        for e in edges {
            if matches!(e, Key::Up | Key::Down | Key::Left | Key::Right) {
                self.code.push(*e);
                if self.code.len() > 8 {
                    self.code.remove(0);
                }
                if self.code.len() >= 8 && self.code[self.code.len() - 8..] == CODE {
                    self.save.secret = true;
                    self.save.write();
                    self.toast = "the true door answers".into();
                    self.toast_t = 2.4;
                    self.code.clear();
                }
            }
        }

        let Scene::Settings { sel } = &mut self.scene else { return };
        let n = 5;
        if pressed(edges, &[Key::Up, Key::W]) {
            *sel = (*sel + n - 1) % n;
        }
        if pressed(edges, &[Key::Down, Key::S]) {
            *sel = (*sel + 1) % n;
        }
        let sel = *sel;
        let toggle = pressed(edges, &[Key::Enter, Key::Left, Key::Right, Key::Space]);
        if toggle {
            match sel {
                0 => self.save.hints = !self.save.hints,
                1 => self.save.bell = !self.save.bell,
                2 => self.save.ascii = !self.save.ascii,
                3 => {
                    if self.save.secret {
                        self.save.write();
                        let stage = self.save.secret_stage.min(SECRET.len().saturating_sub(1) as u8);
                        self.begin_stage(9, 0, stage, false, true);
                        return;
                    } else {
                        self.toast = "still locked".into();
                        self.toast_t = 1.2;
                    }
                }
                4 => {
                    self.save.write();
                    self.scene = Scene::Title { sel: 0 };
                    return;
                }
                _ => {}
            }
            self.save.write();
        }
        if pressed(edges, &[Key::Esc, Key::Q]) {
            self.save.write();
            self.scene = Scene::Title { sel: 0 };
        }
    }

    fn tick_map(&mut self, edges: &[Key]) {
        let Scene::Map { world, door, secret } = &mut self.scene else { return };
        if pressed(edges, &[Key::Esc, Key::Q]) {
            self.scene = Scene::Title { sel: 0 };
            return;
        }
        if pressed(edges, &[Key::Up, Key::W]) {
            if *secret {
                *secret = false;
                *world = WORLDS.len() - 1;
            } else if *world > 0 {
                *world -= 1;
                *door = 0;
            }
        }
        if pressed(edges, &[Key::Down, Key::S]) {
            if *world + 1 < WORLDS.len() && !*secret {
                *world += 1;
                *door = 0;
            } else if self.save.secret {
                *secret = true;
            }
        }
        if !*secret {
            let n = WORLDS.get(*world).map(|w| w.doors.len()).unwrap_or(1);
            if pressed(edges, &[Key::Left, Key::A]) {
                *door = (*door + n - 1) % n;
            }
            if pressed(edges, &[Key::Right, Key::D]) {
                *door = (*door + 1) % n;
            }
        }
        if pressed(edges, &[Key::Enter, Key::Space]) {
            if *secret {
                if self.save.secret {
                    let stage = self.save.secret_stage.min(SECRET.len().saturating_sub(1) as u8);
                    self.begin_stage(9, 0, stage, false, true);
                }
            } else {
                let w = *world as u8;
                let d = *door as u8;
                if self.save.unlocked(w, d) {
                    let follow = self.save.world == w && self.save.door == d && !self.save.cleared;
                    let stage = if follow { self.save.stage } else { 0 };
                    let stages = campaign::stage_count(w, d) as u8;
                    if stages == 0 {
                        return;
                    }
                    let stage = stage.min(stages - 1);
                    self.begin_stage(w, d, stage, follow, false);
                } else {
                    self.toast = "that door is still shut".into();
                    self.toast_t = 1.2;
                }
            }
        }
    }

    fn begin_stage(&mut self, world: u8, door: u8, stage: u8, follow: bool, secret: bool) {
        let def = if secret { SECRET.get(stage as usize) } else { campaign::find(world, door, stage) };
        let Some(def) = def else {
            self.toast = "that stage is missing".into();
            self.toast_t = 1.4;
            return;
        };
        match Sim::boot(def, &self.save.keys, false) {
            Ok(sim) => {
                self.sim = Some(sim);
                self.scene = Scene::Play(Play {
                    world,
                    door,
                    stage,
                    follow,
                    secret,
                    deaths: 0,
                    phase: Phase::Intro,
                    phase_t: 0.0,
                    pause_sel: 0,
                    announce: String::new(),
                    sub: String::new(),
                    end_secret: false,
                    go_map: false,
                    go_end: false,
                });
            }
            Err(e) => {
                self.toast = e;
                self.toast_t = 2.0;
            }
        }
    }

    fn tick_play(&mut self, dt: f32, edges: &[Key]) {
        let Scene::Play(play) = &self.scene else { return };
        let phase = play.phase;
        let no_skip = self.sim.as_ref().is_some_and(|s| s.no_skip);
        let deaths = play.deaths;

        if phase == Phase::Pause {
            self.tick_pause(edges);
            return;
        }
        if pressed(edges, &[Key::Esc, Key::P]) && phase == Phase::Run {
            if let Scene::Play(p) = &mut self.scene {
                p.phase = Phase::Pause;
                p.pause_sel = 0;
            }
            return;
        }
        if pressed(edges, &[Key::R]) && matches!(phase, Phase::Run | Phase::Dead | Phase::Intro) {
            self.reload_stage();
            if let Scene::Play(p) = &mut self.scene {
                p.phase = Phase::Run;
                p.phase_t = 0.0;
            }
            return;
        }
        if phase == Phase::Intro && !edges.is_empty() {
            if let Scene::Play(p) = &mut self.scene {
                p.phase = Phase::Run;
                p.phase_t = 0.0;
            }
            self.keys.eat();
            return;
        }
        if phase == Phase::Run && deaths >= SKIP_AFTER && !no_skip && pressed(edges, &[Key::S]) {
            self.save.skips += 1;
            self.finish_stage(true);
            return;
        }

        let sticks = if phase == Phase::Run { self.campaign_sticks(edges) } else { Sticks::default() };
        let (died, won, key) = if let Some(sim) = &mut self.sim {
            sim.update(dt, &sticks);
            (sim.lost, sim.won, sim.got_key.take())
        } else {
            (false, false, None)
        };
        if let Some(id) = key {
            let was = self.save.secret;
            if self.save.give_key(id) {
                self.save.write();
            }
            if !was && self.save.secret {
                if let Some(sim) = &mut self.sim {
                    sim.banner = "the true door is awake";
                    sim.banner_t = 2.6;
                }
            }
        }

        let Scene::Play(p) = &mut self.scene else { return };
        p.phase_t += dt;
        match p.phase {
            Phase::Intro => {
                if p.phase_t > 0.42 {
                    p.phase = Phase::Run;
                    p.phase_t = 0.0;
                }
            }
            Phase::Run => {
                if died {
                    p.phase = Phase::Dead;
                    p.phase_t = 0.0;
                    p.deaths += 1;
                    self.save.deaths += 1;
                    self.save.write();
                    self.flash = 0.22;
                    self.shake = 0.32;
                    if self.save.bell {
                        self.chime = true;
                    }
                } else if won {
                    self.finish_stage(false);
                }
            }
            Phase::Dead => {
                if p.phase_t > 0.28 || pressed(edges, &[Key::R, Key::Enter, Key::Space]) {
                    self.reload_stage();
                    if let Scene::Play(p) = &mut self.scene {
                        p.phase = Phase::Run;
                        p.phase_t = 0.0;
                    }
                }
            }
            Phase::Clear => {
                let wait = if p.announce.contains("DOOR") || p.announce.contains("WORLD") { 1.25 } else { 0.78 };
                if p.phase_t > wait || pressed(edges, &[Key::Enter, Key::Space]) {
                    let go_end = p.go_end;
                    let end_secret = p.end_secret;
                    let go_map = p.go_map;
                    let secret = p.secret;
                    let world = p.world;
                    let door = p.door;
                    let stage = p.stage;
                    let follow = p.follow;
                    if go_end {
                        self.scene = Scene::Ending { secret: end_secret };
                        self.sim = None;
                    } else if go_map {
                        self.scene = Scene::Map { world: world as usize, door: door as usize, secret };
                        self.sim = None;
                    } else if secret {
                        self.begin_stage(9, 0, stage, false, true);
                    } else {
                        self.begin_stage(world, door, stage, follow, false);
                    }
                }
            }
            Phase::Pause => {}
        }
    }

    fn tick_pause(&mut self, edges: &[Key]) {
        let Scene::Play(p) = &mut self.scene else { return };
        if pressed(edges, &[Key::Up, Key::W]) {
            p.pause_sel = (p.pause_sel + 2) % 3;
        }
        if pressed(edges, &[Key::Down, Key::S]) {
            p.pause_sel = (p.pause_sel + 1) % 3;
        }
        if pressed(edges, &[Key::Esc, Key::P]) {
            p.phase = Phase::Run;
            return;
        }
        if pressed(edges, &[Key::Enter, Key::Space]) {
            match p.pause_sel {
                0 => p.phase = Phase::Run,
                1 => {
                    p.phase = Phase::Run;
                    p.phase_t = 0.0;
                    self.reload_stage();
                }
                _ => {
                    let world = p.world as usize;
                    let door = p.door as usize;
                    let secret = p.secret;
                    self.sim = None;
                    self.scene = Scene::Map { world, door, secret };
                }
            }
        }
    }

    fn finish_stage(&mut self, _skipped: bool) {
        let Scene::Play(p) = &self.scene else { return };
        let secret = p.secret;
        let follow = p.follow;
        let world = p.world;
        let door = p.door;
        let stage = p.stage;

        if secret {
            let next = stage + 1;
            self.save.secret_stage = next;
            self.save.write();
            if next as usize >= SECRET.len() {
                if let Scene::Play(p) = &mut self.scene {
                    p.phase = Phase::Clear;
                    p.phase_t = 0.0;
                    p.announce = "TRUE ENDING".into();
                    p.sub = "the door, for once, stayed".into();
                    p.go_end = true;
                    p.end_secret = true;
                }
            } else if let Scene::Play(p) = &mut self.scene {
                p.stage = next;
                p.phase = Phase::Clear;
                p.phase_t = 0.0;
                p.announce = "CLEARED".into();
                p.sub = "the devil keeps the next one worse".into();
            }
            return;
        }

        if follow {
            let step = campaign::bump(&mut self.save);
            self.save.write();
            match step {
                Step::Done => {
                    if let Scene::Play(p) = &mut self.scene {
                        p.phase = Phase::Clear;
                        p.phase_t = 0.0;
                        p.announce = "YOU ARE OUT".into();
                        p.sub = format!("{} deaths. the door is still a liar.", self.save.deaths);
                        p.go_end = true;
                        p.end_secret = false;
                    }
                }
                Step::World => {
                    let w = self.save.world;
                    if let Scene::Play(p) = &mut self.scene {
                        p.world = w;
                        p.door = 0;
                        p.stage = 0;
                        p.phase = Phase::Clear;
                        p.phase_t = 0.0;
                        p.announce = "WORLD OPEN".into();
                        p.sub = WORLDS.get(w as usize).map(|n| n.name).unwrap_or("").into();
                    }
                }
                Step::Door => {
                    let d = self.save.door;
                    let w = self.save.world;
                    let name = WORLDS.get(w as usize).and_then(|n| n.doors.get(d as usize)).copied().unwrap_or("Door");
                    if let Scene::Play(p) = &mut self.scene {
                        p.world = w;
                        p.door = d;
                        p.stage = 0;
                        p.phase = Phase::Clear;
                        p.phase_t = 0.0;
                        p.announce = format!("DOOR {name}");
                        p.sub = "five more lies".into();
                    }
                }
                Step::Stage => {
                    let s = self.save.stage;
                    if let Scene::Play(p) = &mut self.scene {
                        p.stage = s;
                        p.phase = Phase::Clear;
                        p.phase_t = 0.0;
                        p.announce = "CLEARED".into();
                        p.sub = String::new();
                    }
                }
            }
        } else {
            let stages = campaign::stage_count(world, door) as u8;
            if stage + 1 < stages {
                if let Scene::Play(p) = &mut self.scene {
                    p.stage += 1;
                    p.phase = Phase::Clear;
                    p.phase_t = 0.0;
                    p.announce = "CLEARED".into();
                    p.sub = "replay".into();
                }
            } else if let Scene::Play(p) = &mut self.scene {
                p.phase = Phase::Clear;
                p.phase_t = 0.0;
                p.announce = "DOOR CLEARED".into();
                p.sub = "back to the map".into();
                p.go_map = true;
            }
        }
    }

    fn reload_stage(&mut self) {
        let (world, door, stage, secret) = match &self.scene {
            Scene::Play(p) => (p.world, p.door, p.stage, p.secret),
            _ => return,
        };
        let def = if secret { SECRET.get(stage as usize) } else { campaign::find(world, door, stage) };
        if let Some(def) = def {
            if let Ok(sim) = Sim::boot(def, &self.save.keys, false) {
                self.sim = Some(sim);
            }
        }
    }

    fn campaign_sticks(&self, edges: &[Key]) -> Sticks {
        let left = self.keys.held(Key::Left) || self.keys.held(Key::A);
        let right = self.keys.held(Key::Right) || self.keys.held(Key::D);
        let jump_held = self.keys.held(Key::W) || self.keys.held(Key::Up) || self.keys.held(Key::Space);
        let jump_edge = edges.iter().any(|c| matches!(c, Key::W | Key::Up | Key::Space));
        let dir = i8::from(right) - i8::from(left);
        Sticks { p: [Stick { dir, jump_edge, jump_held }, Stick::default()] }
    }
}

fn pressed(edges: &[Key], keys: &[Key]) -> bool {
    edges.iter().any(|e| keys.contains(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_on_the_title_starts_the_first_stage() {
        let mut g = Game::from_save(Save::default());
        g.tap(Key::Enter);
        g.tick(0.016);
        match &g.scene {
            Scene::Play(p) => {
                assert_eq!((p.world, p.door, p.stage), (0, 0, 0));
                assert!(g.sim.is_some());
            }
            _ => panic!("title should start a stage"),
        }
    }

    #[test]
    fn holding_right_moves_the_player() {
        let mut g = Game::from_save(Save::default());
        g.tap(Key::Enter);
        g.tick(0.016);
        g.tick(1.2);
        let x0 = g.sim.as_ref().unwrap().actors[0].x;
        g.on_key(Key::Right, true, false);
        for _ in 0..24 {
            g.tick(0.05);
        }
        let x1 = g.sim.as_ref().unwrap().actors[0].x;
        assert!(x1 > x0 + 0.4, "expected the player to run right ({x0} -> {x1})");
    }
}

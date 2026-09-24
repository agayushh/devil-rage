//! Screens, input, and the play loop.

use std::collections::HashMap;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::style::Color;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};

use crate::campaign::{self, WorldInfo, CAMPAIGN, SECRET, WORLDS};
use crate::draw::{draw_hud, draw_world, ink, Canvas};
use crate::level::LevelDef;
use crate::model::*;
use crate::sim::{Sim, Stick, Sticks};

struct RawTerm;
impl RawTerm {
    fn enter() -> std::io::Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, Hide, SetTitle("LEVEL DEVIL"))?;
        let _ = execute!(
            out,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
            )
        );
        Ok(Self)
    }
}
impl Drop for RawTerm {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = disable_raw_mode();
        let _ = execute!(out, LeaveAlternateScreen, Show);
        let _ = out.flush();
    }
}

struct Keys {
    down: HashMap<KeyCode, Instant>,
    edges: Vec<KeyCode>,
    /// True once the terminal has delivered a key-up. Until then, a press has to
    /// count as held for long enough to steer through a jump.
    releases: bool,
}

impl Keys {
    fn new() -> Self {
        Self { down: HashMap::new(), edges: Vec::new(), releases: false }
    }
    fn held(&self, code: KeyCode) -> bool {
        match self.down.get(&code) {
            None => false,
            Some(_) if self.releases => true,
            Some(t) => t.elapsed() < Duration::from_millis(900),
        }
    }
    fn any_held(&self, codes: &[KeyCode]) -> bool {
        codes.iter().any(|c| self.held(*c))
    }
    fn note(&mut self, ev: KeyEvent) {
        let code = ev.code;
        match ev.kind {
            KeyEventKind::Release => {
                self.releases = true;
                self.down.remove(&code);
            }
            KeyEventKind::Press | KeyEventKind::Repeat => {
                let fresh = !self.down.contains_key(&code);
                self.down.insert(code, Instant::now());
                if fresh && ev.kind == KeyEventKind::Press {
                    self.edges.push(code);
                }
            }
        }
    }
    fn take_edges(&mut self) -> Vec<KeyCode> {
        std::mem::take(&mut self.edges)
    }
    fn eat(&mut self) {
        self.edges.clear();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Intro,
    Run,
    Dead,
    Clear,
    Pause,
}

struct Play {
    world: u8,
    door: u8,
    stage: u8,
    follow: bool,
    secret: bool,
    deaths: u32,
    phase: Phase,
    phase_t: f32,
    pause_sel: usize,
    announce: String,
    sub: String,
    end_secret: bool,
    go_map: bool,
    go_end: bool,
}

enum Scene {
    Title { sel: usize },
    Settings { sel: usize },
    Map { world: usize, door: usize, secret: bool },
    Play(Play),
    Ending { secret: bool },
}

struct App {
    save: Save,
    scene: Scene,
    keys: Keys,
    sim: Option<Sim>,
    time: f32,
    flash: f32,
    shake: f32,
    quit: bool,
    code: Vec<KeyCode>,
    toast: String,
    toast_t: f32,
}

const CODE: [KeyCode; 8] = [
    KeyCode::Up,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Left,
    KeyCode::Right,
];

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--check") {
        campaign::check_all()?;
        println!("level-devil: {n} campaign, {s} secret — ok", n = CAMPAIGN.len(), s = SECRET.len());
        return Ok(());
    }
    if args.iter().any(|a| a == "--reset") {
        let p = Save::path();
        let _ = std::fs::remove_file(&p);
        println!("cleared {}", p.display());
        return Ok(());
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        return Err("LEVEL DEVIL needs a terminal. Try: cargo run --release".into());
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, Show);
        default_hook(info);
    }));

    let _raw = RawTerm::enter()?;
    let mut app = App::boot();
    if let Some(spec) = args.iter().position(|a| a == "--level").and_then(|i| args.get(i + 1)) {
        if let Some((w, d, s)) = parse_spec(spec) {
            app.begin_stage(w, d, s, false, false);
        }
    } else if args.iter().any(|a| a == "--secret") {
        app.begin_stage(9, 0, app.save.secret_stage, false, true);
    }

    let mut last = Instant::now();
    while !app.quit {
        let frame = Instant::now();
        while poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(ev) = read() {
                app.on_event(ev);
            }
        }
        let now = Instant::now();
        let dt = (now - last).as_secs_f32().clamp(0.0, 0.05);
        last = now;
        app.tick(dt);
        app.draw()?;
        // Sleep out the rest of the frame. Waking on key-repeat made the picture stutter while running.
        let budget = Duration::from_millis(16);
        let spent = frame.elapsed();
        if spent < budget {
            std::thread::sleep(budget - spent);
        }
    }
    Ok(())
}

fn parse_spec(s: &str) -> Option<(u8, u8, u8)> {
    let mut p = s.split('-');
    let w: u8 = p.next()?.parse().ok()?;
    let d: u8 = p.next()?.parse().ok()?;
    let s: u8 = p.next()?.parse().ok()?;
    if w == 0 || d == 0 || s == 0 {
        return None;
    }
    Some((w - 1, d - 1, s - 1))
}

impl App {
    fn boot() -> Self {
        let save = Save::load();
        Self {
            save,
            scene: Scene::Title { sel: 0 },
            keys: Keys::new(),
            sim: None,
            time: 0.0,
            flash: 0.0,
            shake: 0.0,
            quit: false,
            code: Vec::new(),
            toast: String::new(),
            toast_t: 0.0,
        }
    }

    fn on_event(&mut self, ev: Event) {
        let Event::Key(k) = ev else { return };
        if k.modifiers.contains(KeyModifiers::CONTROL) && matches!(k.code, KeyCode::Char('c') | KeyCode::Char('C')) {
            self.quit = true;
            return;
        }
        if k.kind == KeyEventKind::Release {
            self.keys.note(k);
            return;
        }
        self.keys.note(k);
    }

    fn tick(&mut self, dt: f32) {
        self.time += dt;
        self.flash = (self.flash - dt).max(0.0);
        self.shake = (self.shake - dt).max(0.0);
        self.toast_t = (self.toast_t - dt).max(0.0);
        let edges = self.keys.take_edges();
        match &self.scene {
            Scene::Title { .. } => self.tick_title(dt, &edges),
            Scene::Settings { .. } => self.tick_settings(&edges),
            Scene::Map { .. } => self.tick_map(&edges),
            Scene::Play(_) => self.tick_play(dt, &edges),
            Scene::Ending { .. } => {
                if pressed(&edges, &[KeyCode::Enter, KeyCode::Esc, KeyCode::Char(' '), KeyCode::Char('q')]) {
                    self.scene = Scene::Map { world: 0, door: 0, secret: false };
                }
            }
        }
    }

    fn title_items(&self) -> Vec<&'static str> {
        let fresh = self.save.world == 0 && self.save.door == 0 && self.save.stage == 0 && !self.save.cleared && self.save.deaths == 0;
        if fresh {
            vec!["Play", "World map", "Settings", "Quit"]
        } else {
            vec!["Continue", "World map", "Settings", "Quit"]
        }
    }

    fn tick_title(&mut self, _dt: f32, edges: &[KeyCode]) {
        let n = self.title_items().len();
        let Scene::Title { sel } = &mut self.scene else { return };
        if pressed(edges, &[KeyCode::Up, KeyCode::Char('w'), KeyCode::Char('W'), KeyCode::Char('k')]) {
            *sel = (*sel + n - 1) % n;
        }
        if pressed(edges, &[KeyCode::Down, KeyCode::Char('s'), KeyCode::Char('S'), KeyCode::Char('j')]) {
            *sel = (*sel + 1) % n;
        }
        if pressed(edges, &[KeyCode::Enter, KeyCode::Char(' ')]) {
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
                "World map" => self.scene = Scene::Map { world: self.save.world.min(2) as usize, door: 0, secret: false },
                "Settings" => self.scene = Scene::Settings { sel: 0 },
                "Quit" => self.quit = true,
                _ => {}
            }
        }
        if pressed(edges, &[KeyCode::Char('q'), KeyCode::Esc]) {
            self.quit = true;
        }
    }

    fn tick_settings(&mut self, edges: &[KeyCode]) {
        for e in edges {
            if matches!(e, KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right) {
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
        if pressed(edges, &[KeyCode::Up, KeyCode::Char('w'), KeyCode::Char('W')]) {
            *sel = (*sel + n - 1) % n;
        }
        if pressed(edges, &[KeyCode::Down, KeyCode::Char('s'), KeyCode::Char('S')]) {
            *sel = (*sel + 1) % n;
        }
        let sel = *sel;
        let toggle = pressed(edges, &[KeyCode::Enter, KeyCode::Left, KeyCode::Right, KeyCode::Char(' ')]);
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
        if pressed(edges, &[KeyCode::Esc, KeyCode::Char('q')]) {
            self.save.write();
            self.scene = Scene::Title { sel: 0 };
        }
    }

    fn tick_map(&mut self, edges: &[KeyCode]) {
        let Scene::Map { world, door, secret } = &mut self.scene else { return };
        if pressed(edges, &[KeyCode::Esc, KeyCode::Char('q')]) {
            self.scene = Scene::Title { sel: 0 };
            return;
        }
        if pressed(edges, &[KeyCode::Up, KeyCode::Char('w'), KeyCode::Char('W')]) {
            if *secret {
                *secret = false;
                *world = WORLDS.len() - 1;
            } else if *world > 0 {
                *world -= 1;
                *door = 0;
            }
        }
        if pressed(edges, &[KeyCode::Down, KeyCode::Char('s'), KeyCode::Char('S')]) {
            if *world + 1 < WORLDS.len() && !*secret {
                *world += 1;
                *door = 0;
            } else if self.save.secret {
                *secret = true;
            }
        }
        if !*secret {
            let n = WORLDS.get(*world).map(|w| w.doors.len()).unwrap_or(1);
            if pressed(edges, &[KeyCode::Left, KeyCode::Char('a'), KeyCode::Char('A')]) {
                *door = (*door + n - 1) % n;
            }
            if pressed(edges, &[KeyCode::Right, KeyCode::Char('d'), KeyCode::Char('D')]) {
                *door = (*door + 1) % n;
            }
        }
        if pressed(edges, &[KeyCode::Enter, KeyCode::Char(' ')]) {
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
        let def = if secret {
            SECRET.get(stage as usize)
        } else {
            campaign::find(world, door, stage)
        };
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

    fn tick_play(&mut self, dt: f32, edges: &[KeyCode]) {
        let Scene::Play(play) = &self.scene else { return };
        let phase = play.phase;
        let secret = play.secret;
        let no_skip = self.sim.as_ref().is_some_and(|s| s.no_skip);
        let deaths = play.deaths;

        if phase == Phase::Pause {
            self.tick_pause(edges);
            return;
        }
        if pressed(edges, &[KeyCode::Esc, KeyCode::Char('p'), KeyCode::Char('P')]) && phase == Phase::Run {
            if let Scene::Play(p) = &mut self.scene {
                p.phase = Phase::Pause;
                p.pause_sel = 0;
            }
            return;
        }
        if pressed(edges, &[KeyCode::Char('r'), KeyCode::Char('R')]) && matches!(phase, Phase::Run | Phase::Dead | Phase::Intro) {
            self.reload_stage();
            if let Scene::Play(p) = &mut self.scene {
                if phase != Phase::Dead {
                    // retry is not a death
                }
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
        if phase == Phase::Run
            && deaths >= SKIP_AFTER
            && !no_skip
            && pressed(edges, &[KeyCode::Char('s'), KeyCode::Char('S')])
        {
            self.save.skips += 1;
            self.finish_stage(true);
            return;
        }

        let sticks = if phase == Phase::Run { self.campaign_sticks(edges) } else { Sticks::default() };
        let (died, won, cause, key) = if let Some(sim) = &mut self.sim {
            sim.update(dt, &sticks);
            (sim.lost, sim.won, sim.cause, sim.got_key.take())
        } else {
            (false, false, Cause::None, None)
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
                if p.phase_t > 1.15 {
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
                    self.shake = 0.18;
                    if self.save.bell {
                        let _ = execute!(stdout(), crossterm::style::Print("\u{7}"));
                    }
                    let _ = (cause, secret);
                } else if won {
                    self.finish_stage(false);
                }
            }
            Phase::Dead => {
                if p.phase_t > 0.72 || pressed(edges, &[KeyCode::Char('r'), KeyCode::Char('R'), KeyCode::Enter]) {
                    self.reload_stage();
                    if let Scene::Play(p) = &mut self.scene {
                        p.phase = Phase::Run;
                        p.phase_t = 0.0;
                    }
                }
            }
            Phase::Clear => {
                let wait = if p.announce.contains("DOOR") || p.announce.contains("WORLD") { 1.25 } else { 0.78 };
                if p.phase_t > wait || pressed(edges, &[KeyCode::Enter, KeyCode::Char(' ')]) {
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

    fn tick_pause(&mut self, edges: &[KeyCode]) {
        let Scene::Play(p) = &mut self.scene else { return };
        if pressed(edges, &[KeyCode::Up, KeyCode::Char('w'), KeyCode::Char('W')]) {
            p.pause_sel = (p.pause_sel + 2) % 3;
        }
        if pressed(edges, &[KeyCode::Down, KeyCode::Char('s'), KeyCode::Char('S')]) {
            p.pause_sel = (p.pause_sel + 1) % 3;
        }
        if pressed(edges, &[KeyCode::Esc, KeyCode::Char('p'), KeyCode::Char('P')]) {
            p.phase = Phase::Run;
            return;
        }
        if pressed(edges, &[KeyCode::Enter, KeyCode::Char(' ')]) {
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

    fn campaign_sticks(&self, edges: &[KeyCode]) -> Sticks {
        let left = self.keys.any_held(&[KeyCode::Char('a'), KeyCode::Char('A'), KeyCode::Left]);
        let right = self.keys.any_held(&[KeyCode::Char('d'), KeyCode::Char('D'), KeyCode::Right]);
        let jump_held = self.keys.any_held(&[
            KeyCode::Char('w'),
            KeyCode::Char('W'),
            KeyCode::Up,
            KeyCode::Char(' '),
        ]);
        let jump_edge = edges.iter().any(|c| {
            matches!(c, KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up | KeyCode::Char(' '))
        });
        let dir = right as i8 - left as i8;
        Sticks { p: [Stick { dir, jump_edge, jump_held }, Stick::default()] }
    }

    fn draw(&mut self) -> std::io::Result<()> {
        let (cols, rows) = size().unwrap_or((80, 24));
        let w = (cols as usize).max(40);
        let h = (rows as usize).max(12);
        let mut cv = Canvas::new(w, h);
        cv.clear(crate::draw::backdrop());
        match &self.scene {
            Scene::Title { sel } => self.draw_title(&mut cv, *sel),
            Scene::Settings { sel } => self.draw_settings(&mut cv, *sel),
            Scene::Map { world, door, secret } => self.draw_map(&mut cv, *world, *door, *secret),
            Scene::Play(_) => self.draw_play(&mut cv),
            Scene::Ending { secret } => self.draw_ending(&mut cv, *secret),
        }
        if self.toast_t > 0.0 {
            let y = cv.h as i32 - 3;
            cv.text(2, y, &self.toast, ink(255, 210, 120));
        }
        cv.blit()
    }

    fn draw_title(&self, cv: &mut Canvas, sel: usize) {
        let red = ink(230, 70, 90);
        let gold = ink(255, 214, 90);
        let dim = ink(92, 58, 16);
        let fg = ink(230, 230, 235);
        let art = [
            "╦  ╔═╗╦  ╦╔═╗╦    ╔╦╗╔═╗╦  ╦╦╦",
            "║  ║╣ ╚╗╔╝║╣ ║     ║║║╣ ╚╗╔╝║║",
            "╩═╝╚═╝ ╚╝ ╚═╝╩═╝  ═╩╝╚═╝ ╚╝ ╩╩═╝",
        ];
        let x = ((cv.w as i32 - 31) / 2).max(1);
        for (i, line) in art.iter().enumerate() {
            cv.text(x, 2 + i as i32, line, red);
        }
        let sub = "a terminal troll-platformer";
        cv.text((cv.w as i32 - sub.len() as i32) / 2, 6, sub, dim);
        let quote = title_line(self.time);
        cv.text((cv.w as i32 - quote.len() as i32) / 2, 8, quote, gold);

        let items = self.title_items();
        for (i, item) in items.iter().enumerate() {
            let y = 11 + i as i32;
            if i == sel {
                cv.text_bg(x, y, &format!("  {item}  "), ink(20, 8, 12), gold);
            } else {
                cv.text(x, y, &format!("  {item}"), fg);
            }
        }
        let foot = "A/D move    W jump    enter select    esc quit";
        cv.text(2, cv.h as i32 - 2, foot, dim);
        cv.text(2, cv.h as i32 - 1, "original game by Unept — these levels are a new devil", dim);
    }

    fn draw_settings(&self, cv: &mut Canvas, sel: usize) {
        let gold = ink(255, 214, 90);
        let fg = ink(230, 230, 235);
        let dim = ink(92, 58, 16);
        cv.text(3, 2, "SETTINGS", gold);
        let rows = [
            format!("hints          {}", onoff(self.save.hints)),
            format!("death bell     {}", onoff(self.save.bell)),
            format!("ascii          {}", onoff(self.save.ascii)),
            if self.save.secret {
                "true door       enter".to_string()
            } else {
                "true door       locked".to_string()
            },
            "back".to_string(),
        ];
        for (i, row) in rows.iter().enumerate() {
            let y = 5 + i as i32 * 2;
            if i == sel {
                cv.text(3, y, ">", gold);
                cv.text(5, y, row, gold);
            } else {
                cv.text(5, y, row, if i == 3 && !self.save.secret { dim } else { fg });
            }
        }
        cv.text(3, cv.h as i32 - 3, "the devil keeps a rhythm in the arrow keys", dim);
        cv.text(3, cv.h as i32 - 2, format!("keys {}/{}", self.save.keys.len(), KEY_TOTAL).as_str(), dim);
    }

    fn draw_map(&self, cv: &mut Canvas, world: usize, door: usize, secret: bool) {
        let gold = ink(255, 214, 90);
        let fg = ink(230, 230, 235);
        let dim = ink(92, 58, 16);
        let good = ink(90, 230, 150);
        let red = ink(230, 70, 90);
        cv.text(3, 1, "DOORS", gold);
        cv.text(12, 1, &format!("keys {}/{}    deaths {}", self.save.keys.len(), KEY_TOTAL, self.save.deaths), dim);
        for (i, w) in WORLDS.iter().enumerate() {
            let y = 3 + i as i32 * 5;
            let active = !secret && i == world;
            cv.text(3, y, w.name, if active { gold } else { fg });
            cv.text(22, y, w.tag, dim);
            self.draw_doors(cv, w, i, y + 1, door, active, good, dim, red, fg);
        }
        let y = 3 + WORLDS.len() as i32 * 5;
        if self.save.secret {
            let mark = if secret { ">" } else { " " };
            cv.text(3, y, &format!("{mark} TRUE DOOR"), if secret { ink(186, 120, 255) } else { ink(160, 120, 200) });
        } else {
            cv.text(3, y, "  true door locked", dim);
        }
        cv.text(3, cv.h as i32 - 2, "arrows pick a door    enter walks in    esc back", dim);
    }

    fn draw_doors(&self, cv: &mut Canvas, w: &WorldInfo, wi: usize, y: i32, door: usize, active: bool, good: Color, dim: Color, red: Color, fg: Color) {
        let mut x = 3;
        for (di, name) in w.doors.iter().enumerate() {
            let open = self.save.unlocked(wi as u8, di as u8);
            let current = !self.save.cleared && self.save.world == wi as u8 && self.save.door == di as u8;
            let label = if !open {
                format!("[{name}]")
            } else if current {
                format!("<{name} {}/5>", self.save.stage + 1)
            } else {
                format!("({name})")
            };
            let col = if active && di == door {
                ink(20, 8, 12)
            } else {
                ink(174, 126, 36)
            };
            let text_c = if !open {
                dim
            } else if current {
                red
            } else {
                good
            };
            if active && di == door {
                cv.text_bg(x, y, &format!(" {label} "), ink(20, 8, 12), ink(255, 214, 90));
            } else {
                let _ = (col, fg, text_c);
                cv.text(x, y, &format!(" {label} "), text_c);
            }
            x += label.len() as i32 + 3;
        }
    }

    fn draw_play(&self, cv: &mut Canvas) {
        let Scene::Play(p) = &self.scene else { return };
        let Some(sim) = &self.sim else { return };
        let ink_dark = ink(28, 16, 8);
        let gold = ink(255, 214, 90);
        let dim = ink(92, 58, 16);
        let red = ink(120, 36, 40);
        let fg = ink(28, 16, 8);
        let door_name = if p.secret {
            "secret"
        } else {
            WORLDS.get(p.world as usize).and_then(|w| w.doors.get(p.door as usize)).copied().unwrap_or("door")
        };

        let vh = (cv.h as i32 - 2).max(8);
        if cv.w > 20 {
            draw_world(cv, sim, 0, 0, cv.w as i32, vh, self.save.ascii, self.shake);
        }
        draw_hud(cv, p.stage);

        if sim.jet {
            let n = if sim.fuel_max > 0.0 { ((sim.fuel / sim.fuel_max) * 8.0).round() as i32 } else { 0 };
            let mut bar = String::from("jet ");
            for i in 0..8 {
                bar.push(if i < n { '█' } else { '░' });
            }
            cv.text(cv.w as i32 - bar.len() as i32 - 2, 1, &bar, ink_dark);
        }
        if let Some(n) = sim.jumps_left {
            cv.text(cv.w as i32 - 10, 2, &format!("jumps {n}"), ink_dark);
        }

        match p.phase {
            Phase::Intro => {
                card(cv, &format!("{}  {}/5", door_name, p.stage + 1), sim.name, "any key");
            }
            Phase::Dead => {
                let q = sim.cause.quip(self.save.deaths + p.deaths);
                card(cv, "DEAD", q, "R retry");
            }
            Phase::Clear => {
                card(cv, &p.announce, &p.sub, "");
            }
            Phase::Pause => {
                card(cv, "PAUSED", "", "");
                let opts = ["resume", "retry", "map"];
                for (i, o) in opts.iter().enumerate() {
                    let y = cv.h as i32 / 2 + 1 + i as i32;
                    if i == p.pause_sel {
                        cv.text_bg(cv.w as i32 / 2 - 6, y, &format!(" {o} "), ink(255, 236, 210), red);
                    } else {
                        cv.text(cv.w as i32 / 2 - 6, y, &format!("  {o}"), fg);
                    }
                }
            }
            Phase::Run => {}
        }

        let mut foot = if sim.lie && p.phase == Phase::Run {
            "TIP: hold D — the door is that way".to_string()
        } else {
            String::new()
        };
        if p.deaths >= SKIP_AFTER && !sim.no_skip && p.phase == Phase::Run {
            foot.push_str("    S skip");
        }
        if self.save.hints && p.deaths >= 3 && p.phase == Phase::Run && !sim.hint.is_empty() {
            cv.text(2, cv.h as i32 - 1, &clip(sim.hint, cv.w.saturating_sub(4)), ink_dark);
        } else if sim.banner_t > 0.0 {
            cv.text(2, cv.h as i32 - 1, sim.banner, ink_dark);
        } else if !foot.is_empty() {
            cv.text(2, cv.h as i32 - 1, &clip(&foot, cv.w.saturating_sub(4)), dim);
        }
        let _ = gold;
    }

    fn draw_ending(&self, cv: &mut Canvas, secret: bool) {
        let gold = ink(255, 214, 90);
        let fg = ink(230, 230, 235);
        let dim = ink(92, 58, 16);
        let title = if secret { "TRUE ENDING" } else { "ENDING" };
        cv.text(3, 3, title, if secret { ink(186, 120, 255) } else { gold });
        let lines: &[&str] = if secret {
            &[
                "The devil holds the door.",
                "It does not move.",
                "You can stop guessing.",
            ]
        } else if self.save.secret || self.save.keys.len() >= KEY_TOTAL as usize {
            &[
                "You reached a door that stayed a door.",
                "Something else unlocked.",
                "Check the map.",
            ]
        } else {
            &[
                "You reached a door that stayed a door.",
                "Not every key has been found.",
                "The settings menu is a liar too.",
            ]
        };
        for (i, line) in lines.iter().enumerate() {
            cv.text(3, 6 + i as i32 * 2, line, fg);
        }
        cv.text(3, 14, &format!("deaths {}    skips {}    keys {}/{}", self.save.deaths, self.save.skips, self.save.keys.len(), KEY_TOTAL), dim);
        cv.text(3, cv.h as i32 - 2, "enter", dim);
    }
}

fn pressed(edges: &[KeyCode], codes: &[KeyCode]) -> bool {
    edges.iter().any(|e| codes.contains(e))
}

fn card(cv: &mut Canvas, title: &str, sub: &str, hint: &str) {
    let y = cv.h as i32 / 2 - 1;
    let x = (cv.w as i32 - title.len() as i32) / 2;
    cv.text_bg(x - 2, y, &format!("  {title}  "), ink(255, 236, 210), ink(120, 30, 48));
    if !sub.is_empty() {
        let x = (cv.w as i32 - sub.len() as i32) / 2;
        cv.text(x, y + 2, sub, ink(255, 214, 90));
    }
    if !hint.is_empty() {
        let x = (cv.w as i32 - hint.len() as i32) / 2;
        cv.text(x, y + 4, hint, ink(92, 58, 16));
    }
}

fn onoff(v: bool) -> &'static str {
    if v { "on" } else { "off" }
}

fn clip(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[allow(dead_code)]
fn _level_name(def: &LevelDef) -> &str {
    def.name
}

#[allow(dead_code)]
fn _world(w: &WorldInfo) -> &str {
    w.name
}

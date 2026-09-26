//! Realtime troll-platformer simulation. Terminal-free so it can be rehearsed in tests.

use crate::level::{build, LevelDef, Trap};
use crate::model::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Stick {
    pub dir: i8,
    pub jump_edge: bool,
    pub jump_held: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sticks {
    pub p: [Stick; 2],
}

impl Sticks {
    pub fn one(dir: i8, edge: bool, held: bool) -> Self {
        Self { p: [Stick { dir, jump_edge: edge, jump_held: held }, Stick::default()] }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn overlaps(self, o: Rect) -> bool {
        self.x < o.x + o.w - 1e-4
            && self.x + self.w > o.x + 1e-4
            && self.y < o.y + o.h - 1e-4
            && self.y + self.h > o.y + 1e-4
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub w: f32,
    pub h: f32,
    pub on_ground: bool,
    pub on_ice: bool,
    pub coyote: f32,
    pub jump_buf: f32,
    pub rising: bool,
    pub portal_cd: f32,
    pub alive: bool,
    pub reached: bool,
    pub puppet: bool,
    pub role: u8,
    pub profile: Profile,
    pub riding: Option<u16>,
    pub cause: Cause,
}

impl Actor {
    fn body(&self) -> Rect {
        Rect { x: self.x, y: self.y, w: self.w, h: self.h }
    }
    fn hurt(&self) -> Rect {
        let m = 0.06;
        Rect {
            x: self.x + m,
            y: self.y + m,
            w: (self.w - m * 2.0).max(0.12),
            h: (self.h - m * 2.0).max(0.12),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Ent {
    Saw { x: f32, y: f32, phase: f32, range: f32, speed: f32, base_x: f32, base_y: f32 },
    Plat {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        prev_x: f32,
        prev_y: f32,
        phase: f32,
        range: f32,
        speed: f32,
        vertical: bool,
        base_x: f32,
        base_y: f32,
    },
    Wall {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tx: f32,
        ty: f32,
        speed: f32,
        at: f32,
        armed: bool,
        done: bool,
        ldx: f32,
        ldy: f32,
    },
    Chase { x: f32, y: f32 },
}

#[derive(Clone, Debug)]
pub enum Shot {
    Bullet { x: f32, y: f32, vx: f32, vy: f32 },
    Bomb { x: f32, y: f32, vx: f32, vy: f32 },
}

#[derive(Clone, Debug)]
struct Cross {
    at: f32,
    fired: bool,
    kind: CrossKind,
}

#[derive(Clone, Debug)]
enum CrossKind {
    High(f32),
    Gravity,
    JetOff,
    Infinite,
    Taunt(&'static str),
    MoveDoor { tx: f32, ty: f32, speed: f32 },
}

#[derive(Clone, Debug)]
struct Zone {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rate: f32,
}

#[derive(Clone, Debug)]
pub struct Door {
    pub x: f32,
    pub y: f32,
    pub tx: f32,
    pub ty: f32,
    pub speed: f32,
    pub id: u8,
    pub moving: bool,
}

pub struct Sim {
    pub name: &'static str,
    pub hint: &'static str,
    pub world: u8,
    pub door_i: u8,
    pub stage: u8,
    pub key: Option<u8>,
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
    pub actors: Vec<Actor>,
    pub ents: Vec<Ent>,
    pub shots: Vec<Shot>,
    pub doors: Vec<Door>,
    pub pops: Vec<(u16, u16)>,
    pub pop_at: f32,
    pub pop_lead: f32,
    pub pop_t: f32,
    pub pop_armed: bool,
    pub pop_live: bool,
    pub flicker_on: bool,
    pub flicker_t: f32,
    pub flicker_period: f32,
    pub has_flicker: bool,
    pub reversed: bool,
    pub reverse_line: Option<f32>,
    pub reverse_sticky: bool,
    pub grav: f32,
    pub jump_scale: f32,
    pub jet: bool,
    pub jet_off: bool,
    pub fuel: f32,
    pub fuel_max: f32,
    pub infinite: bool,
    pub jumps_left: Option<u8>,
    pub air_left: u8,
    pub air_stock: u8,
    pub clone: Option<CloneKind>,
    pub versus: bool,
    pub lie: bool,
    pub no_skip: bool,
    pub peace: bool,
    pub group_delay: f32,
    pub solo_delay: f32,
    pub won: bool,
    pub lost: bool,
    pub winner: Option<u8>,
    pub cause: Cause,
    pub got_key: Option<u8>,
    pub coins: u32,
    pub banner: &'static str,
    pub banner_t: f32,
    pub time: f32,
    crosses: Vec<Cross>,
    zones: Vec<Zone>,
    chase_speed: f32,
}

struct Hit {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    kind: Kind,
    ent: Option<usize>,
}

impl Sim {
    pub fn start(def: &LevelDef, have: &[u8], versus: bool) -> Result<Self, String> {
        let built = build(def)?;
        let mut sim = Self {
            name: def.name,
            hint: def.hint,
            world: def.world,
            door_i: def.door,
            stage: def.stage,
            key: def.key,
            w: built.w,
            h: built.h,
            cells: built.cells,
            actors: Vec::new(),
            ents: Vec::new(),
            shots: Vec::new(),
            doors: Vec::new(),
            pops: built.pops,
            pop_at: 999.0,
            pop_lead: 1.35,
            pop_t: 0.0,
            pop_armed: false,
            pop_live: false,
            flicker_on: true,
            flicker_t: 0.0,
            flicker_period: 0.7,
            has_flicker: false,
            reversed: false,
            reverse_line: None,
            reverse_sticky: false,
            grav: 1.0,
            jump_scale: 1.0,
            jet: false,
            jet_off: false,
            fuel: 0.0,
            fuel_max: 0.0,
            infinite: false,
            jumps_left: None,
            air_left: 0,
            air_stock: 0,
            clone: None,
            versus: false,
            lie: false,
            no_skip: false,
            peace: false,
            group_delay: 0.10,
            solo_delay: 0.18,
            won: false,
            lost: false,
            winner: None,
            cause: Cause::None,
            got_key: None,
            coins: 0,
            banner: "",
            banner_t: 0.0,
            time: 0.0,
            crosses: Vec::new(),
            zones: Vec::new(),
            chase_speed: 0.0,
        };
        sim.has_flicker = sim.cells.iter().any(|c| c.kind == Kind::Flicker);

        if let Some(id) = def.key {
            if have.contains(&id) {
                for c in &mut sim.cells {
                    if c.kind == Kind::Key {
                        *c = Cell::air();
                    }
                }
            }
        }

        for (x, y, id) in built.doors {
            sim.doors.push(Door { x, y, tx: x, ty: y, speed: 0.0, id, moving: false });
        }
        for (x, y) in built.saws {
            sim.ents.push(Ent::Saw {
                x: x + 0.15,
                y: y + 0.15,
                phase: sim.ents.len() as f32 * 1.7,
                range: 1.65,
                speed: 3.6,
                base_x: x,
                base_y: y,
            });
        }
        for (x, y, w) in built.plats {
            sim.ents.push(Ent::Plat {
                x,
                y,
                w,
                h: 0.34,
                prev_x: x,
                prev_y: y,
                phase: -std::f32::consts::FRAC_PI_2,
                range: 5.0,
                speed: 1.7,
                vertical: false,
                base_x: x,
                base_y: y,
            });
        }

        for trap in def.traps {
            sim.apply_trap(*trap);
        }

        if !sim.pops.is_empty() && sim.pop_at > 900.0 {
            let min_x = sim.pops.iter().map(|p| p.0).min().unwrap_or(0) as f32;
            let spawn_x = built.spawns.iter().find(|s| s.2 == 0).map(|s| s.0).unwrap_or(0.0);
            sim.pop_at = (min_x - sim.pop_lead).max(spawn_x + 1.4);
        }

        let profile = sim.actors_profile_from_traps(def);
        let mut spawns = built.spawns;
        spawns.sort_by_key(|s| s.2);
        for (x, y, role) in spawns {
            if role == 2 && matches!(sim.clone, None | Some(CloneKind::Mirror)) {
                return Err(format!("{}: spawn 2 without a clone trap", def.name));
            }
            if role == 1 && !versus {
                return Err(format!("{}: spawn 1 outside versus", def.name));
            }
            sim.actors.push(sim.make_actor(x, y, role, profile));
        }
        sim.versus = versus;
        if versus && !sim.actors.iter().any(|a| a.role == 1) {
            return Err(format!("{}: versus needs a 1 spawn", def.name));
        }
        match sim.clone {
            Some(CloneKind::Opposite | CloneKind::Shadow | CloneKind::Dual) => {
                if !sim.actors.iter().any(|a| a.role == 2) {
                    return Err(format!("{}: clone needs a 2 spawn", def.name));
                }
            }
            Some(CloneKind::Mirror) => {
                let a = sim.actors[0];
                let mut c = a;
                c.role = 2;
                c.puppet = true;
                c.x = sim.w as f32 - (a.x + a.w);
                sim.actors.push(c);
            }
            None => {}
        }
        if sim.clone == Some(CloneKind::Dual) && !sim.doors.iter().any(|d| d.id == 1) {
            return Err(format!("{}: dual clone needs door E", def.name));
        }
        Ok(sim)
    }

    pub fn boot(def: &LevelDef, have: &[u8], versus: bool) -> Result<Self, String> {
        Self::start(def, have, versus)
    }

    fn actors_profile_from_traps(&self, def: &LevelDef) -> Profile {
        for t in def.traps {
            if let Trap::Start(p) = t {
                return *p;
            }
        }
        Profile::Normal
    }

    fn make_actor(&self, x: f32, mut y: f32, role: u8, profile: Profile) -> Actor {
        let (nw, nh) = dims(Profile::Normal);
        let feet = y + nh;
        let (w, h) = dims(profile);
        y = feet - h;
        let _ = (x, nw);
        Actor {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            w,
            h,
            on_ground: false,
            on_ice: false,
            coyote: 0.0,
            jump_buf: 0.0,
            rising: false,
            portal_cd: 0.0,
            alive: true,
            reached: false,
            puppet: false,
            role,
            profile,
            riding: None,
            cause: Cause::None,
        }
    }

    fn apply_trap(&mut self, trap: Trap) {
        match trap {
            Trap::Delays { group, solo } => {
                self.group_delay = group;
                self.solo_delay = solo;
            }
            Trap::PopLead(v) => self.pop_lead = v,
            Trap::PopAt(x) => self.pop_at = x,
            Trap::Reverse { at, sticky } => {
                if at <= 0.05 {
                    self.reversed = true;
                } else {
                    self.reverse_line = Some(at);
                    self.reverse_sticky = sticky;
                }
            }
            Trap::HighJump { at, scale } => {
                if at <= 0.05 {
                    self.jump_scale = scale;
                } else {
                    self.crosses.push(Cross { at, fired: false, kind: CrossKind::High(scale) });
                }
            }
            Trap::Gravity { at } => {
                if at <= 0.05 {
                    self.grav = -1.0;
                } else {
                    self.crosses.push(Cross { at, fired: false, kind: CrossKind::Gravity });
                }
            }
            Trap::Jetpack(fuel) => {
                self.jet = true;
                self.fuel = fuel;
                self.fuel_max = fuel;
            }
            Trap::JetpackOff { at } => {
                self.crosses.push(Cross { at, fired: false, kind: CrossKind::JetOff });
            }
            Trap::Infinite { at } => {
                if at <= 0.05 {
                    self.infinite = true;
                } else {
                    self.crosses.push(Cross { at, fired: false, kind: CrossKind::Infinite });
                }
            }
            Trap::JumpLimit(n) => self.jumps_left = Some(n),
            Trap::Chase { row, speed } => {
                self.chase_speed = speed;
                self.ents.push(Ent::Chase { x: -3.0, y: row as f32 + 0.08 });
            }
            Trap::Platform { x, y, w, range, speed, vertical } => {
                self.ents.push(Ent::Plat {
                    x,
                    y,
                    w,
                    h: 0.34,
                    prev_x: x,
                    prev_y: y,
                    phase: -std::f32::consts::FRAC_PI_2,
                    range,
                    speed,
                    vertical,
                    base_x: x,
                    base_y: y,
                });
            }
            Trap::Wall { x, y, w, h, tx, ty, speed, at } => {
                self.ents.push(Ent::Wall {
                    x,
                    y,
                    w,
                    h,
                    tx,
                    ty,
                    speed,
                    at,
                    armed: false,
                    done: false,
                    ldx: 0.0,
                    ldy: 0.0,
                });
            }
            Trap::MoveDoor { at, tx, ty, speed } => {
                self.crosses.push(Cross {
                    at,
                    fired: false,
                    kind: CrossKind::MoveDoor { tx, ty, speed },
                });
            }
            Trap::FuelDrain { x, y, w, h, rate } => self.zones.push(Zone { x, y, w, h, rate }),
            Trap::Taunt { at, text } => {
                self.crosses.push(Cross { at, fired: false, kind: CrossKind::Taunt(text) });
            }
            Trap::FlickerPeriod(p) => self.flicker_period = p.max(0.2),
            Trap::Clone(k) => self.clone = Some(k),
            Trap::Start(_) => {}
            Trap::Lie => self.lie = true,
            Trap::NoSkip => self.no_skip = true,
            Trap::Peace => self.peace = true,
        }
    }

    pub fn update(&mut self, dt: f32, sticks: &Sticks) {
        if self.won || self.lost {
            return;
        }
        let dt = dt.min(0.05);
        let mut left = dt;
        let mut first = true;
        while left > 1e-6 {
            let h = left.min(1.0 / 120.0);
            let mut s = *sticks;
            if !first {
                s.p[0].jump_edge = false;
                s.p[1].jump_edge = false;
            }
            first = false;
            self.step(h, &s);
            left -= h;
            if self.won || self.lost {
                break;
            }
        }
    }

    fn step(&mut self, dt: f32, sticks: &Sticks) {
        self.time += dt;
        if self.banner_t > 0.0 {
            self.banner_t -= dt;
        }
        self.tick_pop(dt);
        self.tick_flicker(dt);
        self.tick_doors(dt);
        self.tick_ents(dt);
        self.carry();
        self.tick_guns(dt);

        let n = self.actors.len();
        for i in 0..n {
            if self.actors[i].puppet || !self.actors[i].alive {
                continue;
            }
            let stick = self.stick_for(i, sticks);
            self.integrate(i, dt, stick);
        }
        self.mirror_puppet();
        self.arm_crumbles();
        self.tick_crumbles(dt);
        self.push_walls();
        self.hazards();
        self.interact();
        self.crosses_after();
        self.finish_flags();
    }

    fn stick_for(&self, i: usize, sticks: &Sticks) -> Stick {
        let versus = self.versus;
        let base = if versus {
            sticks.p[i.min(1)]
        } else {
            sticks.p[0]
        };
        let mut screen = base.dir;
        if self.reversed {
            screen = -screen;
        }
        if i > 0 && self.clone == Some(CloneKind::Opposite) {
            screen = -screen;
        }
        Stick { dir: screen, jump_edge: base.jump_edge, jump_held: base.jump_held }
    }

    fn integrate(&mut self, i: usize, dt: f32, stick: Stick) {
        let mut a = self.actors[i];
        a.portal_cd = (a.portal_cd - dt).max(0.0);
        a.coyote -= dt;
        if stick.jump_edge {
            a.jump_buf = JUMP_BUFFER;
        } else {
            a.jump_buf -= dt;
        }

        let target = stick.dir as f32 * MOVE_SPEED;
        let accel = if a.on_ice {
            if stick.dir == 0 { ICE_FRICTION } else { ICE_ACCEL }
        } else if !a.on_ground {
            if stick.dir == 0 { FRICTION * 0.72 } else { ACCEL }
        } else if stick.dir == 0 {
            FRICTION
        } else {
            ACCEL
        };
        let dv = accel * dt;
        if (a.vx - target).abs() <= dv {
            a.vx = target;
        } else if a.vx < target {
            a.vx += dv;
        } else {
            a.vx -= dv;
        }

        let grounded = a.on_ground || a.coyote > 0.0;
        let limit_ok = self.jumps_left.map(|n| n > 0).unwrap_or(true);
        let air_ok = grounded || self.infinite || self.air_left > 0;
        if a.jump_buf > 0.0 && limit_ok && air_ok && a.alive {
            if !grounded && !self.infinite {
                self.air_left = self.air_left.saturating_sub(1);
            }
            if let Some(n) = self.jumps_left {
                self.jumps_left = Some(n.saturating_sub(1));
            }
            let sign = if self.grav >= 0.0 { -1.0 } else { 1.0 };
            a.vy = sign * JUMP_V * self.jump_scale;
            a.on_ground = false;
            a.coyote = 0.0;
            a.jump_buf = 0.0;
            a.rising = true;
        }

        let jet_ok = self.jet && !self.jet_off && self.fuel > 0.0;
        if jet_ok && stick.jump_held && !a.on_ground {
            a.vy = (a.vy - 55.0 * dt).max(-5.2);
            self.fuel = (self.fuel - dt).max(0.0);
            a.rising = false;
        } else if a.rising && !stick.jump_held && a.vy * self.grav < 0.0 {
            a.vy *= 0.34;
            a.rising = false;
        }

        if jet_ok {
            for z in &self.zones {
                let r = Rect { x: z.x, y: z.y, w: z.w, h: z.h };
                if a.body().overlaps(r) {
                    self.fuel = (self.fuel - z.rate * dt).max(0.0);
                }
            }
        }

        if a.on_ground {
            if self.grav > 0.0 && a.vy > 0.0 {
                a.vy = 0.0;
            }
            if self.grav < 0.0 && a.vy < 0.0 {
                a.vy = 0.0;
            }
        } else {
            a.vy += self.grav * GRAVITY * dt;
        }
        a.vy = a.vy.clamp(-MAX_FALL, MAX_FALL);
        if a.vy * self.grav > 0.0 {
            a.rising = false;
        }

        let was_ground = a.on_ground;
        a.on_ground = false;
        a.on_ice = false;
        a.riding = None;

        a.x += a.vx * dt;
        self.resolve_x(&mut a);
        let prev_bottom = a.y + a.h;
        let prev_top = a.y;
        a.y += a.vy * dt;
        self.resolve_y(&mut a, prev_bottom, prev_top);

        if a.x < 0.0 {
            a.x = 0.0;
            a.vx = 0.0;
        }
        let max_x = self.w as f32 - a.w;
        if a.x > max_x {
            a.x = max_x;
            a.vx = 0.0;
        }

        if self.grav > 0.0 && a.vy >= -0.01 && !a.rising {
            let feet = a.y + a.h;
            let probe = Rect {
                x: a.x + 0.1,
                y: feet,
                w: (a.w - 0.2).max(0.12),
                h: 0.08,
            };
            if let Some(hit) = self.best_floor(&probe, feet) {
                if feet <= hit.y + 0.1 && feet >= hit.y - 0.04 {
                    a.y = hit.y - a.h - 0.002;
                    a.vy = 0.0;
                    a.on_ground = true;
                    a.on_ice = hit.kind == Kind::Ice;
                    a.riding = hit.ent.map(|id| id as u16);
                }
            }
        }

        if was_ground && !a.on_ground && a.vy * self.grav >= -1.0 {
            a.coyote = COYOTE;
        }
        if a.on_ground {
            a.coyote = 0.0;
        }

        if a.y > self.h as f32 + 0.05 {
            a.alive = false;
            a.cause = Cause::Fall;
        }
        if a.y + a.h < -7.0 {
            a.alive = false;
            a.cause = Cause::Fall;
        }

        self.actors[i] = a;
    }

    fn resolve_x(&self, a: &mut Actor) {
        for _ in 0..6 {
            let body = Rect { x: a.x, y: a.y + 0.12, w: a.w, h: (a.h - 0.24).max(0.15) };
            let Some(hit) = self.best_x(&body, a.vx) else { break };
            if a.vx > 0.0 {
                a.x = hit.x - a.w - 0.002;
            } else if a.vx < 0.0 {
                a.x = hit.x + hit.w + 0.002;
            } else {
                let amid = a.x + a.w * 0.5;
                let hmid = hit.x + hit.w * 0.5;
                if amid < hmid {
                    a.x = hit.x - a.w - 0.002;
                } else {
                    a.x = hit.x + hit.w + 0.002;
                }
            }
            a.vx = 0.0;
        }
    }

    fn resolve_y(&self, a: &mut Actor, prev_bottom: f32, prev_top: f32) {
        let into = if self.grav >= 0.0 { a.vy >= 0.0 } else { a.vy <= 0.0 };
        let body = Rect {
            x: a.x + 0.08,
            y: a.y,
            w: (a.w - 0.16).max(0.12),
            h: a.h,
        };
        if self.grav >= 0.0 {
            if a.vy > 0.0 {
                if let Some(hit) = self.best_floor(&body, prev_bottom) {
                    self.land(a, &hit);
                }
            } else if a.vy < 0.0 {
                if let Some(hit) = self.best_ceiling(&body, prev_top) {
                    a.y = hit.y + hit.h + 0.002;
                    a.vy = 0.0;
                    a.rising = false;
                }
            }
        } else if into && a.vy < 0.0 {
            // falling "up" onto a ceiling
            if let Some(hit) = self.best_ceiling(&body, prev_top) {
                self.land_ceiling(a, &hit);
            }
        } else if a.vy > 0.0 {
            if let Some(hit) = self.best_floor(&body, prev_bottom) {
                a.y = hit.y - a.h - 0.002;
                a.vy = 0.0;
                a.rising = false;
            }
        }
    }

    fn land(&self, a: &mut Actor, hit: &Hit) {
        if matches!(hit.kind, Kind::Spring | Kind::Super) {
            let mul = if hit.kind == Kind::Super { 1.62 } else { 1.18 };
            a.y = hit.y - a.h - 0.002;
            a.vy = -JUMP_V * mul;
            a.rising = true;
            a.on_ground = false;
            return;
        }
        a.y = hit.y - a.h - 0.002;
        a.vy = 0.0;
        a.on_ground = true;
        a.rising = false;
        a.on_ice = hit.kind == Kind::Ice;
        if let Some(id) = hit.ent {
            a.riding = Some(id as u16);
        }
    }

    fn land_ceiling(&self, a: &mut Actor, hit: &Hit) {
        if matches!(hit.kind, Kind::Spring | Kind::Super) {
            let mul = if hit.kind == Kind::Super { 1.62 } else { 1.18 };
            a.y = hit.y + hit.h + 0.002;
            a.vy = JUMP_V * mul;
            a.rising = true;
            a.on_ground = false;
            return;
        }
        a.y = hit.y + hit.h + 0.002;
        a.vy = 0.0;
        a.on_ground = true;
        a.rising = false;
        a.on_ice = hit.kind == Kind::Ice;
    }

    fn best_x(&self, body: &Rect, vx: f32) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        let mut best_pen = -1.0f32;
        self.for_each_solid(*body, false, 0.0, |hit| {
            let pen = if vx > 0.0 {
                body.x + body.w - hit.x
            } else {
                hit.x + hit.w - body.x
            };
            if pen > best_pen {
                best_pen = pen;
                best = Some(hit);
            }
        });
        best
    }

    fn best_floor(&self, body: &Rect, prev_bottom: f32) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        self.for_each_solid(*body, true, prev_bottom, |hit| {
            let take = match &best {
                None => true,
                Some(b) => hit.y < b.y,
            };
            if take {
                best = Some(hit);
            }
        });
        best
    }

    fn best_ceiling(&self, body: &Rect, prev_top: f32) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        self.for_each_solid(*body, false, prev_top, |hit| {
            let bottom = hit.y + hit.h;
            let take = match &best {
                None => true,
                Some(b) => bottom > b.y + b.h,
            };
            if take && prev_top >= hit.y - 0.2 {
                best = Some(hit);
            }
        });
        best
    }

    fn for_each_solid(&self, body: Rect, feet: bool, prev_edge: f32, mut f: impl FnMut(Hit)) {
        let shrink = body;
        let x0 = shrink.x.floor() as i32 - 1;
        let y0 = shrink.y.floor() as i32 - 1;
        let x1 = (shrink.x + shrink.w).ceil() as i32 + 1;
        let y1 = (shrink.y + shrink.h).ceil() as i32 + 1;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if tx < 0 || ty < 0 || tx >= self.w as i32 || ty >= self.h as i32 {
                    continue;
                }
                let cell = self.cells[ty as usize * self.w + tx as usize];
                let tile = Rect { x: tx as f32, y: ty as f32, w: 1.0, h: 1.0 };
                if !shrink.overlaps(tile) {
                    continue;
                }
                if cell.kind.blocks(self.flicker_on) {
                    f(Hit { x: tile.x, y: tile.y, w: 1.0, h: 1.0, kind: cell.kind, ent: None });
                } else if feet && cell.kind == Kind::OneWay && self.grav > 0.0 && prev_edge <= tile.y + 0.08 {
                    f(Hit { x: tile.x, y: tile.y, w: 1.0, h: 1.0, kind: cell.kind, ent: None });
                }
            }
        }
        for (i, ent) in self.ents.iter().enumerate() {
            match ent {
                Ent::Wall { x, y, w, h, .. } => {
                    let tile = Rect { x: *x, y: *y, w: *w, h: *h };
                    if shrink.overlaps(tile) {
                        f(Hit { x: *x, y: *y, w: *w, h: *h, kind: Kind::Solid, ent: Some(i) });
                    }
                }
                Ent::Plat { x, y, w, h, .. } if feet && self.grav > 0.0 && prev_edge <= *y + 0.12 => {
                    let tile = Rect { x: *x, y: *y, w: *w, h: *h };
                    if shrink.overlaps(tile) {
                        f(Hit { x: *x, y: *y, w: *w, h: *h, kind: Kind::OneWay, ent: Some(i) });
                    }
                }
                _ => {}
            }
        }
    }

    fn carry(&mut self) {
        let deltas: Vec<(u16, f32, f32)> = self
            .ents
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                Ent::Plat { x, y, prev_x, prev_y, .. } => Some((i as u16, x - prev_x, y - prev_y)),
                _ => None,
            })
            .collect();
        for a in &mut self.actors {
            if let Some(id) = a.riding {
                if let Some((_, dx, dy)) = deltas.iter().find(|(i, _, _)| *i == id) {
                    a.x += dx;
                    a.y += dy;
                }
            }
        }
    }

    fn tick_ents(&mut self, dt: f32) {
        let px = self.actors.first().map(|a| a.x).unwrap_or(0.0);
        let chase_speed = self.chase_speed;
        for e in &mut self.ents {
            match e {
                Ent::Saw { x, y, phase, range, speed, base_x, base_y } => {
                    *phase += *speed * dt;
                    *x = *base_x + 0.15 + phase.sin() * *range;
                    *y = *base_y + 0.15;
                }
                Ent::Plat { x, y, prev_x, prev_y, phase, range, speed, vertical, base_x, base_y, .. } => {
                    *prev_x = *x;
                    *prev_y = *y;
                    *phase += *speed * dt;
                    let t = (phase.sin() + 1.0) * 0.5;
                    if *vertical {
                        *x = *base_x;
                        *y = *base_y + t * *range;
                    } else {
                        *x = *base_x + t * *range;
                        *y = *base_y;
                    }
                }
                Ent::Wall { x, y, tx, ty, speed, at, armed, done, ldx, ldy, .. } => {
                    *ldx = 0.0;
                    *ldy = 0.0;
                    if !*armed && px >= *at {
                        *armed = true;
                    }
                    if *armed && !*done {
                        let dx = *tx - *x;
                        let dy = *ty - *y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < 0.05 {
                            *x = *tx;
                            *y = *ty;
                            *done = true;
                        } else {
                            let step = (*speed * dt).min(dist);
                            *ldx = dx / dist * step;
                            *ldy = dy / dist * step;
                            *x += *ldx;
                            *y += *ldy;
                        }
                    }
                }
                Ent::Chase { x, y: _ } => {
                    *x += chase_speed * dt;
                }
            }
        }
    }

    fn tick_guns(&mut self, dt: f32) {
        let w = self.w;
        let mut born = Vec::new();
        for (i, c) in self.cells.iter_mut().enumerate() {
            let x = (i % w) as f32;
            let y = (i / w) as f32;
            match c.kind {
                Kind::GunR | Kind::GunL | Kind::BombGun => {
                    c.timer -= dt;
                    if c.timer <= 0.0 {
                        c.timer = if c.kind == Kind::BombGun { 1.7 } else { 1.25 };
                        born.push((c.kind, x, y));
                    }
                }
                _ => {}
            }
        }
        for (k, x, y) in born {
            match k {
                Kind::GunR => self.shots.push(Shot::Bullet { x: x + 1.0, y: y + 0.25, vx: 12.0, vy: 0.0 }),
                Kind::GunL => self.shots.push(Shot::Bullet { x: x - 0.4, y: y + 0.25, vx: -12.0, vy: 0.0 }),
                Kind::BombGun => self.shots.push(Shot::Bomb { x: x - 0.4, y: y + 0.1, vx: -6.5, vy: -2.0 }),
                _ => {}
            }
        }
        for s in &mut self.shots {
            match s {
                Shot::Bullet { x, y, vx, vy } => {
                    *x += *vx * dt;
                    *y += *vy * dt;
                }
                Shot::Bomb { x, y, vx, vy } => {
                    *vy += 16.0 * dt;
                    *x += *vx * dt;
                    *y += *vy * dt;
                }
            }
        }
        let solids = self.solid_snapshot();
        self.shots.retain(|s| {
            let (x, y) = match s {
                Shot::Bullet { x, y, .. } | Shot::Bomb { x, y, .. } => (*x, *y),
            };
            if x < -2.0 || y < -2.0 || x > self.w as f32 + 2.0 || y > self.h as f32 + 2.0 {
                return false;
            }
            let r = Rect { x, y, w: 0.35, h: 0.35 };
            !solids.iter().any(|t| r.overlaps(*t))
        });
    }

    fn solid_snapshot(&self) -> Vec<Rect> {
        let mut v = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                let c = self.cells[y * self.w + x];
                if c.kind.blocks(self.flicker_on) {
                    v.push(Rect { x: x as f32, y: y as f32, w: 1.0, h: 1.0 });
                }
            }
        }
        v
    }

    fn tick_pop(&mut self, dt: f32) {
        if self.pops.is_empty() || self.pop_live {
            return;
        }
        let px = self.actors.first().map(|a| a.x + a.w * 0.5).unwrap_or(0.0);
        if !self.pop_armed && px >= self.pop_at {
            self.pop_armed = true;
            self.pop_t = 0.0;
        }
        if self.pop_armed {
            self.pop_t += dt;
            if self.pop_t > 0.05 {
                for &(x, y) in &self.pops.clone() {
                    if let Some(c) = self.cell_mut(x as i32, y as i32) {
                        if c.kind == Kind::Air {
                            c.kind = Kind::Spike;
                        }
                    }
                }
                self.pop_live = true;
            }
        }
    }

    fn tick_flicker(&mut self, dt: f32) {
        if !self.has_flicker {
            return;
        }
        self.flicker_t += dt;
        if self.flicker_t >= self.flicker_period {
            self.flicker_t = 0.0;
            self.flicker_on = !self.flicker_on;
        }
    }

    fn tick_doors(&mut self, dt: f32) {
        for d in &mut self.doors {
            if !d.moving {
                continue;
            }
            let dx = d.tx - d.x;
            let dy = d.ty - d.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 0.04 {
                d.x = d.tx;
                d.y = d.ty;
                d.moving = false;
            } else {
                let step = (d.speed * dt).min(dist);
                d.x += dx / dist * step;
                d.y += dy / dist * step;
            }
        }
    }

    fn mirror_puppet(&mut self) {
        if self.clone != Some(CloneKind::Mirror) || self.actors.len() < 2 {
            return;
        }
        let s = self.actors[0];
        let mut c = s;
        c.puppet = true;
        c.role = 2;
        c.x = self.w as f32 - (s.x + s.w);
        c.alive = true;
        c.cause = Cause::None;
        self.actors[1] = c;
    }

    fn arm_crumbles(&mut self) {
        let samples: Vec<(i32, i32, bool)> = self
            .actors
            .iter()
            .filter(|a| a.alive && a.on_ground && !a.puppet)
            .flat_map(|a| {
                let y = if self.grav >= 0.0 { a.y + a.h + 0.04 } else { a.y - 0.04 };
                let xs = [a.x + 0.08, a.x + a.w * 0.5, a.x + a.w - 0.08];
                xs.into_iter().map(move |x| (x.floor() as i32, y.floor() as i32, true)).collect::<Vec<_>>()
            })
            .collect();
        let solo_delay = self.solo_delay;
        let mut groups = Vec::new();
        for (x, y, _) in samples {
            if let Some(c) = self.cell_at(x, y) {
                if c.kind == Kind::Crumble && c.timer < 0.0 {
                    if c.meta >= 0 {
                        groups.push(c.meta);
                    } else if let Some(c) = self.cell_mut(x, y) {
                        c.timer = solo_delay;
                    }
                }
            }
        }
        groups.sort_unstable();
        groups.dedup();
        if !groups.is_empty() {
            let delay = self.group_delay;
            for c in &mut self.cells {
                if c.kind == Kind::Crumble && c.meta >= 0 && groups.contains(&c.meta) && c.timer < 0.0 {
                    c.timer = delay;
                }
            }
        }
    }

    fn tick_crumbles(&mut self, dt: f32) {
        for c in &mut self.cells {
            if c.kind == Kind::Crumble && c.timer >= 0.0 {
                c.timer -= dt;
                if c.timer <= 0.0 {
                    *c = Cell::air();
                }
            }
        }
    }

    fn push_walls(&mut self) {
        let walls: Vec<(Rect, f32, f32)> = self
            .ents
            .iter()
            .filter_map(|e| match e {
                Ent::Wall { x, y, w, h, ldx, ldy, .. } => {
                    Some((Rect { x: *x, y: *y, w: *w, h: *h }, *ldx, *ldy))
                }
                _ => None,
            })
            .collect();
        for a in &mut self.actors {
            if !a.alive || a.puppet {
                continue;
            }
            for (wall, ldx, ldy) in &walls {
                if !a.body().overlaps(*wall) {
                    continue;
                }
                if *ldx > 0.0 {
                    a.x = wall.x + wall.w + 0.01;
                } else if *ldx < 0.0 {
                    a.x = wall.x - a.w - 0.01;
                } else if *ldy > 0.0 {
                    a.y = wall.y + wall.h + 0.01;
                } else if *ldy < 0.0 {
                    a.y = wall.y - a.h - 0.01;
                } else {
                    a.y = wall.y - a.h - 0.01;
                }
                if a.body().overlaps(*wall) {
                    a.alive = false;
                    a.cause = Cause::Crush;
                }
            }
        }
    }

    fn hazards(&mut self) {
        let spike_rects = self.spike_rects();
        let saws: Vec<Rect> = self
            .ents
            .iter()
            .filter_map(|e| match e {
                Ent::Saw { x, y, .. } => Some(Rect { x: *x, y: *y, w: 0.7, h: 0.7 }),
                _ => None,
            })
            .collect();
        let chases: Vec<Rect> = self
            .ents
            .iter()
            .filter_map(|e| match e {
                Ent::Chase { x, y } => Some(Rect { x: *x, y: *y, w: 0.85, h: 0.85 }),
                _ => None,
            })
            .collect();
        let shots: Vec<(Rect, bool)> = self
            .shots
            .iter()
            .map(|s| match s {
                Shot::Bullet { x, y, .. } => (Rect { x: *x, y: *y, w: 0.35, h: 0.35 }, false),
                Shot::Bomb { x, y, .. } => (Rect { x: *x, y: *y, w: 0.45, h: 0.45 }, true),
            })
            .collect();

        for a in &mut self.actors {
            if !a.alive {
                continue;
            }
            let h = a.hurt();
            if spike_rects.iter().any(|s| h.overlaps(*s)) {
                a.alive = false;
                a.cause = if a.puppet { Cause::Clone } else { Cause::Spike };
            } else if saws.iter().any(|s| h.overlaps(*s)) {
                a.alive = false;
                a.cause = if a.puppet { Cause::Clone } else { Cause::Saw };
            } else if chases.iter().any(|s| h.overlaps(*s)) {
                a.alive = false;
                a.cause = if a.puppet { Cause::Clone } else { Cause::Chase };
            } else if let Some((_, bomb)) = shots.iter().find(|(r, _)| h.overlaps(*r)) {
                a.alive = false;
                a.cause = if *bomb { Cause::Bomb } else { Cause::Shot };
            }
        }
    }

    fn spike_rects(&self) -> Vec<Rect> {
        let mut v = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                let c = self.cells[y * self.w + x];
                if c.kind == Kind::Spike {
                    v.push(Rect { x: x as f32 + 0.08, y: y as f32 + 0.08, w: 0.84, h: 0.84 });
                }
            }
        }
        v
    }

    fn interact(&mut self) {
        let positions: Vec<(usize, Rect, bool)> = self
            .actors
            .iter()
            .enumerate()
            .map(|(i, a)| (i, a.body(), a.alive && !a.puppet))
            .collect();
        for (i, body, alive) in positions {
            if !alive {
                continue;
            }
            self.touch_cells(i, body);
            self.touch_doors(i, body);
        }
    }

    fn touch_cells(&mut self, i: usize, body: Rect) {
        let x0 = body.x.floor() as i32;
        let y0 = body.y.floor() as i32;
        let x1 = (body.x + body.w).floor() as i32;
        let y1 = (body.y + body.h).floor() as i32;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let Some(cell) = self.cell_at(tx, ty) else { continue };
                let tile = Rect { x: tx as f32, y: ty as f32, w: 1.0, h: 1.0 };
                if !body.overlaps(tile) {
                    continue;
                }
                match cell.kind {
                    Kind::Coin => {
                        self.coins += 1;
                        if let Some(c) = self.cell_mut(tx, ty) {
                            *c = Cell::air();
                        }
                    }
                    Kind::Bomb => {
                        self.kill(i, Cause::Coin);
                        self.banner = "it was never a coin";
                        self.banner_t = 1.6;
                    }
                    Kind::Key => {
                        let id = cell.meta as u8;
                        self.got_key = Some(id);
                        if let Some(c) = self.cell_mut(tx, ty) {
                            *c = Cell::air();
                        }
                        self.banner = "a key. suspicious.";
                        self.banner_t = 1.8;
                    }
                    Kind::FakeDoor => {
                        if let Some(c) = self.cell_mut(tx, ty) {
                            c.kind = Kind::Spike;
                        }
                        self.kill(i, Cause::FakeDoor);
                        self.banner = "nope";
                        self.banner_t = 1.4;
                    }
                    Kind::Portal => {
                        if self.actors[i].portal_cd <= 0.0 {
                            self.teleport(i, cell.meta, tx, ty);
                        }
                    }
                    Kind::Grow => self.resize(i, Profile::Wide),
                    Kind::Shrink => self.resize(i, Profile::Small),
                    Kind::TallBtn => self.resize(i, Profile::Tall),
                    Kind::NormBtn => self.resize(i, Profile::Normal),
                    Kind::Pvp => {
                        if let Some(c) = self.cell_mut(tx, ty) {
                            *c = Cell::air();
                        }
                        self.betray(i);
                    }
                    _ => {}
                }
            }
        }
    }

    fn resize(&mut self, i: usize, profile: Profile) {
        if !self.actors[i].alive || self.actors[i].profile == profile {
            return;
        }
        let mut a = self.actors[i];
        let feet = if self.grav >= 0.0 { a.y + a.h } else { a.y };
        let (w, h) = dims(profile);
        let old = a;
        a.w = w;
        a.h = h;
        if self.grav >= 0.0 {
            a.y = feet - h;
        }
        if self.body_in_solid(&a) {
            self.banner = "you do not fit";
            self.banner_t = 1.2;
            self.actors[i] = old;
            return;
        }
        a.profile = profile;
        self.actors[i] = a;
    }

    fn body_in_solid(&self, a: &Actor) -> bool {
        let body = Rect { x: a.x + 0.05, y: a.y + 0.05, w: (a.w - 0.1).max(0.1), h: (a.h - 0.1).max(0.1) };
        let mut hit = false;
        self.for_each_solid(body, false, 0.0, |_| hit = true);
        hit
    }

    fn betray(&mut self, i: usize) {
        let other = if self.actors.len() > 1 { if i == 0 { 1 } else { 0 } } else { i };
        let a = self.actors[other];
        let cx = (a.x + a.w * 0.5).floor() as i32;
        let cy = (a.y + a.h * 0.5).floor() as i32;
        if let Some(c) = self.cell_mut(cx, cy) {
            if c.kind == Kind::Air || c.kind == Kind::Coin {
                c.kind = Kind::Spike;
            }
        }
        self.banner = "bold of you";
        self.banner_t = 1.2;
        if other == i {
            self.kill(i, Cause::Button);
        }
    }

    fn teleport(&mut self, i: usize, meta: i16, from_x: i32, from_y: i32) {
        let mut dest = None;
        for y in 0..self.h {
            for x in 0..self.w {
                let c = self.cells[y * self.w + x];
                if c.kind == Kind::Portal && c.meta == meta && (x as i32 != from_x || y as i32 != from_y) {
                    dest = Some((x as f32, y as f32));
                }
            }
        }
        let Some((dx, dy)) = dest else { return };
        let mut a = self.actors[i];
        let candidates = [(1.0, 0.0), (-1.0, 0.0), (0.0, -1.0), (2.0, 0.0), (0.0, 0.0)];
        for (ox, oy) in candidates {
            a.x = dx + ox + 0.2;
            a.y = dy + oy + 1.0 - a.h;
            a.vx = 0.0;
            a.vy = 0.0;
            if !self.body_in_solid(&a) {
                a.portal_cd = 0.45;
                self.actors[i] = a;
                return;
            }
        }
        a.x = dx + 0.2;
        a.y = dy + 1.0 - a.h;
        a.portal_cd = 0.45;
        self.actors[i] = a;
    }

    fn touch_doors(&mut self, i: usize, body: Rect) {
        for d in &self.doors {
            let r = Rect { x: d.x + 0.12, y: d.y, w: 0.76, h: 1.35 };
            if !body.overlaps(r) {
                continue;
            }
            if self.clone == Some(CloneKind::Dual) {
                if d.id == i as u8 {
                    self.actors[i].reached = true;
                }
            } else if self.versus {
                if self.winner.is_none() {
                    self.winner = Some(i as u8);
                    self.won = true;
                }
            } else if i == 0 && d.id == 0 {
                self.won = true;
            }
        }
    }

    fn kill(&mut self, i: usize, cause: Cause) {
        if self.actors[i].alive {
            self.actors[i].alive = false;
            self.actors[i].cause = cause;
        }
    }

    fn crosses_after(&mut self) {
        let Some(a) = self.actors.first() else { return };
        let x = a.x + a.w * 0.5;
        if let Some(line) = self.reverse_line {
            let past = x >= line;
            if self.reverse_sticky {
                if past {
                    self.reversed = true;
                    self.reverse_line = None;
                }
            } else {
                self.reversed = past;
            }
        }
        let fired: Vec<CrossKind> = self
            .crosses
            .iter_mut()
            .filter(|c| !c.fired && x >= c.at)
            .map(|c| {
                c.fired = true;
                c.kind.clone()
            })
            .collect();
        for kind in fired {
            match kind {
                CrossKind::High(s) => self.jump_scale = s,
                CrossKind::Gravity => self.grav = -1.0,
                CrossKind::JetOff => {
                    self.jet_off = true;
                    self.fuel = 0.0;
                    self.banner = "the pack quits";
                    self.banner_t = 1.5;
                }
                CrossKind::Infinite => {
                    self.infinite = true;
                    self.banner = "the ground is optional";
                    self.banner_t = 1.5;
                }
                CrossKind::Taunt(text) => {
                    self.banner = text;
                    self.banner_t = 2.0;
                }
                CrossKind::MoveDoor { tx, ty, speed } => {
                    if let Some(d) = self.doors.iter_mut().find(|d| d.id == 0) {
                        d.tx = tx;
                        d.ty = ty - 0.45;
                        d.speed = speed;
                        d.moving = true;
                    }
                    self.banner = "the door disagrees";
                    self.banner_t = 1.6;
                }
            }
        }
    }

    fn finish_flags(&mut self) {
        if self.clone == Some(CloneKind::Dual) && self.actors.len() >= 2 && self.actors.iter().all(|a| a.reached) {
            self.won = true;
        }
        if self.versus {
            if self.winner.is_some() {
                self.won = true;
                return;
            }
            if self.actors.iter().all(|a| !a.alive) {
                self.lost = true;
                self.cause = self.actors.iter().find(|a| a.cause != Cause::None).map(|a| a.cause).unwrap_or(Cause::Fall);
            }
            return;
        }
        if self.won {
            return;
        }
        if let Some(a) = self.actors.iter().find(|a| !a.alive) {
            self.lost = true;
            self.cause = a.cause;
        }
    }

    fn cell_at(&self, x: i32, y: i32) -> Option<Cell> {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return None;
        }
        Some(self.cells[y as usize * self.w + x as usize])
    }

    fn cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return None;
        }
        let w = self.w;
        Some(&mut self.cells[y as usize * w + x as usize])
    }

    pub fn run_script(&mut self, steps: &[(f32, i8, bool)]) {
        let frame = 1.0 / 60.0;
        let mut held = false;
        for &(dur, dir, jump) in steps {
            let mut t = 0.0;
            while t < dur - 1e-4 {
                let edge = jump && !held;
                held = jump;
                self.update(frame, &Sticks::one(dir, edge, jump));
                t += frame;
                if self.won || self.lost {
                    return;
                }
            }
        }
    }

    pub fn dump(&self) -> String {
        let mut s = format!(
            "{} won={} lost={} cause={:?} rev={} grav={} fuel={:.2}\n",
            self.name, self.won, self.lost, self.cause, self.reversed, self.grav, self.fuel
        );
        for (i, a) in self.actors.iter().enumerate() {
            s.push_str(&format!(
                "  actor{i} x={:.2} y={:.2} vx={:.2} vy={:.2} gnd={} alive={} cause={:?}\n",
                a.x, a.y, a.vx, a.vy, a.on_ground, a.alive, a.cause
            ));
        }
        for y in 0..self.h {
            for x in 0..self.w {
                let mark = self.actors.iter().enumerate().find_map(|(i, a)| {
                    let cx = (a.x + a.w * 0.5).floor() as usize;
                    let cy = (a.y + a.h * 0.5).floor() as usize;
                    if cx == x && cy == y {
                        Some(if i == 0 { '@' } else { '&' })
                    } else {
                        None
                    }
                });
                if let Some(ch) = mark {
                    s.push(ch);
                    continue;
                }
                let c = self.cells[y * self.w + x];
                let ch = match c.kind {
                    Kind::Air => ' ',
                    Kind::Solid | Kind::Fake => '#',
                    Kind::Ghost => '.',
                    Kind::Flicker => {
                        if self.flicker_on {
                            '#'
                        } else {
                            ' '
                        }
                    }
                    Kind::Spike => '^',
                    Kind::Crumble => {
                        if c.timer >= 0.0 && c.timer < 0.12 {
                            '%'
                        } else {
                            '#'
                        }
                    }
                    Kind::Ice => '~',
                    Kind::Spring | Kind::Super => 'B',
                    Kind::OneWay => '=',
                    Kind::FakeDoor => 'd',
                    Kind::Coin | Kind::Bomb => 'o',
                    Kind::Key => 'K',
                    Kind::Portal => 'O',
                    Kind::Grow => '+',
                    Kind::Shrink => '-',
                    Kind::TallBtn => 't',
                    Kind::NormBtn => 'n',
                    Kind::Pvp => '!',
                    Kind::GunR => '>',
                    Kind::GunL => '<',
                    Kind::BombGun => 'X',
                };
                s.push(ch);
            }
            s.push('\n');
        }
        s
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn rehearse(def: &LevelDef, steps: &[(f32, i8, bool)]) -> Result<(), String> {
    let mut sim = Sim::boot(def, &[], false).map_err(|e| e)?;
    sim.run_script(steps);
    if sim.won {
        Ok(())
    } else {
        Err(format!("failed to clear\n{}", sim.dump()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LevelDef;

    fn def(map: &'static str, traps: &'static [Trap]) -> LevelDef {
        LevelDef { world: 0, door: 0, stage: 0, name: "t", hint: "", key: None, map, traps }
    }

    fn settle(map: &'static str, traps: &'static [Trap]) -> Sim {
        let mut s = Sim::boot(&def(map, traps), &[], false).unwrap();
        s.run_script(&[(0.6, 0, false)]);
        s
    }

    #[test]
    fn stands_on_floor() {
        let s = settle("@D\n##", &[]);
        assert!(s.actors[0].on_ground, "{}", s.dump());
        assert!(!s.lost, "{}", s.dump());
        assert!((s.actors[0].y - (1.0 - 0.84)).abs() < 0.08, "{}", s.dump());
    }

    #[test]
    fn jump_forward_clears_a_pit() {
        let map = "@       D\n####   ##";
        let mut s = Sim::boot(&def(map, &[]), &[], false).unwrap();
        s.run_script(&[(0.2, 1, false), (0.5, 1, true), (1.4, 1, false)]);
        assert!(s.actors[0].x > 5.0, "forward jump should carry across the hole\n{}", s.dump());
        assert!(s.won, "{}", s.dump());
    }

    #[test]
    fn walks_into_door() {
        let mut s = Sim::boot(&def("@     D\n#######", &[]), &[], false).unwrap();
        s.run_script(&[(2.0, 1, false)]);
        assert!(s.won, "{}", s.dump());
    }

    #[test]
    fn spike_kills() {
        let mut s = Sim::boot(&def("@  ^  D\n#######", &[]), &[], false).unwrap();
        s.run_script(&[(2.0, 1, false)]);
        assert!(s.lost, "{}", s.dump());
        assert_eq!(s.cause, Cause::Spike);
    }

    #[test]
    fn fake_floor_drops() {
        let mut s = Sim::boot(&def("@ D\nFFF", &[]), &[], false).unwrap();
        s.run_script(&[(0.8, 0, false)]);
        assert!(s.lost, "{}", s.dump());
        assert_eq!(s.cause, Cause::Fall);
    }

    #[test]
    fn ghost_holds() {
        let s = settle("@D\nGG", &[]);
        assert!(s.actors[0].on_ground, "{}", s.dump());
        assert!(!s.lost);
    }

    #[test]
    fn group_crumble_if_you_walk() {
        let mut s = Sim::boot(&def("@        D\n###MMMM###", &[]), &[], false).unwrap();
        s.run_script(&[(2.5, 1, false)]);
        assert!(s.lost, "{}", s.dump());
    }

    #[test]
    fn reverse_sends_you_left() {
        let traps: &[Trap] = &[Trap::Reverse { at: 0.0, sticky: true }];
        let mut s = Sim::boot(&def("#@    D\n#######", traps), &[], false).unwrap();
        let x0 = s.actors[0].x;
        s.run_script(&[(0.4, 1, false)]);
        assert!(s.actors[0].x <= x0 + 0.05, "pressed right, should not move right\n{}", s.dump());
    }

    #[test]
    fn spring_launches() {
        let mut s = Sim::boot(&def("@D\nB#\n##", &[]), &[], false).unwrap();
        s.update(1.0 / 60.0, &Sticks::one(0, false, false));
        // fall onto the spring over a few frames
        for _ in 0..40 {
            s.update(1.0 / 60.0, &Sticks::one(0, false, false));
        }
        assert!(s.actors[0].vy < -4.0 || s.actors[0].y < 0.2, "{}", s.dump());
    }
}

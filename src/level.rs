//! Level definitions and map parsing.

use crate::model::{Cell, CloneKind, Kind, Profile};

#[derive(Clone, Copy, Debug)]
pub enum Trap {
    Delays { group: f32, solo: f32 },
    PopLead(f32),
    PopAt(f32),
    Reverse { at: f32, sticky: bool },
    HighJump { at: f32, scale: f32 },
    Gravity { at: f32 },
    Jetpack(f32),
    JetpackOff { at: f32 },
    Infinite { at: f32 },
    JumpLimit(u8),
    Chase { row: u16, speed: f32 },
    Platform { x: f32, y: f32, w: f32, range: f32, speed: f32, vertical: bool },
    Wall { x: f32, y: f32, w: f32, h: f32, tx: f32, ty: f32, speed: f32, at: f32 },
    MoveDoor { at: f32, tx: f32, ty: f32, speed: f32 },
    FuelDrain { x: f32, y: f32, w: f32, h: f32, rate: f32 },
    Taunt { at: f32, text: &'static str },
    FlickerPeriod(f32),
    Clone(CloneKind),
    Start(Profile),
    Lie,
    NoSkip,
    Peace,
}

#[derive(Clone, Copy, Debug)]
pub struct LevelDef {
    pub world: u8,
    pub door: u8,
    pub stage: u8,
    pub name: &'static str,
    pub hint: &'static str,
    pub key: Option<u8>,
    pub map: &'static str,
    pub traps: &'static [Trap],
}

#[derive(Clone, Debug)]
pub struct Built {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
    pub spawns: Vec<(f32, f32, u8)>,
    pub doors: Vec<(f32, f32, u8)>,
    pub saws: Vec<(f32, f32)>,
    pub plats: Vec<(f32, f32, f32)>,
    pub pops: Vec<(u16, u16)>,
}

pub fn build(def: &LevelDef) -> Result<Built, String> {
    let text = def.map.trim_matches('\n');
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Err(format!("{}: empty map", def.name));
    }
    let w = lines[0].chars().count();
    if w == 0 || w > 96 {
        return Err(format!("{}: bad width {w}", def.name));
    }
    let mut rows: Vec<Vec<char>> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() != w {
            return Err(format!(
                "{}: line {} width {} != {w} [{line}]",
                def.name,
                i,
                chars.len()
            ));
        }
        rows.push(chars);
    }
    let h = rows.len();
    if h > 40 {
        return Err(format!("{}: too tall", def.name));
    }

    let mut cells = vec![Cell::air(); w * h];
    let mut spawns = Vec::new();
    let mut doors = Vec::new();
    let mut saws = Vec::new();
    let mut plat_cells: Vec<(usize, usize)> = Vec::new();
    let mut pops = Vec::new();
    let mut portal_n: [usize; 3] = [0; 3];
    let (pw, ph) = crate::model::dims(Profile::Normal);

    let at = |x: usize, y: usize| y * w + x;

    for y in 0..h {
        for x in 0..w {
            let ch = rows[y][x];
            let cell = &mut cells[at(x, y)];
            match ch {
                ' ' | '.' => {}
                '#' => cell.kind = Kind::Solid,
                'C' => {
                    cell.kind = Kind::Crumble;
                    cell.meta = -1;
                }
                'M' => {
                    cell.kind = Kind::Crumble;
                    cell.meta = 0;
                }
                'F' => cell.kind = Kind::Fake,
                'G' => cell.kind = Kind::Ghost,
                'f' => cell.kind = Kind::Flicker,
                '^' => cell.kind = Kind::Spike,
                'v' => {
                    cell.kind = Kind::Spike;
                    cell.meta = 1;
                }
                '~' => cell.kind = Kind::Ice,
                'B' => cell.kind = Kind::Spring,
                'b' => cell.kind = Kind::Super,
                '=' => cell.kind = Kind::OneWay,
                '*' => cell.kind = Kind::Coin,
                'o' => cell.kind = Kind::Bomb,
                'K' => {
                    cell.kind = Kind::Key;
                    cell.meta = def.key.unwrap_or(0) as i16;
                }
                'd' => cell.kind = Kind::FakeDoor,
                '+' => cell.kind = Kind::Grow,
                '-' => cell.kind = Kind::Shrink,
                't' => cell.kind = Kind::TallBtn,
                'n' => cell.kind = Kind::NormBtn,
                '!' => cell.kind = Kind::Pvp,
                '>' => {
                    cell.kind = Kind::GunR;
                    cell.timer = 0.85;
                }
                '<' => {
                    cell.kind = Kind::GunL;
                    cell.timer = 0.85;
                }
                'X' => {
                    cell.kind = Kind::BombGun;
                    cell.timer = 1.1;
                }
                ',' => pops.push((x as u16, y as u16)),
                'A' => saws.push((x as f32, y as f32)),
                '_' => plat_cells.push((x, y)),
                '@' | '1' | '2' => {
                    let role = match ch {
                        '1' => 1,
                        '2' => 2,
                        _ => 0,
                    };
                    let sx = x as f32 + (1.0 - pw) * 0.5;
                    let sy = y as f32 + 1.0 - ph;
                    spawns.push((sx, sy, role));
                }
                'D' => doors.push((x as f32, y as f32 - 0.45, 0)),
                'E' => doors.push((x as f32, y as f32 - 0.45, 1)),
                'p' | 'q' | 'r' => {
                    let slot = match ch {
                        'p' => 0,
                        'q' => 1,
                        _ => 2,
                    };
                    let n = portal_n[slot];
                    portal_n[slot] = n + 1;
                    cell.kind = Kind::Portal;
                    cell.meta = (slot * 10 + n / 2) as i16;
                }
                other => {
                    return Err(format!("{}: unknown '{other}' at {x},{y}", def.name));
                }
            }
        }
    }

    for slot in 0..3 {
        if portal_n[slot] != 0 && portal_n[slot] != 2 {
            return Err(format!(
                "{}: portal group {slot} has {} mouths (need 2)",
                def.name, portal_n[slot]
            ));
        }
    }

    let key_tiles = cells.iter().filter(|c| c.kind == Kind::Key).count();
    if key_tiles > 1 {
        return Err(format!("{}: more than one key", def.name));
    }
    if key_tiles == 1 && def.key.is_none() {
        return Err(format!("{}: map has K but no key id", def.name));
    }
    if key_tiles == 0 && def.key.is_some() {
        return Err(format!("{}: key id set but no K", def.name));
    }
    if spawns.iter().filter(|s| s.2 == 0).count() != 1 {
        return Err(format!("{}: need exactly one @", def.name));
    }
    if doors.is_empty() {
        return Err(format!("{}: need a door", def.name));
    }

    let mut plats = Vec::new();
    plat_cells.sort_unstable();
    let mut i = 0;
    while i < plat_cells.len() {
        let (x0, y0) = plat_cells[i];
        let mut x1 = x0;
        i += 1;
        while i < plat_cells.len() && plat_cells[i].1 == y0 && plat_cells[i].0 == x1 + 1 {
            x1 = plat_cells[i].0;
            i += 1;
        }
        plats.push((x0 as f32, y0 as f32, (x1 - x0 + 1) as f32));
    }

    Ok(Built { w, h, cells, spawns, doors, saws, plats, pops })
}

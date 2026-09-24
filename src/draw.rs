//! Framebuffer drawing for the terminal.

use std::io::{stdout, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate};

use crate::model::*;
use crate::sim::{Ent, Shot, Sim};

#[derive(Clone, Copy, PartialEq)]
pub struct Glyph {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    cells: Vec<Glyph>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        let bg = ink(8, 6, 12);
        let g = Glyph { ch: ' ', fg: ink(220, 220, 220), bg };
        Self { w, h, cells: vec![g; w * h] }
    }

    pub fn clear(&mut self, bg: Color) {
        for c in &mut self.cells {
            c.ch = ' ';
            c.bg = bg;
            c.fg = ink(180, 180, 180);
        }
    }

    pub fn put(&mut self, x: i32, y: i32, ch: char, fg: Color, bg: Color) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let i = y as usize * self.w + x as usize;
        self.cells[i] = Glyph { ch, fg, bg };
    }

    pub fn stamp(&mut self, x: i32, y: i32, ch: char, fg: Color) {
        let bg = self.bg_at(x, y);
        self.put(x, y, ch, fg, bg);
    }

    pub fn fill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, ch: char, fg: Color, bg: Color) {
        for y in y0..y1 {
            for x in x0..x1 {
                self.put(x, y, ch, fg, bg);
            }
        }
    }

    pub fn text(&mut self, x: i32, y: i32, s: &str, fg: Color) {
        let bg = self.bg_at(x.max(0), y.max(0));
        for (i, ch) in s.chars().enumerate() {
            self.put(x + i as i32, y, ch, fg, bg);
        }
    }

    pub fn text_bg(&mut self, x: i32, y: i32, s: &str, fg: Color, bg: Color) {
        for (i, ch) in s.chars().enumerate() {
            self.put(x + i as i32, y, ch, fg, bg);
        }
    }

    fn bg_at(&self, x: i32, y: i32) -> Color {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return ink(8, 6, 12);
        }
        self.cells[y as usize * self.w + x as usize].bg
    }

    pub fn hline(&mut self, x: i32, y: i32, n: i32, ch: char, fg: Color) {
        for i in 0..n {
            let bg = self.bg_at(x + i, y);
            self.put(x + i, y, ch, fg, bg);
        }
    }

    pub fn glyph_at(&self, x: i32, y: i32) -> char {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return ' ';
        }
        self.cells[y as usize * self.w + x as usize].ch
    }

    pub fn bg_of(&self, x: i32, y: i32) -> Color {
        self.bg_at(x, y)
    }

    pub fn blit(&self) -> std::io::Result<()> {
        let mut out = stdout();
        queue!(out, BeginSynchronizedUpdate)?;
        for y in 0..self.h {
            queue!(out, MoveTo(0, y as u16))?;
            let row = &self.cells[y * self.w..(y + 1) * self.w];
            let mut i = 0;
            while i < row.len() {
                let g = row[i];
                let mut j = i + 1;
                while j < row.len() && row[j].fg == g.fg && row[j].bg == g.bg {
                    j += 1;
                }
                queue!(out, SetForegroundColor(g.fg), SetBackgroundColor(g.bg))?;
                let s: String = row[i..j].iter().map(|c| c.ch).collect();
                queue!(out, Print(s))?;
                i = j;
            }
            queue!(out, ResetColor, Clear(ClearType::UntilNewLine))?;
        }
        queue!(out, EndSynchronizedUpdate)?;
        out.flush()
    }
}

pub fn ink(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

pub fn backdrop() -> Color {
    ink(152, 107, 6)
}

pub fn room_color() -> Color {
    ink(254, 185, 84)
}

pub fn draw_world(cv: &mut Canvas, sim: &Sim, ox: i32, oy: i32, vw: i32, vh: i32, ascii: bool, shake: f32) {
    if sim.w == 0 || sim.h == 0 || vw < 8 || vh < 6 {
        return;
    }
    let jx = if shake > 0.0 { ((sim.time * 70.0).sin() * shake).round() as i32 } else { 0 };
    let jy = if shake > 0.0 { ((sim.time * 54.0).cos() * shake * 0.4).round() as i32 } else { 0 };
    let room_w = ((vw as f32) * 0.74).round() as i32;
    let room_h = ((vh as f32) * 0.20).clamp(6.0, 11.0).round() as i32;
    let room_x = ox + (vw - room_w) / 2 + jx;
    let mut room_y = oy + ((vh as f32) * 0.46).round() as i32 + jy;
    if room_y + room_h > oy + vh {
        room_y = oy + vh - room_h;
    }
    let stand = stand_y(sim);
    let view = View {
        x: room_x,
        y: room_y,
        w: room_w.max(8),
        h: room_h.max(6),
        mw: sim.w as f32,
        top: stand - 2.6,
        bot: stand + 0.04,
    };

    let brown = backdrop();
    let peach = room_color();
    let black = ink(8, 6, 4);
    cv.fill(view.x, view.y, view.x + view.w, view.y + view.h, ' ', peach, peach);

    for tx in 0..sim.w as i32 {
        let mut held = false;
        for ty in 0..sim.h as i32 {
            let kind = sim.cells[ty as usize * sim.w + tx as usize].kind;
            if is_surface(kind, sim.flicker_on) {
                held = true;
                // A ceiling or a ledge above the floor. The floor itself is the bottom edge of the room.
                if (sim.grav >= 0.0 && (ty as f32) < stand - 0.2) || (sim.grav < 0.0 && (ty as f32 + 1.0) > stand + 0.2) {
                    let (x0, y0, x1, y1) = view.cell(tx, ty);
                    cv.fill(x0, y0, x1, y1, ' ', brown, brown);
                }
            }
        }
        if !held {
            let x0 = view.px(tx as f32);
            let x1 = view.px(tx as f32 + 1.0).max(x0 + 1);
            cv.fill(x0, view.y, x1, view.y + view.h, ' ', brown, brown);
        }
    }

    for ty in 0..sim.h as i32 {
        for tx in 0..sim.w as i32 {
            let cell = sim.cells[ty as usize * sim.w + tx as usize];
            paint_prop(cv, &view, tx, ty, cell.kind, cell.meta, ascii, black, peach);
        }
    }

    for e in &sim.ents {
        match e {
            Ent::Saw { x, y, .. } => {
                if ascii {
                    let (x0, y0, _, _) = view.rect(*x, *y, 0.85, 0.85);
                    cv.put(x0, y0, 'o', black, peach);
                } else {
                    let (x0, y0, x1, y1) = view.frect(*x, *y, 0.85, 0.85);
                    paint_subrect(cv, x0, y0, x1, y1, black);
                }
            }
            Ent::Plat { x, y, w, h, .. } => {
                let (x0, y0, x1, y1) = view.frect(*x, *y, *w, (*h).max(0.28));
                paint_subrect(cv, x0, y0, x1, y1.max(y0 + 0.35), ink(232, 164, 86));
            }
            Ent::Wall { x, y, w, h, .. } => {
                let (x0, y0, x1, y1) = view.rect(*x, *y, *w, *h);
                cv.fill(x0, y0, x1, y1, if ascii { '#' } else { '█' }, ink(120, 36, 44), ink(120, 36, 44));
            }
            Ent::Chase { x, y } => {
                if ascii {
                    let (x0, y0, _, _) = view.rect(*x, *y, 0.9, 0.9);
                    cv.put(x0, y0, '>', black, peach);
                } else {
                    let (x0, y0, x1, y1) = view.frect(*x, *y, 0.9, 0.9);
                    paint_subrect(cv, x0, y0, x1, y1, black);
                }
            }
        }
    }

    for d in &sim.doors {
        paint_door(cv, &view, d.x, d.y + 0.45, sim.peace, ascii);
    }

    for s in &sim.shots {
        let (x, y) = match s {
            Shot::Bullet { x, y, .. } | Shot::Bomb { x, y, .. } => (*x, *y),
        };
        if ascii {
            cv.put(view.px(x), view.py(y), '*', black, peach);
        } else {
            let (x0, y0, x1, y1) = view.frect(x, y, 0.4, 0.35);
            paint_subrect(cv, x0, y0, x1, y1.max(y0 + 0.4), black);
        }
    }

    for a in &sim.actors {
        paint_actor(cv, sim, &view, a, ascii, black, peach);
    }
}

struct View {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    mw: f32,
    top: f32,
    bot: f32,
}

impl View {
    fn fx(&self, wx: f32) -> f32 {
        self.x as f32 + wx / self.mw * self.w as f32
    }
    fn fy(&self, wy: f32) -> f32 {
        let t = (wy - self.top) / (self.bot - self.top).max(0.01);
        self.y as f32 + t * self.h as f32
    }
    fn px(&self, wx: f32) -> i32 {
        self.fx(wx).round() as i32
    }
    fn py(&self, wy: f32) -> i32 {
        self.fy(wy).round() as i32
    }
    fn frect(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
        (self.fx(x), self.fy(y), self.fx(x + w), self.fy(y + h))
    }
    fn cell(&self, tx: i32, ty: i32) -> (i32, i32, i32, i32) {
        self.rect(tx as f32, ty as f32, 1.0, 1.0)
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> (i32, i32, i32, i32) {
        let x0 = self.px(x);
        let y0 = self.py(y);
        let x1 = self.px(x + w).max(x0 + 1);
        let y1 = self.py(y + h).max(y0 + 1);
        (x0, y0, x1, y1)
    }
}

fn is_surface(kind: Kind, flicker_on: bool) -> bool {
    matches!(kind, Kind::Solid | Kind::Fake | Kind::Crumble | Kind::Ice | Kind::OneWay | Kind::Spring | Kind::Super)
        || (kind == Kind::Flicker && flicker_on)
}

/// World y the player stands on. The peach room ends on this line.
fn stand_y(sim: &Sim) -> f32 {
    if sim.grav < 0.0 {
        for ty in 0..sim.h as i32 {
            for tx in 0..sim.w as i32 {
                let kind = sim.cells[ty as usize * sim.w + tx as usize].kind;
                if is_surface(kind, sim.flicker_on) {
                    return ty as f32 + 1.0;
                }
            }
        }
        return 1.0;
    }
    for ty in (0..sim.h as i32).rev() {
        for tx in 0..sim.w as i32 {
            let kind = sim.cells[ty as usize * sim.w + tx as usize].kind;
            if is_surface(kind, sim.flicker_on) {
                return ty as f32;
            }
        }
    }
    1.0
}

fn paint_prop(cv: &mut Canvas, view: &View, tx: i32, ty: i32, kind: Kind, meta: i16, ascii: bool, black: Color, peach: Color) {
    let (x0, y0, x1, y1) = view.cell(tx, ty);
    let mark = |cv: &mut Canvas, ch: char, fg: Color| {
        let mx = (x0 + x1) / 2;
        let my = (y0 + y1) / 2;
        cv.fill(x0, y0, x1, y1, ' ', peach, peach);
        cv.put(mx, my, ch, fg, peach);
    };
    match kind {
        Kind::Spike => {
            cv.fill(x0, y0, x1, y1, if meta == 1 { if ascii { 'v' } else { '▼' } } else { if ascii { '^' } else { '▲' } }, black, peach);
        }
        Kind::Coin | Kind::Bomb => mark(cv, if ascii { 'o' } else { '●' }, ink(120, 72, 16)),
        Kind::Key => mark(cv, if ascii { 'k' } else { '◆' }, black),
        Kind::Portal => mark(cv, if ascii { 'O' } else { '○' }, black),
        Kind::Spring | Kind::Super => mark(cv, if ascii { '^' } else { '⌃' }, black),
        Kind::FakeDoor => paint_door(cv, view, tx as f32, ty as f32, false, ascii),
        Kind::Grow => mark(cv, '+', black),
        Kind::Shrink => mark(cv, '-', black),
        Kind::TallBtn | Kind::NormBtn | Kind::Pvp => mark(cv, if ascii { '!' } else { '▪' }, black),
        Kind::GunR => mark(cv, if ascii { '>' } else { '►' }, black),
        Kind::GunL => mark(cv, if ascii { '<' } else { '◄' }, black),
        Kind::BombGun => mark(cv, if ascii { 'x' } else { '✸' }, black),
        _ => {}
    }
}

fn paint_door(cv: &mut Canvas, view: &View, x: f32, y: f32, peace: bool, _ascii: bool) {
    let floor = view.py(y + 1.0).clamp(view.y + 2, view.y + view.h);
    let cx = view.px(x + 0.5);
    let w = 3;
    let h = 2;
    let x0 = cx - w / 2;
    let y0 = floor - h;
    let frame = if peace { ink(110, 70, 160) } else { ink(168, 172, 176) };
    let fill = if peace { ink(190, 150, 230) } else { ink(214, 216, 214) };
    cv.fill(x0, y0, x0 + w, floor, '█', frame, frame);
    cv.fill(x0 + 1, y0 + 1, x0 + w - 1, floor, '█', fill, fill);
}

fn paint_actor(cv: &mut Canvas, sim: &Sim, view: &View, a: &crate::sim::Actor, ascii: bool, black: Color, peach: Color) {
    let cx = view.fx(a.x + a.w * 0.5);
    let feet = view.fy(a.y + a.h);
    if ascii {
        cv.put(cx.round() as i32, feet.round() as i32 - 1, if a.alive { '@' } else { 'x' }, black, peach);
        return;
    }
    if !a.alive && sim.lost {
        cv.put(cx.round() as i32, feet.round() as i32 - 1, 'x', black, peach);
        return;
    }
    let body_w = (0.9 * (a.w / 0.56)).clamp(0.55, 3.6);
    let body_h = (2.0 * (a.h / 0.84)).clamp(1.2, 4.2);
    if !a.on_ground {
        if let Some(ground) = stand_line(sim, a) {
            let sy = view.fy(ground);
            paint_subrect(cv, cx - body_w * 0.7, sy - 0.14, cx + body_w * 0.7, sy, black);
        }
    }
    paint_subrect(cv, cx - body_w * 0.5, feet - body_h, cx + body_w * 0.5, feet, black);
}

/// Draw a rectangle in fractional cell coordinates so motion slides instead of popping a whole cell.
fn paint_subrect(cv: &mut Canvas, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let left = x0.floor() as i32;
    let right = x1.ceil() as i32;
    let top = y0.floor() as i32;
    let bot = y1.ceil() as i32;
    for cy in top..bot {
        for cx in left..right {
            let lx = (x0 - cx as f32).max(0.0);
            let rx = (x1 - cx as f32).min(1.0);
            let ty = (y0 - cy as f32).max(0.0);
            let by = (y1 - cy as f32).min(1.0);
            if rx - lx < 0.06 || by - ty < 0.06 {
                continue;
            }
            let Some((ch, invert)) = best_block(lx, ty, rx, by) else { continue };
            let bg = cv.bg_of(cx, cy);
            if ch == '█' {
                cv.put(cx, cy, '█', color, color);
            } else if invert {
                cv.put(cx, cy, ch, bg, color);
            } else {
                cv.put(cx, cy, ch, color, bg);
            }
        }
    }
}

fn best_block(x0: f32, y0: f32, x1: f32, y1: f32) -> Option<(char, bool)> {
    let target = mask_rect(x0, y0, x1, y1);
    if target.count_ones() < 4 {
        return None;
    }
    let mut best: Option<(char, bool)> = None;
    let mut best_dist = u32::MAX;
    for &(ch, invert, mask) in blocks() {
        let dist = (target ^ mask).count_ones();
        if dist < best_dist {
            best_dist = dist;
            best = Some((ch, invert));
        }
    }
    best
}

fn mask_rect(x0: f32, y0: f32, x1: f32, y1: f32) -> u64 {
    let mut m = 0u64;
    for sy in 0..8 {
        for sx in 0..8 {
            let px = (sx as f32 + 0.5) / 8.0;
            let py = (sy as f32 + 0.5) / 8.0;
            if px >= x0 && px < x1 && py >= y0 && py < y1 {
                m |= 1u64 << (sy * 8 + sx);
            }
        }
    }
    m
}

fn blocks() -> &'static [(char, bool, u64)] {
    use std::sync::OnceLock;
    static G: OnceLock<Vec<(char, bool, u64)>> = OnceLock::new();
    G.get_or_init(|| {
        let left = ['▏', '▎', '▍', '▌', '▋', '▊', '▉'];
        let low = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];
        let mut out = Vec::new();
        let mut push = |ch: char, invert: bool, x0: f32, y0: f32, x1: f32, y1: f32| {
            out.push((ch, invert, mask_rect(x0, y0, x1, y1)));
        };
        push('█', false, 0.0, 0.0, 1.0, 1.0);
        for n in 1..=7 {
            let t = n as f32 / 8.0;
            let e = (8 - n) as f32 / 8.0;
            push(left[n - 1], false, 0.0, 0.0, t, 1.0);
            push(left[7 - n], true, e, 0.0, 1.0, 1.0);
            push(low[n - 1], false, 0.0, e, 1.0, 1.0);
            push(low[7 - n], true, 0.0, 0.0, 1.0, t);
        }
        push('▘', false, 0.0, 0.0, 0.5, 0.5);
        push('▝', false, 0.5, 0.0, 1.0, 0.5);
        push('▖', false, 0.0, 0.5, 0.5, 1.0);
        push('▗', false, 0.5, 0.5, 1.0, 1.0);
        out
    })
}

fn stand_line(sim: &Sim, a: &crate::sim::Actor) -> Option<f32> {
    let x = (a.x + a.w * 0.5).floor() as i32;
    if x < 0 || x >= sim.w as i32 {
        return None;
    }
    let start = if sim.grav >= 0.0 { (a.y + a.h).floor() as i32 } else { a.y.floor() as i32 };
    if sim.grav >= 0.0 {
        for ty in start.max(0)..sim.h as i32 {
            let kind = sim.cells[ty as usize * sim.w + x as usize].kind;
            if is_surface(kind, sim.flicker_on) {
                return Some(ty as f32);
            }
        }
    }
    None
}

/// Pause bars, restart button, and the five stage squares.
pub fn draw_hud(cv: &mut Canvas, stage: u8) {
    let red = ink(109, 20, 1);
    let gold = ink(196, 156, 64);
    let brown = backdrop();
    let black = ink(12, 8, 4);
    cv.fill(2, 1, 4, 4, '█', red, red);
    cv.fill(5, 1, 7, 4, '█', red, red);
    cv.fill(9, 1, 14, 4, '█', red, red);
    cv.fill(10, 2, 13, 3, '█', gold, gold);
    cv.put(11, 2, '↻', red, gold);

    let slots = 5i32;
    let pip_w = 4;
    let gap = 1;
    let span = slots * pip_w + (slots - 1) * gap;
    let x0 = ((cv.w as i32 - span) / 2).max(16);
    for i in 0..slots {
        let x = x0 + i * (pip_w + gap);
        let on = i == stage as i32;
        if on {
            cv.fill(x, 0, x + pip_w, 2, '█', black, black);
        } else {
            cv.put(x, 0, '╔', black, brown);
            cv.put(x + 1, 0, '═', black, brown);
            cv.put(x + 2, 0, '═', black, brown);
            cv.put(x + 3, 0, '╗', black, brown);
            cv.put(x, 1, '╚', black, brown);
            cv.put(x + 1, 1, '═', black, brown);
            cv.put(x + 2, 1, '═', black, brown);
            cv.put(x + 3, 1, '╝', black, brown);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LevelDef;
    use crate::sim::{Sim, Sticks};

    fn stage() -> LevelDef {
        LevelDef {
            world: 0,
            door: 0,
            stage: 0,
            name: "view",
            hint: "",
            key: None,
            map: " @                          D           \n##########MMMMM#########################",
            traps: &[],
        }
    }

    fn top_player(cv: &Canvas) -> i32 {
        (0..cv.h as i32)
            .find(|y| (0..cv.w as i32 / 3).any(|x| cv.glyph_at(x, *y) != ' '))
            .unwrap_or(cv.h as i32)
    }

    #[test]
    fn room_sits_in_the_frame_and_a_jump_rises() {
        let mut sim = Sim::boot(&stage(), &[], false).unwrap();
        let mut cv = Canvas::new(100, 30);
        cv.clear(backdrop());
        draw_world(&mut cv, &sim, 0, 0, 100, 30, false, 0.0);
        let peach = (0..30).filter(|y| (0..100).any(|x| cv.bg_of(x, *y) == room_color())).count();
        assert!(peach > 4, "the room should be a band, got {peach} rows");
        assert_eq!(cv.bg_of(0, 0), backdrop(), "the frame stays brown");
        let before = top_player(&cv);
        sim.update(0.28, &Sticks::one(0, true, true));
        let mut jumped = Canvas::new(100, 30);
        jumped.clear(backdrop());
        draw_world(&mut jumped, &sim, 0, 0, 100, 30, false, 0.0);
        let after = top_player(&jumped);
        assert!(after < before, "jump should move the figure up ({after} vs {before})");
        assert!(!sim.actors[0].on_ground);
    }

    #[test]
    fn player_slides_between_cells() {
        let mut sim = Sim::boot(&stage(), &[], false).unwrap();
        let mut before = Canvas::new(100, 30);
        before.clear(backdrop());
        draw_world(&mut before, &sim, 0, 0, 100, 30, false, 0.0);
        sim.actors[0].x += 0.2;
        sim.actors[0].y -= 0.15;
        let mut after = Canvas::new(100, 30);
        after.clear(backdrop());
        draw_world(&mut after, &sim, 0, 0, 100, 30, false, 0.0);
        let picture = |cv: &Canvas| {
            (0..cv.h as i32)
                .flat_map(|y| (0..cv.w as i32).map(move |x| cv.glyph_at(x, y)))
                .collect::<String>()
        };
        assert_ne!(picture(&before), picture(&after), "a fraction of a cell should move the figure");
    }
}

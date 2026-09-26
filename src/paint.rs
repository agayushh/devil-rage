//! Canvas drawing for the browser build.

use web_sys::CanvasRenderingContext2d;

use crate::campaign::{self, SECRET, WORLDS};
use crate::game::{Game, Phase, Scene};
use crate::model::{Kind, SKIP_AFTER};
use crate::sim::{Ent, Shot, Sim};
use crate::view::{self, Frame};

const BROWN: &str = "#986b06";
const PEACH: &str = "#feb954";
const INK: &str = "#140c08";
const GOLD: &str = "#ffd65a";
const CREAM: &str = "#fff6e8";
const DIM: &str = "#6a4518";
const RED: &str = "#8f1d12";

#[derive(Clone, Copy)]
pub enum Act {
    Title(usize),
    Settings(usize),
    Door { world: usize, door: usize },
    Secret,
    Pause(usize),
    HudPause,
    HudRestart,
    Advance,
}

pub struct Hit {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub act: Act,
}

pub fn paint(ctx: &CanvasRenderingContext2d, game: &Game, w: f32, h: f32, hits: &mut Vec<Hit>) {
    hits.clear();
    ctx.set_fill_style_str("#4a2e08");
    ctx.fill_rect(0.0, 0.0, w as f64, h as f64);
    match &game.scene {
        Scene::Title { sel } => paint_title(ctx, game, *sel, w, h, hits),
        Scene::Settings { sel } => paint_settings(ctx, game, *sel, w, h, hits),
        Scene::Map { world, door, secret } => paint_map(ctx, game, *world, *door, *secret, w, h, hits),
        Scene::Play(_) => paint_play(ctx, game, w, h, hits),
        Scene::Ending { secret } => paint_ending(ctx, game, *secret, w, h, hits),
    }
    if game.toast_t > 0.0 {
        text(ctx, &game.toast, w * 0.5, h - 22.0, 16.0, GOLD, "center");
    }
    if game.flash > 0.0 {
        ctx.set_global_alpha((game.flash * 2.4).min(0.55) as f64);
        ctx.set_fill_style_str("#ffd0cc");
        ctx.fill_rect(0.0, 0.0, w as f64, h as f64);
        ctx.set_global_alpha(1.0);
    }
}

fn paint_title(ctx: &CanvasRenderingContext2d, game: &Game, sel: usize, w: f32, h: f32, hits: &mut Vec<Hit>) {
    let m = 16.0;
    panel(ctx, m, m, w - m * 2.0, h - m * 2.0);
    let cx = w * 0.5;
    let items = game.title_items();
    let n = items.len().max(1) as f32;
    let bh = 58.0_f32.min((h * 0.09).max(46.0));
    let gap = 14.0;
    let block = 148.0 + n * bh + (n - 1.0) * gap;
    let y = ((h - block) * 0.46).clamp(28.0, h * 0.34);
    tri(ctx, cx - 70.0, y + 36.0, cx - 28.0, y - 18.0, cx - 8.0, y + 42.0, RED);
    tri(ctx, cx + 70.0, y + 36.0, cx + 28.0, y - 18.0, cx + 8.0, y + 42.0, RED);
    text(ctx, "LEVEL DEVIL", cx, y + 78.0, (w * 0.055).clamp(34.0, 68.0), RED, "center");
    text(ctx, crate::model::title_line(game.time), cx, y + 118.0, 18.0, INK, "center");
    let bw = (w * 0.36).clamp(220.0, 380.0);
    let bx = cx - bw * 0.5;
    let start = y + 148.0;
    for (i, item) in items.iter().enumerate() {
        let by = start + i as f32 * (bh + gap);
        button(ctx, hits, bx, by, bw, bh, item, i == sel, Act::Title(i));
    }
}

fn paint_settings(ctx: &CanvasRenderingContext2d, game: &Game, sel: usize, w: f32, h: f32, hits: &mut Vec<Hit>) {
    let pw = (w * 0.8).clamp(280.0, 480.0);
    let ph = 420.0_f32.min(h - 24.0);
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.45;
    panel(ctx, x, y, pw, ph);
    text(ctx, "SETTINGS", x + pw * 0.5, y + 36.0, 22.0, RED, "center");
    let rows = [
        format!("hints: {}", onoff(game.save.hints)),
        format!("death bell: {}", onoff(game.save.bell)),
        format!("plain shapes: {}", onoff(game.save.ascii)),
        if game.save.secret { "true door: enter".into() } else { "true door: locked".into() },
        "back".into(),
    ];
    let bw = pw - 64.0;
    for (i, row) in rows.iter().enumerate() {
        let by = y + 68.0 + i as f32 * 58.0;
        button(ctx, hits, x + 32.0, by, bw, 48.0, row, i == sel, Act::Settings(i));
    }
    text(
        ctx,
        &format!("keys {}/{}    ·    the arrow keys have a rhythm", game.save.keys.len(), crate::model::KEY_TOTAL),
        x + pw * 0.5,
        y + ph - 22.0,
        13.0,
        DIM,
        "center",
    );
}

fn paint_map(
    ctx: &CanvasRenderingContext2d,
    game: &Game,
    world: usize,
    door: usize,
    secret: bool,
    w: f32,
    h: f32,
    hits: &mut Vec<Hit>,
) {
    text(ctx, "DOORS", 28.0, 28.0, 22.0, CREAM, "left");
    text(
        ctx,
        &format!("keys {}/{}    deaths {}", game.save.keys.len(), crate::model::KEY_TOTAL, game.save.deaths),
        w - 28.0,
        28.0,
        14.0,
        GOLD,
        "right",
    );
    let mut y = 64.0;
    for (wi, info) in WORLDS.iter().enumerate() {
        let active = !secret && wi == world;
        text(ctx, info.name, 28.0, y + 12.0, 18.0, if active { GOLD } else { CREAM }, "left");
        text(ctx, info.tag, 28.0, y + 34.0, 13.0, "#f0d7a4", "left");
        y += 52.0;
        let mut x = 28.0;
        for (di, name) in info.doors.iter().enumerate() {
            let open = game.save.unlocked(wi as u8, di as u8);
            let current = !game.save.cleared && game.save.world == wi as u8 && game.save.door == di as u8;
            let label = if current {
                format!("{name} {}/{}", game.save.stage + 1, campaign::stage_count(wi as u8, di as u8))
            } else if open {
                (*name).to_string()
            } else {
                format!("{name} · shut")
            };
            let bw = (label.len() as f32 * 8.6 + 36.0).clamp(96.0, 210.0);
            if x + bw > w - 20.0 {
                x = 28.0;
                y += 52.0;
            }
            let hot = active && di == door;
            button(ctx, hits, x, y, bw, 44.0, &label, hot, Act::Door { world: wi, door: di });
            x += bw + 8.0;
        }
        y += 64.0;
    }
    if y < h - 70.0 {
        let label = if game.save.secret { "true door" } else { "true door locked" };
        button(ctx, hits, 28.0, y, 200.0, 44.0, label, secret, Act::Secret);
    }
    text(ctx, "esc returns to the title", 28.0, h - 18.0, 13.0, CREAM, "left");
}

fn paint_ending(ctx: &CanvasRenderingContext2d, game: &Game, secret: bool, w: f32, h: f32, hits: &mut Vec<Hit>) {
    let pw = (w * 0.8).clamp(280.0, 560.0);
    let ph = 300.0_f32.min(h - 40.0);
    let x = (w - pw) * 0.5;
    let y = (h - ph) * 0.42;
    panel(ctx, x, y, pw, ph);
    let title = if secret { "TRUE ENDING" } else { "ENDING" };
    text(ctx, title, x + pw * 0.5, y + 40.0, 26.0, if secret { "#5b2d82" } else { RED }, "center");
    let lines: &[&str] = if secret {
        &["The devil holds the door.", "It does not move.", "You can stop guessing."]
    } else if game.save.secret || game.save.keys.len() >= crate::model::KEY_TOTAL as usize {
        &["You reached a door that stayed a door.", "Something else unlocked.", "Check the map."]
    } else {
        &[
            "You reached a door that stayed a door.",
            "Not every key has been found.",
            "The settings menu is a liar too.",
        ]
    };
    for (i, line) in lines.iter().enumerate() {
        text(ctx, line, x + pw * 0.5, y + 88.0 + i as f32 * 28.0, 16.0, INK, "center");
    }
    text(
        ctx,
        &format!(
            "deaths {}    skips {}    keys {}/{}",
            game.save.deaths,
            game.save.skips,
            game.save.keys.len(),
            crate::model::KEY_TOTAL
        ),
        x + pw * 0.5,
        y + ph - 78.0,
        14.0,
        DIM,
        "center",
    );
    button(ctx, hits, x + pw * 0.5 - 90.0, y + ph - 56.0, 180.0, 40.0, "world map", true, Act::Advance);
}

fn paint_play(ctx: &CanvasRenderingContext2d, game: &Game, w: f32, h: f32, hits: &mut Vec<Hit>) {
    let Scene::Play(p) = &game.scene else { return };
    let Some(sim) = &game.sim else { return };
    let shake = game.shake;
    paint_world(ctx, sim, w, h, shake, game.save.ascii);
    hud(ctx, hits, p.stage, stage_total(p), w);

    if sim.jet {
        let n = if sim.fuel_max > 0.0 { ((sim.fuel / sim.fuel_max) * 8.0).round() as i32 } else { 0 };
        text(ctx, "jet", w - 150.0, 22.0, 13.0, INK, "left");
        for i in 0..8 {
            ctx.set_fill_style_str(if i < n { INK } else { "#e0b15a" });
            ctx.fill_rect((w - 118.0 + i as f32 * 12.0) as f64, 14.0, 8.0, 16.0);
        }
    }
    if let Some(n) = sim.jumps_left {
        text(ctx, &format!("jumps {n}"), w - 24.0, 44.0, 13.0, INK, "right");
    }

    match p.phase {
        Phase::Intro => {
            let total = stage_total(p);
            let door = door_name(p);
            screen_hit(hits, w, h);
            card(
                ctx,
                w,
                h,
                &format!("{door}  {}/{total}", p.stage + 1),
                sim.name,
                "click or press a key",
            );
        }
        Phase::Dead => {
            screen_hit(hits, w, h);
            card(ctx, w, h, "DEAD", sim.cause.quip(game.save.deaths + p.deaths), "click or R to retry");
        }
        Phase::Clear => {
            screen_hit(hits, w, h);
            card(ctx, w, h, &p.announce, &p.sub, "click to continue");
        }
        Phase::Pause => {
            card(ctx, w, h, "PAUSED", "", "");
            let opts = ["resume", "retry", "map"];
            let bw = 180.0;
            let bx = (w - bw) * 0.5;
            for (i, o) in opts.iter().enumerate() {
                let by = h * 0.5 + 8.0 + i as f32 * 52.0;
                button(ctx, hits, bx, by, bw, 44.0, o, i == p.pause_sel, Act::Pause(i));
            }
        }
        Phase::Run => {}
    }

    let foot = footer(game, p, sim);
    if !foot.is_empty() {
        text(ctx, &foot, w * 0.5, h - 18.0, 14.0, INK, "center");
    }
}

fn stage_total(p: &crate::game::Play) -> usize {
    if p.secret {
        SECRET.len().max(1)
    } else {
        campaign::stage_count(p.world, p.door).max(1)
    }
}

fn door_name(p: &crate::game::Play) -> &str {
    if p.secret {
        "secret"
    } else {
        WORLDS.get(p.world as usize).and_then(|w| w.doors.get(p.door as usize)).copied().unwrap_or("door")
    }
}

fn footer(game: &Game, p: &crate::game::Play, sim: &Sim) -> String {
    if game.save.hints && p.deaths >= 3 && p.phase == Phase::Run && !sim.hint.is_empty() {
        return sim.hint.to_string();
    }
    if sim.banner_t > 0.0 && p.phase == Phase::Run {
        return sim.banner.to_string();
    }
    let mut foot = String::new();
    if sim.lie && p.phase == Phase::Run {
        foot.push_str("hold right — the door is that way");
    }
    if p.deaths >= SKIP_AFTER && !sim.no_skip && p.phase == Phase::Run {
        if !foot.is_empty() {
            foot.push_str("    ");
        }
        foot.push_str("S skips this stage");
    }
    foot
}

fn paint_world(ctx: &CanvasRenderingContext2d, sim: &Sim, w: f32, h: f32, shake: f32, plain: bool) {
    let f = view::frame(sim, w, h, shake);
    ctx.set_fill_style_str(PEACH);
    ctx.fill_rect(f.x as f64, f.y as f64, f.w as f64, f.h as f64);

    let stand = view::stand_y(sim);
    for tx in 0..sim.w as i32 {
        let mut held = false;
        for ty in 0..sim.h as i32 {
            let kind = sim.cells[ty as usize * sim.w + tx as usize].kind;
            if view::is_surface(kind, sim.flicker_on) {
                held = true;
                let above = (sim.grav >= 0.0 && (ty as f32) < stand - 0.2)
                    || (sim.grav < 0.0 && (ty as f32 + 1.0) > stand + 0.2);
                if above {
                    let (x, y, bw, bh) = cell_box(&f, tx, ty);
                    ctx.set_fill_style_str(BROWN);
                    ctx.fill_rect(x as f64, y as f64, bw as f64 + 0.5, bh as f64 + 0.5);
                }
            }
        }
        if !held {
            let x0 = f.wx(tx as f32);
            let x1 = f.wx(tx as f32 + 1.0);
            ctx.set_fill_style_str(BROWN);
            ctx.fill_rect(x0 as f64, f.y as f64, (x1 - x0) as f64 + 0.5, f.h as f64);
        }
    }

    for ty in 0..sim.h as i32 {
        for tx in 0..sim.w as i32 {
            let cell = sim.cells[ty as usize * sim.w + tx as usize];
            paint_cell(ctx, &f, tx, ty, cell.kind, cell.meta, cell.timer, stand, plain);
        }
    }

    for e in &sim.ents {
        match e {
            Ent::Saw { x, y, .. } => {
                let (bx, by, bw, bh) = world_box(&f, *x, *y, 0.85, 0.85);
                let cx = bx + bw * 0.5;
                let cy = by + bh * 0.5;
                let r = bw.min(bh) * 0.48;
                circle(ctx, cx, cy, r, INK);
                if !plain {
                    let ang = sim.time * 7.0;
                    ctx.set_stroke_style_str(PEACH);
                    ctx.set_line_width(2.0);
                    ctx.begin_path();
                    ctx.move_to(cx as f64, cy as f64);
                    ctx.line_to((cx + ang.cos() * r) as f64, (cy + ang.sin() * r) as f64);
                    ctx.stroke();
                }
            }
            Ent::Plat { x, y, w, h, .. } => {
                let (bx, by, bw, bh) = world_box(&f, *x, *y, *w, (*h).max(0.28));
                ctx.set_fill_style_str("#e8a456");
                ctx.fill_rect(bx as f64, by as f64, bw as f64, bh.max(4.0) as f64);
            }
            Ent::Wall { x, y, w, h, .. } => {
                let (bx, by, bw, bh) = world_box(&f, *x, *y, *w, *h);
                ctx.set_fill_style_str("#78242c");
                ctx.fill_rect(bx as f64, by as f64, bw as f64, bh as f64);
            }
            Ent::Chase { x, y } => {
                let (bx, by, bw, bh) = world_box(&f, *x, *y, 0.9, 0.9);
                ctx.set_fill_style_str(INK);
                ctx.fill_rect(bx as f64, by as f64, bw as f64, bh as f64);
                if !plain {
                    circle(ctx, bx + bw * 0.72, by + bh * 0.35, (bw * 0.12).max(2.0), CREAM);
                }
            }
        }
    }

    for d in &sim.doors {
        paint_door(ctx, &f, d.x, d.y, sim.peace);
    }

    for s in &sim.shots {
        let (x, y, bomb) = match s {
            Shot::Bullet { x, y, .. } => (*x, *y, false),
            Shot::Bomb { x, y, .. } => (*x, *y, true),
        };
        let (bx, by, bw, bh) = world_box(&f, x, y, if bomb { 0.55 } else { 0.35 }, if bomb { 0.55 } else { 0.35 });
        circle(ctx, bx + bw * 0.5, by + bh * 0.5, bw.min(bh) * 0.5, if bomb { RED } else { INK });
    }

    let actors: Vec<_> = sim.actors.iter().collect();
    for a in actors.iter().filter(|a| a.puppet) {
        paint_actor(ctx, sim, &f, a, plain);
    }
    for a in actors.iter().filter(|a| !a.puppet) {
        paint_actor(ctx, sim, &f, a, plain);
    }
    let edge = (f.w / sim.w as f32 * 0.08).clamp(5.0, 12.0);
    ctx.set_stroke_style_str("#3d220c");
    ctx.set_line_width(edge as f64);
    ctx.stroke_rect(f.x as f64, f.y as f64, f.w as f64, f.h as f64);
}

fn paint_cell(
    ctx: &CanvasRenderingContext2d,
    f: &Frame,
    tx: i32,
    ty: i32,
    kind: Kind,
    meta: i16,
    timer: f32,
    stand: f32,
    plain: bool,
) {
    let (x, y, w, h) = cell_box(f, tx, ty);
    if (ty as f32 - stand).abs() < 0.15 && view::is_surface(kind, true) {
        ctx.set_fill_style_str("#e08a20");
        ctx.fill_rect(x as f64, (y - 3.0) as f64, w as f64 + 0.5, 4.0);
    }
    if kind == Kind::Crumble && timer >= 0.0 {
        ctx.set_stroke_style_str(INK);
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(x as f64, (y + h * 0.5) as f64);
        ctx.line_to((x + w) as f64, (y + h * 0.3) as f64);
        ctx.stroke();
    }
    match kind {
        Kind::Spike => spikes(ctx, x, y, w, h, meta == 1),
        Kind::Coin => {
            circle(ctx, x + w * 0.5, y + h * 0.5, w.min(h) * 0.28, if plain { "#8a5a10" } else { "#f2c14e" });
        }
        Kind::Bomb => circle(ctx, x + w * 0.5, y + h * 0.5, w.min(h) * 0.26, INK),
        Kind::Key => diamond(ctx, x + w * 0.5, y + h * 0.5, w.min(h) * 0.28, INK),
        Kind::Portal => {
            let r = w.min(h) * 0.28;
            ctx.set_stroke_style_str(INK);
            ctx.set_line_width(2.0);
            ctx.begin_path();
            let _ = ctx.arc((x + w * 0.5) as f64, (y + h * 0.5) as f64, r as f64, 0.0, std::f64::consts::TAU);
            ctx.stroke();
        }
        Kind::Spring | Kind::Super => {
            ctx.set_stroke_style_str(INK);
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to((x + w * 0.25) as f64, (y + h * 0.7) as f64);
            ctx.line_to((x + w * 0.5) as f64, (y + h * 0.3) as f64);
            ctx.line_to((x + w * 0.75) as f64, (y + h * 0.7) as f64);
            ctx.stroke();
        }
        Kind::FakeDoor => paint_door(ctx, f, tx as f32, ty as f32 - 0.45, false),
        Kind::Grow => text(ctx, "+", x + w * 0.5, y + h * 0.5, h * 0.7, INK, "center"),
        Kind::Shrink => text(ctx, "–", x + w * 0.5, y + h * 0.5, h * 0.7, INK, "center"),
        Kind::TallBtn | Kind::NormBtn | Kind::Pvp => {
            let s = w.min(h) * 0.28;
            ctx.set_fill_style_str(INK);
            ctx.fill_rect((x + w * 0.5 - s) as f64, (y + h * 0.5 - s) as f64, (s * 2.0) as f64, (s * 2.0) as f64);
        }
        Kind::GunR => tri(ctx, x + w * 0.25, y + h * 0.25, x + w * 0.8, y + h * 0.5, x + w * 0.25, y + h * 0.75, INK),
        Kind::GunL => tri(ctx, x + w * 0.75, y + h * 0.25, x + w * 0.2, y + h * 0.5, x + w * 0.75, y + h * 0.75, INK),
        Kind::BombGun => circle(ctx, x + w * 0.5, y + h * 0.5, w.min(h) * 0.22, RED),
        Kind::Ice => {
            ctx.set_fill_style_str("#fff1c9");
            ctx.fill_rect(x as f64, y as f64, w as f64, (h * 0.22).max(3.0) as f64);
        }
        _ => {}
    }
}

fn paint_door(ctx: &CanvasRenderingContext2d, f: &Frame, x: f32, y: f32, peace: bool) {
    let top = f.wy(y + 0.25);
    let bot = f.wy(y + 1.45);
    let left = f.wx(x + 0.18);
    let right = f.wx(x + 0.82);
    let frame = if peace { "#6e469e" } else { "#b9bdc2" };
    let fill = if peace { "#d7b8f2" } else { "#f4f5f4" };
    ctx.set_fill_style_str(frame);
    ctx.fill_rect(left as f64, top as f64, (right - left) as f64, (bot - top) as f64);
    let inset = ((right - left) * 0.16).max(2.0);
    ctx.set_fill_style_str(fill);
    ctx.fill_rect(
        (left + inset) as f64,
        (top + inset) as f64,
        (right - left - inset * 2.0) as f64,
        (bot - top - inset * 1.4) as f64,
    );
    circle(ctx, right - inset * 1.6, top + (bot - top) * 0.55, inset * 0.45, frame);
}

fn paint_actor(ctx: &CanvasRenderingContext2d, sim: &Sim, f: &Frame, a: &crate::sim::Actor, plain: bool) {
    let x = f.wx(a.x);
    let y = f.wy(a.y);
    let w = (f.wx(a.x + a.w) - x).max(6.0);
    let h = (f.wy(a.y + a.h) - y).max(8.0);
    let cx = x + w * 0.5;
    if !a.alive {
        text(ctx, "x", cx, y + h * 0.45, h, INK, "center");
        return;
    }
    if !a.on_ground {
        if let Some(ground) = stand_line(sim, a) {
            let sy = f.wy(ground);
            ctx.set_global_alpha(0.35);
            ctx.set_fill_style_str(INK);
            ctx.fill_rect((cx - w * 0.55) as f64, (sy - 2.0) as f64, (w * 1.1) as f64, 4.0);
            ctx.set_global_alpha(1.0);
        }
    }
    let color = if a.puppet { "#3a2058" } else { INK };
    ctx.set_fill_style_str(color);
    ctx.fill_rect(x as f64, y as f64, w as f64, h as f64);
    if plain {
        return;
    }
    let face = if a.vx < -0.15 { -1.0 } else { 1.0 };
    tri(ctx, x + w * 0.22, y + 1.0, x + w * 0.08, y - h * 0.28, x + w * 0.40, y + 1.0, color);
    tri(ctx, x + w * 0.62, y + 1.0, x + w * 0.50, y + 1.0, x + w * 0.78, y - h * 0.22, color);
    let eye_x = cx + face * w * 0.16;
    let eye_y = y + h * 0.38;
    circle(ctx, eye_x, eye_y, (w * 0.14).max(2.0), CREAM);
    circle(ctx, eye_x + face * w * 0.04, eye_y, (w * 0.07).max(1.2), INK);
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
            if view::is_surface(kind, sim.flicker_on) {
                return Some(ty as f32);
            }
        }
    }
    None
}

fn hud(ctx: &CanvasRenderingContext2d, hits: &mut Vec<Hit>, stage: u8, total: usize, w: f32) {
    button(ctx, hits, 16.0, 12.0, 52.0, 36.0, "II", false, Act::HudPause);
    button(ctx, hits, 76.0, 12.0, 52.0, 36.0, "R", false, Act::HudRestart);
    let n = total.max(1) as i32;
    let pip = 22.0_f32.min(((w * 0.5) / n as f32) - 6.0).max(10.0);
    let gap = 6.0;
    let span = n as f32 * pip + (n - 1) as f32 * gap;
    let x0 = (w - span) * 0.5;
    for i in 0..n {
        let x = x0 + i as f32 * (pip + gap);
        let on = i == stage as i32;
        ctx.set_fill_style_str(if on { INK } else { "#e0b15a" });
        ctx.fill_rect(x as f64, 18.0, pip as f64, 16.0);
        if !on {
            ctx.set_stroke_style_str(INK);
            ctx.set_line_width(2.0);
            ctx.stroke_rect(x as f64, 18.0, pip as f64, 16.0);
        }
    }
}

fn card(ctx: &CanvasRenderingContext2d, w: f32, h: f32, title: &str, sub: &str, hint: &str) {
    let pw = (w * 0.7).clamp(240.0, 460.0);
    let ph = if sub.is_empty() { 72.0 } else { 120.0 };
    let x = (w - pw) * 0.5;
    let y = h * 0.5 - ph * 0.8;
    ctx.set_fill_style_str("#6e140c");
    ctx.fill_rect(x as f64, y as f64, pw as f64, ph as f64);
    text(ctx, title, w * 0.5, y + 30.0, 28.0, CREAM, "center");
    if !sub.is_empty() {
        text(ctx, sub, w * 0.5, y + 60.0, 15.0, GOLD, "center");
    }
    if !hint.is_empty() {
        text(ctx, hint, w * 0.5, y + ph - 18.0, 13.0, "#f0d7a4", "center");
    }
}

fn screen_hit(hits: &mut Vec<Hit>, w: f32, h: f32) {
    hits.push(Hit { x: 0.0, y: 0.0, w, h, act: Act::Advance });
}

fn panel(ctx: &CanvasRenderingContext2d, x: f32, y: f32, w: f32, h: f32) {
    ctx.set_fill_style_str(PEACH);
    ctx.fill_rect(x as f64, y as f64, w as f64, h as f64);
    ctx.set_stroke_style_str(INK);
    ctx.set_line_width(4.0);
    ctx.stroke_rect((x + 2.0) as f64, (y + 2.0) as f64, (w - 4.0) as f64, (h - 4.0) as f64);
}

fn button(
    ctx: &CanvasRenderingContext2d,
    hits: &mut Vec<Hit>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    hot: bool,
    act: Act,
) {
    ctx.set_fill_style_str(if hot { GOLD } else { "#3d220c" });
    ctx.fill_rect(x as f64, y as f64, w as f64, h as f64);
    text(ctx, label, x + w * 0.5, y + h * 0.5, (h * 0.42).clamp(15.0, 28.0), if hot { INK } else { CREAM }, "center");
    hits.push(Hit { x, y, w, h, act });
}

fn text(ctx: &CanvasRenderingContext2d, s: &str, x: f32, y: f32, size: f32, color: &str, align: &str) {
    if s.is_empty() {
        return;
    }
    ctx.set_fill_style_str(color);
    ctx.set_font(&format!("700 {size:.0}px ui-sans-serif, system-ui, sans-serif"));
    ctx.set_text_align(align);
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(s, x as f64, y as f64);
}

fn circle(ctx: &CanvasRenderingContext2d, x: f32, y: f32, r: f32, color: &str) {
    if r <= 0.0 {
        return;
    }
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    let _ = ctx.arc(x as f64, y as f64, r as f64, 0.0, std::f64::consts::TAU);
    ctx.fill();
}

fn diamond(ctx: &CanvasRenderingContext2d, x: f32, y: f32, r: f32, color: &str) {
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    ctx.move_to(x as f64, (y - r) as f64);
    ctx.line_to((x + r) as f64, y as f64);
    ctx.line_to(x as f64, (y + r) as f64);
    ctx.line_to((x - r) as f64, y as f64);
    ctx.close_path();
    ctx.fill();
}

fn tri(ctx: &CanvasRenderingContext2d, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, color: &str) {
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    ctx.move_to(x0 as f64, y0 as f64);
    ctx.line_to(x1 as f64, y1 as f64);
    ctx.line_to(x2 as f64, y2 as f64);
    ctx.close_path();
    ctx.fill();
}

fn spikes(ctx: &CanvasRenderingContext2d, x: f32, y: f32, w: f32, h: f32, down: bool) {
    ctx.set_fill_style_str(INK);
    let n = 3;
    for i in 0..n {
        let x0 = x + w * (i as f32) / n as f32;
        let x1 = x + w * ((i + 1) as f32) / n as f32;
        let mid = (x0 + x1) * 0.5;
        ctx.begin_path();
        if down {
            ctx.move_to(x0 as f64, y as f64);
            ctx.line_to(x1 as f64, y as f64);
            ctx.line_to(mid as f64, (y + h) as f64);
        } else {
            ctx.move_to(x0 as f64, (y + h) as f64);
            ctx.line_to(x1 as f64, (y + h) as f64);
            ctx.line_to(mid as f64, y as f64);
        }
        ctx.close_path();
        ctx.fill();
    }
}

fn cell_box(f: &Frame, tx: i32, ty: i32) -> (f32, f32, f32, f32) {
    world_box(f, tx as f32, ty as f32, 1.0, 1.0)
}

fn world_box(f: &Frame, x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
    let x0 = f.wx(x);
    let y0 = f.wy(y);
    let x1 = f.wx(x + w);
    let y1 = f.wy(y + h);
    (x0, y0, x1 - x0, y1 - y0)
}

fn onoff(v: bool) -> &'static str {
    if v { "on" } else { "off" }
}

pub fn hit_at(hits: &[Hit], x: f32, y: f32) -> Option<Act> {
    hits.iter().rev().find(|h| x >= h.x && y >= h.y && x < h.x + h.w && y < h.y + h.h).map(|h| h.act)
}

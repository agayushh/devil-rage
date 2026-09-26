//! Camera for the browser view. A short stretch of the room, big enough to play.

use crate::model::Kind;
use crate::sim::Sim;

pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub mw: f32,
    pub top: f32,
    pub bot: f32,
}

impl Frame {
    pub fn wx(&self, world_x: f32) -> f32 {
        self.x + world_x / self.mw * self.w
    }

    pub fn wy(&self, world_y: f32) -> f32 {
        let t = (world_y - self.top) / (self.bot - self.top).max(0.01);
        self.y + t * self.h
    }
}

pub fn is_surface(kind: Kind, flicker_on: bool) -> bool {
    matches!(
        kind,
        Kind::Solid | Kind::Fake | Kind::Crumble | Kind::Ice | Kind::OneWay | Kind::Spring | Kind::Super
    ) || (kind == Kind::Flicker && flicker_on)
}

/// World y the player stands on. The peach room ends on this line.
pub fn stand_y(sim: &Sim) -> f32 {
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

pub fn frame(sim: &Sim, vw: f32, vh: f32, shake: f32) -> Frame {
    if sim.w == 0 || sim.h == 0 || vw < 32.0 || vh < 32.0 {
        return Frame { x: 0.0, y: 0.0, w: 1.0, h: 1.0, mw: 1.0, top: 0.0, bot: 1.0 };
    }
    let stand = stand_y(sim);
    let top = stand - 2.45;
    let bot = stand + 0.08;
    let span = (bot - top).max(0.01);
    let jx = if shake > 0.0 { (sim.time * 70.0).sin() * shake * 28.0 } else { 0.0 };
    let jy = if shake > 0.0 { (sim.time * 54.0).cos() * shake * 12.0 } else { 0.0 };
    // About a dozen tiles. The lie stays offscreen until you are committed.
    let tiles = (sim.w as f32).min(12.0).max(8.0);
    let scale = ((vw * 0.98) / tiles).min((vh * 0.74) / span);
    let w = scale * sim.w as f32;
    let h = scale * span;
    let focus = sim.actors.first().map(|a| a.x + a.w * 0.5).unwrap_or(sim.w as f32 * 0.5);
    let mut x = if w <= vw {
        (vw - w) * 0.5
    } else {
        vw * 0.30 - (focus / sim.w as f32) * w
    };
    if w > vw {
        x = x.clamp(vw - w, 0.0);
    }
    x += jx;
    let mut y = (vh - h) * 0.50 + jy;
    if y < 52.0 {
        y = 52.0;
    }
    if y + h > vh - 12.0 {
        y = (vh - 12.0 - h).max(40.0);
    }
    Frame { x, y, w, h, mw: sim.w as f32, top, bot }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LevelDef;
    use crate::sim::Sim;

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

    #[test]
    fn the_whole_level_fits_the_band() {
        let sim = Sim::boot(&stage(), &[], false).unwrap();
        let f = frame(&sim, 1280.0, 720.0, 0.0);
        let left = f.wx(0.0);
        let right = f.wx(sim.w as f32);
        assert!((right - left - f.w).abs() < 0.5, "level width should fill the band");
        assert!(f.wy(f.bot) > f.wy(f.top), "down the world is down the screen");
        let floor = f.wy(stand_y(&sim));
        assert!(floor > f.y && floor <= f.y + f.h + 0.5, "the floor sits on the band");
    }
}

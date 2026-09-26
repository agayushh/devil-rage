//! Browser host: keyboard, touch, sound, and the animation frame.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    AddEventListenerOptions, AudioContext, CanvasRenderingContext2d, Document, Element, EventTarget,
    HtmlCanvasElement, KeyboardEvent, OscillatorType, PointerEvent, Window,
};

use crate::game::{Game, Key};
use crate::paint::{self, Act};

struct State {
    game: Game,
    hits: Vec<paint::Hit>,
    last_t: f64,
    last_status: String,
    audio: Option<AudioContext>,
    reduce: bool,
    shown: bool,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    let window = window();
    if document().get_element_by_id("game").is_some() {
        boot();
        return;
    }
    let closure = Closure::once(boot);
    let _ = window
        .add_event_listener_with_callback("DOMContentLoaded", closure.as_ref().unchecked_ref());
    closure.forget();
}

fn boot() {
    let window = window();
    let document = document();
    let Some(node) = document.get_element_by_id("game") else {
        return;
    };
    let canvas: HtmlCanvasElement = node.dyn_into().expect("game is a canvas");
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
        .expect("2d context");
    let _ = canvas.set_attribute("tabindex", "0");
    let el: &web_sys::HtmlElement = canvas.unchecked_ref();
    let _ = el.focus();
    let reduce = window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|list| list.matches());

    STATE.with(|slot| {
        *slot.borrow_mut() = Some(State {
            game: Game::boot(),
            hits: Vec::new(),
            last_t: 0.0,
            last_status: String::new(),
            audio: None,
            reduce,
            shown: false,
            canvas,
            ctx,
        });
    });

    bind_keys(&window);
    bind_pad(&document);
    bind_canvas();

    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let win = window.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |t: f64| {
        frame(t);
        let _ = win.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }) as Box<dyn FnMut(f64)>));
    let _ = window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
}

fn frame(t: f64) {
    STATE.with(|slot| {
        let mut borrowed = slot.borrow_mut();
        let Some(state) = borrowed.as_mut() else { return };
        let dt = if state.last_t == 0.0 { 0.0 } else { ((t - state.last_t) / 1000.0) as f32 };
        state.last_t = t;
        state.game.tick(dt.clamp(0.0, 0.05));
        if state.reduce {
            state.game.shake = 0.0;
        }
        if state.game.chime {
            state.game.chime = false;
            ring(&mut state.audio);
        }
        let css_w = state.canvas.client_width() as f32;
        let css_h = state.canvas.client_height() as f32;
        if css_w < 8.0 || css_h < 8.0 {
            return;
        }
        let dpr = window().device_pixel_ratio().max(1.0);
        let bw = (css_w as f64 * dpr) as u32;
        let bh = (css_h as f64 * dpr) as u32;
        if state.canvas.width() != bw {
            state.canvas.set_width(bw);
        }
        if state.canvas.height() != bh {
            state.canvas.set_height(bh);
        }
        let _ = state.ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
        paint::paint(&state.ctx, &state.game, css_w, css_h, &mut state.hits);
        let status = state.game.status_line();
        if status != state.last_status {
            state.last_status = status.clone();
            if let Some(live) = document().get_element_by_id("live") {
                live.set_text_content(Some(&status));
            }
        }
        if !state.shown {
            state.shown = true;
            if let Some(boot) = document().get_element_by_id("boot") {
                let _ = boot.set_attribute("hidden", "");
            }
        }
    });
}

fn bind_keys(window: &Window) {
    let down = Closure::wrap(Box::new(|ev: KeyboardEvent| on_board(ev, true)) as Box<dyn FnMut(KeyboardEvent)>);
    let up = Closure::wrap(Box::new(|ev: KeyboardEvent| on_board(ev, false)) as Box<dyn FnMut(KeyboardEvent)>);
    let blur = Closure::wrap(Box::new(|| {
        STATE.with(|slot| {
            if let Some(state) = slot.borrow_mut().as_mut() {
                state.game.release_all();
            }
        });
    }) as Box<dyn FnMut()>);
    let opts = AddEventListenerOptions::new();
    opts.set_passive(false);
    opts.set_capture(true);
    let target: &EventTarget = window;
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        "keydown",
        down.as_ref().unchecked_ref(),
        &opts,
    );
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        "keyup",
        up.as_ref().unchecked_ref(),
        &opts,
    );
    let _ = target.add_event_listener_with_callback("blur", blur.as_ref().unchecked_ref());
    down.forget();
    up.forget();
    blur.forget();
}

fn on_board(ev: KeyboardEvent, down: bool) {
    if interactive_target(&ev) {
        return;
    }
    let code = ev.code();
    let Some(key) = map_code(&code) else { return };
    ev.prevent_default();
    STATE.with(|slot| {
        if let Some(state) = slot.borrow_mut().as_mut() {
            state.game.on_key(key, down, ev.repeat());
        }
    });
}

fn interactive_target(ev: &KeyboardEvent) -> bool {
    let Some(target) = ev.target() else { return false };
    let Ok(el) = target.dyn_into::<Element>() else { return false };
    let tag = el.tag_name();
    if matches!(tag.as_str(), "A" | "INPUT" | "TEXTAREA" | "SELECT") {
        return true;
    }
    if tag == "BUTTON" && !el.has_attribute("data-k") {
        return true;
    }
    false
}

fn bind_canvas() {
    let Some(canvas) = ({
        STATE.with(|slot| slot.borrow().as_ref().map(|s| s.canvas.clone()))
    }) else {
        return;
    };
    let focus_target = canvas.clone();
    let pointer = Closure::wrap(Box::new(move |ev: PointerEvent| {
        let x = ev.offset_x() as f32;
        let y = ev.offset_y() as f32;
        STATE.with(|slot| {
            let mut borrowed = slot.borrow_mut();
            let Some(state) = borrowed.as_mut() else { return };
            if let Some(act) = paint::hit_at(&state.hits, x, y) {
                apply(&mut state.game, act);
            }
        });
        let el: &web_sys::HtmlElement = focus_target.unchecked_ref();
        let _ = el.focus();
    }) as Box<dyn FnMut(PointerEvent)>);
    let target: &EventTarget = canvas.unchecked_ref();
    let _ = target.add_event_listener_with_callback("pointerdown", pointer.as_ref().unchecked_ref());
    pointer.forget();
}

fn bind_pad(document: &Document) {
    let Some(pad) = document.get_element_by_id("pad") else { return };
    let down = Closure::wrap(Box::new(|ev: PointerEvent| pad_pointer(ev, true)) as Box<dyn FnMut(PointerEvent)>);
    let up = Closure::wrap(Box::new(|ev: PointerEvent| pad_pointer(ev, false)) as Box<dyn FnMut(PointerEvent)>);
    let target: &EventTarget = pad.unchecked_ref();
    let opts = AddEventListenerOptions::new();
    opts.set_passive(false);
    let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
        "pointerdown",
        down.as_ref().unchecked_ref(),
        &opts,
    );
    let _ = target.add_event_listener_with_callback("pointerup", up.as_ref().unchecked_ref());
    let _ = target.add_event_listener_with_callback("pointercancel", up.as_ref().unchecked_ref());
    let _ = target.add_event_listener_with_callback("lostpointercapture", up.as_ref().unchecked_ref());
    down.forget();
    up.forget();
}

fn pad_pointer(ev: PointerEvent, down: bool) {
    let Some((el, key)) = pad_key(&ev) else { return };
    ev.prevent_default();
    if down {
        let _ = el.set_pointer_capture(ev.pointer_id());
    }
    STATE.with(|slot| {
        if let Some(state) = slot.borrow_mut().as_mut() {
            state.game.on_key(key, down, false);
        }
    });
}

fn pad_key(ev: &PointerEvent) -> Option<(Element, Key)> {
    let target = ev.target()?;
    let el = target.dyn_into::<Element>().ok()?;
    let btn = if el.has_attribute("data-k") {
        el
    } else {
        el.closest("[data-k]").ok().flatten()?
    };
    let name = btn.get_attribute("data-k")?;
    let key = parse_pad(&name)?;
    Some((btn, key))
}

fn apply(game: &mut Game, act: Act) {
    match act {
        Act::Title(i) => game.pick_title(i),
        Act::Settings(i) => game.pick_settings(i),
        Act::Door { world, door } => game.pick_door(world, door),
        Act::Secret => game.pick_secret(),
        Act::Pause(i) => game.pick_pause(i),
        Act::HudPause => game.tap(Key::Esc),
        Act::HudRestart => game.tap(Key::R),
        Act::Advance => game.tap(Key::Enter),
    }
}

fn ring(audio: &mut Option<AudioContext>) {
    let ctx = match audio {
        Some(ctx) => ctx,
        None => {
            let Ok(ctx) = AudioContext::new() else { return };
            *audio = Some(ctx);
            audio.as_ref().unwrap()
        }
    };
    let _ = ctx.resume();
    let Ok(osc) = ctx.create_oscillator() else { return };
    let Ok(gain) = ctx.create_gain() else { return };
    osc.set_type(OscillatorType::Square);
    osc.frequency().set_value(196.0);
    gain.gain().set_value(0.045);
    if osc.connect_with_audio_node(&gain).is_err() {
        return;
    }
    if gain.connect_with_audio_node(&ctx.destination()).is_err() {
        return;
    }
    let now = ctx.current_time();
    if osc.start().is_err() {
        return;
    }
    let _ = osc.stop_with_when(now + 0.09);
}

fn map_code(code: &str) -> Option<Key> {
    Some(match code {
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "KeyA" => Key::A,
        "KeyD" => Key::D,
        "KeyW" => Key::W,
        "KeyS" => Key::S,
        "Space" => Key::Space,
        "Enter" => Key::Enter,
        "Escape" => Key::Esc,
        "KeyR" => Key::R,
        "KeyP" => Key::P,
        "KeyQ" => Key::Q,
        "KeyK" => Key::K,
        "KeyJ" => Key::J,
        _ => return None,
    })
}

fn parse_pad(name: &str) -> Option<Key> {
    Some(match name {
        "Left" => Key::Left,
        "Right" => Key::Right,
        "Up" => Key::Up,
        "Down" => Key::Down,
        "Space" => Key::Space,
        "Esc" => Key::Esc,
        "R" => Key::R,
        "Enter" => Key::Enter,
        _ => return None,
    })
}

fn window() -> Window {
    web_sys::window().expect("window")
}

fn document() -> Document {
    window().document().expect("document")
}

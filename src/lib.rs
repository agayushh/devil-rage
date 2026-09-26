mod campaign;
pub mod game;

pub use campaign::check_all;
mod level;
mod model;
mod sim;
#[cfg(any(test, target_arch = "wasm32"))]
mod view;

#[cfg(target_arch = "wasm32")]
mod paint;
#[cfg(target_arch = "wasm32")]
mod web;

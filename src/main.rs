mod app;
mod campaign;
mod draw;
mod level;
mod model;
mod sim;

fn main() {
    if let Err(e) = app::run() {
        eprintln!("level-devil: {e}");
        std::process::exit(1);
    }
}

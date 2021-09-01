mod api;
mod gui;
mod model;
mod timeline;

pub const INSTANCE: &str = "bunne.garden";

fn main() {
    env_logger::init();
    gui::thread();
}

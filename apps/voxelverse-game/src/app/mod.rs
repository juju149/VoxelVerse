pub(crate) mod content_bootstrap;
mod golden_scene;
mod inventory_events;
mod runtime_loop;

pub fn run() {
    runtime_loop::run();
}

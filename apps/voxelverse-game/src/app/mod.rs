pub(crate) mod content_bootstrap;
mod inventory_events;
mod runtime_loop;

pub fn run() {
    runtime_loop::run();
}

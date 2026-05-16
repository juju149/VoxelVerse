use crate::app::game_app::GameApp;
use vv_diagnostics::SystemDiagnostics;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, Window, WindowBuilder};

pub fn run() {
    SystemDiagnostics::print_startup_info();

    let event_loop = EventLoop::new().unwrap();
    let window = create_window(&event_loop);
    let mut app = GameApp::new(&window);

    event_loop
        .run(move |event, target| match event {
            Event::DeviceEvent { event, .. } => {
                app.handle_device_event(event);
            }
            Event::WindowEvent { event, window_id } if window_id == app.window_id() => {
                app.handle_window_event(event, target);
            }
            Event::AboutToWait => {
                app.tick();
            }
            _ => {}
        })
        .unwrap();
}

fn create_window(event_loop: &EventLoop<()>) -> Window {
    WindowBuilder::new()
        .with_title("voxelverse")
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(event_loop)
        .unwrap()
}

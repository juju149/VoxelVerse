use crate::app::inventory_events::handle_inventory_window_event;
use crate::app::runtime_state::GameRuntime;
use vv_render::Renderer;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Handle console-specific keyboard input.
///
/// Returns `true` if the event was consumed by the console.
pub(super) fn handle_console_input(event: &WindowEvent, runtime: &mut GameRuntime) -> bool {
    let console_open = runtime.console().is_open;
    if console_open {
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = event
        {
            if key_event.state == ElementState::Pressed {
                match key_event.physical_key {
                    PhysicalKey::Code(KeyCode::Backquote) => {
                        runtime.console_mut().toggle();
                    }
                    PhysicalKey::Code(KeyCode::Enter) => {
                        runtime.submit_console_command();
                    }
                    PhysicalKey::Code(KeyCode::Backspace) => {
                        runtime.console_mut().handle_backspace();
                    }
                    _ => {
                        if let Some(txt) = &key_event.text {
                            let chars: Vec<char> = txt.chars().collect();
                            for c in chars {
                                runtime.console_mut().handle_char(c);
                            }
                        }
                    }
                }
            }
            return true;
        }
    }

    if let WindowEvent::KeyboardInput {
        event: key_event, ..
    } = event
    {
        if key_event.state == ElementState::Pressed {
            if let PhysicalKey::Code(KeyCode::Backquote) = key_event.physical_key {
                runtime.console_mut().toggle();
                return true;
            }
        }
    }

    false
}

/// Route inventory window events when the inventory screen is open.
pub(super) fn handle_inventory_input(
    event: WindowEvent,
    target: &EventLoopWindowTarget<()>,
    runtime: &mut GameRuntime,
    renderer: &mut Renderer<'_>,
) {
    let mut ctx = runtime.as_inventory_context();
    handle_inventory_window_event(event, target, renderer, &mut ctx);
}

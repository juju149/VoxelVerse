use crate::app::cursor::{grab_cursor, release_cursor};
use crate::app::debug_input::handle_dev_key;
use crate::app::game_app::GameApp;
use crate::app::input_intent::{intent_for_hotbar_key, intent_for_mouse_button, intent_for_scroll};
use crate::app::player_input_router::dispatch_intent;
use crate::app::ui_input_router::{handle_console_input, handle_inventory_input};
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn route_device_event(app: &mut GameApp<'_>, event: DeviceEvent) {
    if let DeviceEvent::MouseMotion { delta } = event {
        if !app.runtime.ui_captures_input() {
            app.runtime.controller_mut().process_mouse_motion(delta);
        }
    }
}

pub(super) fn route_window_event(
    app: &mut GameApp<'_>,
    event: WindowEvent,
    target: &EventLoopWindowTarget<()>,
) {
    // Console input always gets first priority.
    if handle_console_input(&event, &mut app.runtime) {
        if let WindowEvent::RedrawRequested = event {
            render_current_frame(app);
        }
        return;
    }

    // Console open but did not consume the event — only system events are valid.
    if app.runtime.console().is_open {
        route_system_event(app, event, target);
        return;
    }

    // Inventory open — route all events to inventory handler.
    if app.runtime.inventory_ui().is_open {
        handle_inventory_input(event, target, &mut app.runtime, &mut app.renderer);
        return;
    }

    // Normal gameplay event routing.
    app.runtime.process_controller_event(&event);
    route_game_event(app, event, target);
}

// ─── Gameplay event routing ───────────────────────────────────────────────────

fn route_game_event(app: &mut GameApp<'_>, event: WindowEvent, target: &EventLoopWindowTarget<()>) {
    match event {
        WindowEvent::CloseRequested => target.exit(),

        WindowEvent::Resized(size) => {
            app.renderer.resize(size.width, size.height);
        }

        WindowEvent::Focused(true) if app.runtime.first_person() => {
            grab_cursor(app.renderer.window);
        }
        WindowEvent::Focused(true) => {}
        WindowEvent::Focused(false) => release_cursor(app.renderer.window),

        WindowEvent::MouseInput { state, button, .. } => {
            if let Some(intent) = intent_for_mouse_button(button, state) {
                dispatch_intent(intent, &mut app.runtime, &mut app.renderer, &mut app.audio);
            }
        }

        WindowEvent::MouseWheel { delta, .. } if app.runtime.first_person() => {
            if let Some(intent) = intent_for_scroll(delta) {
                dispatch_intent(intent, &mut app.runtime, &mut app.renderer, &mut app.audio);
            }
        }

        WindowEvent::KeyboardInput { event: key, .. } if key.state == ElementState::Pressed => {
            route_pressed_key(app, key.physical_key);
        }

        WindowEvent::RedrawRequested => render_current_frame(app),

        _ => {}
    }
}

fn route_pressed_key(app: &mut GameApp<'_>, key: PhysicalKey) {
    // Inventory toggle.
    if key == PhysicalKey::Code(KeyCode::KeyE) {
        dispatch_intent(
            crate::app::input_intent::InputIntent::ToggleInventory,
            &mut app.runtime,
            &mut app.renderer,
            &mut app.audio,
        );
        if app.runtime.inventory_ui().is_open {
            release_cursor(app.renderer.window);
        } else if app.runtime.first_person() {
            grab_cursor(app.renderer.window);
        }
        return;
    }

    // Hotbar digit keys.
    if let Some(intent) = intent_for_hotbar_key(key) {
        dispatch_intent(intent, &mut app.runtime, &mut app.renderer, &mut app.audio);
        return;
    }

    // Dev / debug keys (planet resize, quality toggles).
    let (planet, player) = app.runtime.planet_and_player_mut();
    handle_dev_key(key, &mut app.renderer, planet, player);
}

// ─── System event routing (active while console or other overlay is open) ─────

fn route_system_event(
    app: &mut GameApp<'_>,
    event: WindowEvent,
    target: &EventLoopWindowTarget<()>,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => app.renderer.resize(size.width, size.height),
        WindowEvent::RedrawRequested => render_current_frame(app),
        _ => {}
    }
}

// ─── Render helper ────────────────────────────────────────────────────────────

fn render_current_frame(app: &mut GameApp<'_>) {
    app.renderer.render(
        app.runtime.controller(),
        app.runtime.player(),
        app.runtime.planet(),
        app.runtime.hotbar(),
        app.runtime.inventory(),
        app.runtime.inventory_ui(),
        app.runtime.recipes(),
        app.runtime.console(),
    );
}

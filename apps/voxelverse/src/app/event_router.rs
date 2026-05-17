use crate::app::cursor::{grab_cursor, release_cursor};
use crate::app::debug_input::decode_dev_key;
use crate::app::frame_commands::{apply_action_result, apply_frame_commands};
use crate::app::game_app::GameApp;
use crate::app::input_intent::{intent_for_hotbar_key, intent_for_mouse_button, intent_for_scroll};
use crate::app::player_input_router::dispatch_intent;
use crate::app::ui_input_router::{handle_console_input, handle_inventory_input};
use vv_render::{RenderCamera, RenderConsoleSnapshot, RenderDebugFlags, RenderFrameSnapshot};
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn route_device_event(app: &mut GameApp<'_>, event: DeviceEvent) {
    if let DeviceEvent::MouseMotion { delta } = event {
        if !app.runtime.ui_captures_input() {
            app.input_accum.push_mouse_motion(delta);
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
    app.input_accum.push_window_event(&event);
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
                let w = app.renderer.config.width as f32;
                let h = app.renderer.config.height as f32;
                let result = dispatch_intent(intent, &mut app.runtime, w, h);
                apply_action_result(result, &mut app.renderer, &mut app.audio, &mut app.runtime);
            }
        }

        WindowEvent::MouseWheel { delta, .. } if app.runtime.first_person() => {
            if let Some(intent) = intent_for_scroll(delta) {
                let w = app.renderer.config.width as f32;
                let h = app.renderer.config.height as f32;
                let result = dispatch_intent(intent, &mut app.runtime, w, h);
                apply_action_result(result, &mut app.renderer, &mut app.audio, &mut app.runtime);
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
        let w = app.renderer.config.width as f32;
        let h = app.renderer.config.height as f32;
        let result = dispatch_intent(
            crate::app::input_intent::InputIntent::ToggleInventory,
            &mut app.runtime,
            w,
            h,
        );
        apply_action_result(result, &mut app.renderer, &mut app.audio, &mut app.runtime);
        return;
    }

    // Hotbar digit keys.
    if let Some(intent) = intent_for_hotbar_key(key) {
        let w = app.renderer.config.width as f32;
        let h = app.renderer.config.height as f32;
        let result = dispatch_intent(intent, &mut app.runtime, w, h);
        apply_action_result(result, &mut app.renderer, &mut app.audio, &mut app.runtime);
        return;
    }

    // Dev / debug keys — only active when dev_mode is enabled.
    if app.runtime.dev_mode() {
        let cmds = decode_dev_key(key);
        if !cmds.is_empty() {
            apply_frame_commands(cmds, &mut app.renderer, &mut app.runtime);
        }
    }
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
    let w = app.renderer.config.width as f32;
    let h = app.renderer.config.height as f32;
    let dev = app.runtime.dev_state();
    let frame = RenderFrameSnapshot {
        camera: RenderCamera {
            view_proj: app
                .runtime
                .controller()
                .get_matrix(app.runtime.player(), w, h),
            camera_pos: app
                .runtime
                .controller()
                .get_camera_pos(app.runtime.player()),
            player_pos: app.runtime.player().position,
            model_matrix: app.runtime.player().get_model_matrix(),
            is_first_person: app.runtime.first_person(),
            cursor_id: app.runtime.cursor_id(),
        },
        planet: app.runtime.planet(),
        hotbar: app.runtime.hotbar(),
        inventory: app.runtime.inventory(),
        inventory_ui: app.runtime.inventory_ui(),
        recipes: app.runtime.recipes(),
        console: RenderConsoleSnapshot {
            height_fraction: app.runtime.console().height_fraction,
            history: &app.runtime.console().history,
            input_buffer: &app.runtime.console().input_buffer,
        },
        debug: RenderDebugFlags {
            show_collisions: dev.show_collisions,
            freeze_culling: dev.freeze_culling,
            is_wireframe: dev.is_wireframe,
            debug_mode: app.runtime.dev_mode(),
        },
    };
    app.renderer.render(&frame);
}

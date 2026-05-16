use crate::app::cursor::{grab_cursor, release_cursor};
use crate::app::game_app::GameApp;
use crate::app::gameplay_actions::{place_block, PlaceBlockContext};
use crate::app::inventory_events::handle_inventory_window_event;
use vv_gameplay::{Console, Hotbar, PlanetResize, PlanetResizeIntent, Player};
use vv_render::Renderer;
use vv_world::PlanetData;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn route_device_event(app: &mut GameApp<'_>, event: DeviceEvent) {
    if let DeviceEvent::MouseMotion { delta } = event {
        if !app.runtime.ui.console.is_open && !app.runtime.ui.inventory.is_open {
            app.runtime.gameplay.controller.process_mouse_motion(delta);
        }
    }
}

pub(super) fn route_window_event(
    app: &mut GameApp<'_>,
    event: WindowEvent,
    target: &EventLoopWindowTarget<()>,
) {
    if handle_console_event(
        &event,
        &mut app.runtime.ui.console,
        &mut app.runtime.gameplay.player,
    ) {
        return;
    }

    if app.runtime.ui.console.is_open {
        handle_console_window_event(app, event, target);
        return;
    }

    if app.runtime.ui.inventory.is_open {
        handle_inventory_window_event(
            event,
            target,
            &mut app.renderer,
            &mut app.runtime.gameplay.controller,
            &app.runtime.gameplay.player,
            &app.runtime.planet,
            &mut app.runtime.gameplay.hotbar,
            &mut app.runtime.gameplay.inventory,
            &mut app.runtime.ui.inventory,
            &app.runtime.content.recipes,
            &app.runtime.content.tags,
            &mut app.runtime.ui.shift_held,
            &app.runtime.ui.console,
        );
        return;
    }

    app.runtime
        .gameplay
        .controller
        .process_events(&event, &app.runtime.gameplay.player);
    handle_game_window_event(app, event, target);
}

fn handle_console_event(event: &WindowEvent, console: &mut Console, player: &mut Player) -> bool {
    if console.is_open {
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = event
        {
            if key_event.state == ElementState::Pressed {
                match key_event.physical_key {
                    PhysicalKey::Code(KeyCode::Backquote) => console.toggle(),
                    PhysicalKey::Code(KeyCode::Enter) => console.submit(player),
                    PhysicalKey::Code(KeyCode::Backspace) => console.handle_backspace(),
                    _ => {
                        if let Some(txt) = &key_event.text {
                            for c in txt.chars() {
                                console.handle_char(c);
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
                console.toggle();
                return true;
            }
        }
    }

    false
}

fn handle_console_window_event(
    app: &mut GameApp<'_>,
    event: WindowEvent,
    target: &EventLoopWindowTarget<()>,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => app.renderer.resize(size.width, size.height),
        WindowEvent::RedrawRequested => app.renderer.render(
            &app.runtime.gameplay.controller,
            &app.runtime.gameplay.player,
            &app.runtime.planet,
            &app.runtime.gameplay.hotbar,
            &app.runtime.gameplay.inventory,
            &app.runtime.ui.inventory,
            &app.runtime.content.recipes,
            &app.runtime.ui.console,
        ),
        _ => {}
    }
}

fn handle_game_window_event(
    app: &mut GameApp<'_>,
    event: WindowEvent,
    target: &EventLoopWindowTarget<()>,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => app.renderer.resize(size.width, size.height),
        WindowEvent::Focused(true)
            if app.runtime.gameplay.controller.first_person && !app.runtime.ui.console.is_open =>
        {
            grab_cursor(app.renderer.window);
        }
        WindowEvent::Focused(true) => {}
        WindowEvent::Focused(false) => release_cursor(app.renderer.window),
        WindowEvent::MouseInput { state, button, .. } => match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => {
                app.runtime.gameplay.mining_button_held = true;
            }
            (MouseButton::Left, ElementState::Released) => {
                app.runtime.gameplay.mining_button_held = false;
            }
            (MouseButton::Right, ElementState::Pressed) => place_block(PlaceBlockContext {
                renderer: &mut app.renderer,
                audio: &mut app.audio,
                controller: &mut app.runtime.gameplay.controller,
                player: &app.runtime.gameplay.player,
                planet: &mut app.runtime.planet,
                hotbar: &mut app.runtime.gameplay.hotbar,
            }),
            _ => {}
        },
        WindowEvent::MouseWheel { delta, .. } if app.runtime.gameplay.controller.first_person => {
            handle_hotbar_scroll(delta, &mut app.runtime.gameplay.hotbar);
        }
        WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
            if let PhysicalKey::Code(KeyCode::KeyE) = event.physical_key {
                app.runtime.ui.inventory.toggle();
                if app.runtime.ui.inventory.is_open {
                    release_cursor(app.renderer.window);
                }
                app.renderer.window.request_redraw();
                return;
            }
            handle_pressed_key(
                event.physical_key,
                &mut app.renderer,
                &mut app.runtime.gameplay.player,
                &mut app.runtime.planet,
                &mut app.runtime.gameplay.hotbar,
            );
        }
        WindowEvent::RedrawRequested => app.renderer.render(
            &app.runtime.gameplay.controller,
            &app.runtime.gameplay.player,
            &app.runtime.planet,
            &app.runtime.gameplay.hotbar,
            &app.runtime.gameplay.inventory,
            &app.runtime.ui.inventory,
            &app.runtime.content.recipes,
            &app.runtime.ui.console,
        ),
        _ => {}
    }
}

fn handle_pressed_key(
    key: PhysicalKey,
    renderer: &mut Renderer<'_>,
    player: &mut Player,
    planet: &mut PlanetData,
    hotbar: &mut Hotbar,
) {
    if let Some(index) = hotbar_index_for_key(key) {
        hotbar.select(index);
        renderer.window.request_redraw();
        return;
    }

    let resize = match key {
        PhysicalKey::Code(KeyCode::BracketRight) => Some(PlanetResizeIntent::Grow),
        PhysicalKey::Code(KeyCode::BracketLeft) => Some(PlanetResizeIntent::Shrink),
        _ => None,
    };

    if let Some(intent) = resize {
        PlanetResize::apply(intent, planet, player);
        renderer.force_reload_all(planet, player.position);
        renderer.log_memory(planet);
        renderer.window.request_redraw();
        return;
    }

    match key {
        PhysicalKey::Code(KeyCode::F3) | PhysicalKey::Code(KeyCode::Fn) => {
            renderer.quality.color_only_mode = !renderer.quality.color_only_mode;
            println!(
                "[quality] color-only mode = {} (textures {})",
                renderer.quality.color_only_mode,
                if renderer.quality.color_only_mode {
                    "OFF"
                } else {
                    "ON"
                }
            );
        }
        PhysicalKey::Code(KeyCode::F2) => {
            renderer.toggle_engine_debug_page();
            renderer.window.request_redraw();
        }
        PhysicalKey::Code(KeyCode::F5) => {
            renderer.quality.triplanar_grain = !renderer.quality.triplanar_grain;
            println!(
                "[quality] triplanar grain = {}",
                renderer.quality.triplanar_grain
            );
        }
        PhysicalKey::Code(KeyCode::F6) => {
            use vv_render::PcfQuality;
            renderer.quality.pcf = match renderer.quality.pcf {
                PcfQuality::Low => PcfQuality::Medium,
                PcfQuality::Medium => PcfQuality::High,
                PcfQuality::High => PcfQuality::Low,
            };
            println!("[quality] PCF = {:?}", renderer.quality.pcf);
        }
        _ => {}
    }
}

fn hotbar_index_for_key(key: PhysicalKey) -> Option<usize> {
    match key {
        PhysicalKey::Code(KeyCode::Digit1) => Some(0),
        PhysicalKey::Code(KeyCode::Digit2) => Some(1),
        PhysicalKey::Code(KeyCode::Digit3) => Some(2),
        PhysicalKey::Code(KeyCode::Digit4) => Some(3),
        PhysicalKey::Code(KeyCode::Digit5) => Some(4),
        PhysicalKey::Code(KeyCode::Digit6) => Some(5),
        PhysicalKey::Code(KeyCode::Digit7) => Some(6),
        PhysicalKey::Code(KeyCode::Digit8) => Some(7),
        PhysicalKey::Code(KeyCode::Digit9) => Some(8),
        _ => None,
    }
}

fn handle_hotbar_scroll(delta: MouseScrollDelta, hotbar: &mut Hotbar) {
    let y = match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
    };
    if y.abs() > f32::EPSILON {
        hotbar.select_offset(if y > 0.0 { -1 } else { 1 });
    }
}

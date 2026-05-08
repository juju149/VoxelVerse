use crate::app::content_bootstrap::load_core_content;
use crate::diagnostics::{Console, SystemDiagnostics};
use crate::gameplay::{
    BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode, PlanetResize,
    PlanetResizeIntent, Player, PlayerController,
};
use crate::input::Controller;
use crate::rendering::Renderer;
use crate::world::PlanetData;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowBuilder};

pub fn run() {
    SystemDiagnostics::print_startup_info();

    let content = load_core_content();
    let event_loop = EventLoop::new().unwrap();
    let window = create_window(&event_loop);
    grab_cursor(&window);

    let mut renderer = pollster::block_on(Renderer::new(&window, &content.textures));
    renderer.render_loading(0.0, "Initialisation planète");
    let mut controller = Controller::new();
    let mut player = Player::new();
    let mut planet = PlanetData::new_with_progress(
        content.planet,
        content.blocks,
        content.procedural,
        content.procedural_planet_index,
        |progress, message| renderer.render_loading(progress, message),
    );
    renderer.render_loading(1.0, "Monde prêt");
    renderer.window.set_title("voxelverse");
    let mut console = create_console();

    player.spawn(planet.spawn_position());
    let mut last_time = Instant::now();
    let mut cursor_grabbed = false;

    event_loop
        .run(move |event, target| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } if !console.is_open => {
                controller.process_mouse_motion(delta);
            }
            Event::WindowEvent { event, window_id } if window_id == renderer.window.id() => {
                if handle_console_event(&event, &mut console, &mut player) {
                    return;
                }

                if console.is_open {
                    handle_console_window_event(
                        event,
                        target,
                        &mut renderer,
                        &controller,
                        &player,
                        &planet,
                        &console,
                    );
                    return;
                }

                controller.process_events(&event, &player);
                handle_game_window_event(
                    event,
                    target,
                    &mut renderer,
                    &mut controller,
                    &mut player,
                    &mut planet,
                    &console,
                );
            }
            Event::AboutToWait => {
                let now = Instant::now();
                let dt = (now - last_time).as_secs_f32();
                last_time = now;

                sync_cursor_mode(
                    renderer.window,
                    controller.first_person,
                    console.is_open,
                    &mut cursor_grabbed,
                );
                tick_game_frame(
                    dt,
                    &mut renderer,
                    &mut controller,
                    &mut player,
                    &mut planet,
                    &mut console,
                );
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

fn create_console() -> Console {
    let mut console = Console::new();
    console.log("Welcome to voxelverse.", [0.0, 1.0, 0.0]);
    console.log("Press ` to open console.", [1.0, 1.0, 1.0]);
    console
}

fn grab_cursor(window: &Window) {
    let _ = window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
    window.set_cursor_visible(false);
}

fn release_cursor(window: &Window) {
    let _ = window.set_cursor_grab(CursorGrabMode::None);
    window.set_cursor_visible(true);
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
    event: WindowEvent,
    target: &winit::event_loop::EventLoopWindowTarget<()>,
    renderer: &mut Renderer<'_>,
    controller: &Controller,
    player: &Player,
    planet: &PlanetData,
    console: &Console,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
        WindowEvent::RedrawRequested => renderer.render(controller, player, planet, console),
        _ => {}
    }
}

fn handle_game_window_event(
    event: WindowEvent,
    target: &winit::event_loop::EventLoopWindowTarget<()>,
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &mut Player,
    planet: &mut PlanetData,
    console: &Console,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
        WindowEvent::Focused(true) if controller.first_person && !console.is_open => {
            grab_cursor(renderer.window);
        }
        WindowEvent::Focused(true) => {}
        WindowEvent::Focused(false) => release_cursor(renderer.window),
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button,
            ..
        } => handle_mouse_action(button, renderer, controller, player, planet),
        WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
            handle_pressed_key(event.physical_key, renderer, player, planet);
        }
        WindowEvent::RedrawRequested => renderer.render(controller, player, planet, console),
        _ => {}
    }
}

fn handle_mouse_action(
    button: MouseButton,
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &Player,
    planet: &mut PlanetData,
) {
    let intent = match button {
        MouseButton::Right => Some(BlockActionIntent::Place),
        MouseButton::Left => Some(BlockActionIntent::Mine),
        _ => None,
    };

    let Some(intent) = intent else {
        return;
    };

    let placement = if intent == BlockActionIntent::Place {
        let ray = controller.view_ray(
            player,
            renderer.config.width as f32,
            renderer.config.height as f32,
        );
        BlockSelection::trace(
            ray,
            controller.interaction_reach(),
            planet,
            BlockSelectionMode::Placement,
        )
        .map(|(id, _)| id)
    } else {
        None
    };

    if let Some(action) = BlockInteraction::resolve(intent, controller.cursor_id, placement) {
        let edit = BlockInteraction::apply(action, planet);
        let _changed_voxel = edit.changed;
        renderer.refresh_dirty_chunks(edit.dirty_chunks);
        renderer.window.request_redraw();
    } else if controller.cursor_id.is_none() && controller.first_person {
        grab_cursor(renderer.window);
    }
}

fn handle_pressed_key(
    key: PhysicalKey,
    renderer: &mut Renderer<'_>,
    player: &mut Player,
    planet: &mut PlanetData,
) {
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

    // Quality hotkeys — pure renderer-side toggles, no chunk reload required.
    match key {
        PhysicalKey::Code(KeyCode::F5) => {
            renderer.quality.triplanar_grain = !renderer.quality.triplanar_grain;
            println!(
                "[quality] triplanar grain = {}",
                renderer.quality.triplanar_grain
            );
        }
        PhysicalKey::Code(KeyCode::F6) => {
            use crate::rendering::PcfQuality;
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

fn sync_cursor_mode(
    window: &Window,
    first_person: bool,
    console_open: bool,
    cursor_grabbed: &mut bool,
) {
    let should_grab = first_person && !console_open;
    if should_grab == *cursor_grabbed {
        return;
    }

    *cursor_grabbed = should_grab;
    if should_grab {
        grab_cursor(window);
    } else {
        release_cursor(window);
    }
}

fn tick_game_frame(
    dt: f32,
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &mut Player,
    planet: &mut PlanetData,
    console: &mut Console,
) {
    console.update_animation(dt);

    if !console.is_open {
        let player_input = controller.sample_player_input();
        PlayerController::update(player, planet, player_input, dt);

        let width = renderer.config.width as f32;
        let height = renderer.config.height as f32;
        let ray = controller.view_ray(player, width, height);
        let ray_result = BlockSelection::trace(
            ray,
            controller.interaction_reach(),
            planet,
            BlockSelectionMode::HitSolid,
        );
        controller.cursor_id = ray_result.map(|(id, _)| id);
    } else {
        controller.clear_transient_input();
        controller.cursor_id = None;
        release_cursor(renderer.window);
    }

    renderer.update_cursor(planet, controller.cursor_id);
    renderer.update_view(player.position, planet);
    renderer.window.request_redraw();
}

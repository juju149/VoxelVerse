use crate::app::content_bootstrap::load_core_content;
use crate::diagnostics::{Console, SystemDiagnostics};
use crate::gameplay::{
    BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode, Hotbar, HotbarNotice,
    PlanetResize, PlanetResizeIntent, Player, PlayerController,
};
use crate::input::Controller;
use crate::rendering::Renderer;
use crate::world::{PlanetData, VoxModelRegistry};
use std::sync::Arc;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowBuilder};

pub fn run() {
    SystemDiagnostics::print_startup_info();

    let content = load_core_content();
    let event_loop = EventLoop::new().unwrap();
    let window = create_window(&event_loop);
    grab_cursor(&window);

    let mut renderer = pollster::block_on(Renderer::new(
        &window,
        &content.textures,
        content.blocks.material_colors(),
        &content.render,
    ));
    renderer.render_loading(0.0, "Initialisation planète");
    let mut controller = Controller::new();
    let mut player = Player::new();
    let mut hotbar = Hotbar::new();

    // Pre-load only the .vox prop models referenced by scatter variant defs.
    renderer.render_loading(0.05, "Chargement des modèles vox…");
    let prop_models = Arc::new(VoxModelRegistry::load_all(
        &content.core_pack_dir,
        &content.vox_asset_paths,
        &content.needed_vox_keys,
    ));

    let mut planet = PlanetData::new_with_progress(
        content.planet,
        content.blocks,
        content.terrain_visuals,
        content.procedural,
        content.procedural_planet_index,
        prop_models,
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
                        &hotbar,
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
                    &mut hotbar,
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
                    &mut hotbar,
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
    hotbar: &Hotbar,
    console: &Console,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
        WindowEvent::RedrawRequested => {
            renderer.render(controller, player, planet, hotbar, console)
        }
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
    hotbar: &mut Hotbar,
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
        } => handle_mouse_action(button, renderer, controller, player, planet, hotbar),
        WindowEvent::MouseWheel { delta, .. } if controller.first_person => {
            handle_hotbar_scroll(delta, hotbar);
        }
        WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
            handle_pressed_key(event.physical_key, renderer, player, planet, hotbar);
        }
        WindowEvent::RedrawRequested => {
            renderer.render(controller, player, planet, hotbar, console)
        }
        _ => {}
    }
}

fn handle_mouse_action(
    button: MouseButton,
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &Player,
    planet: &mut PlanetData,
    hotbar: &mut Hotbar,
) {
    let intent = match button {
        MouseButton::Right => Some(BlockActionIntent::Place),
        MouseButton::Left => Some(BlockActionIntent::Mine),
        _ => None,
    };

    let Some(intent) = intent else {
        return;
    };

    let active_voxel = if intent == BlockActionIntent::Place {
        match hotbar.selected_voxel() {
            Some(voxel) => Some(voxel),
            None => {
                hotbar.show_notice(HotbarNotice::EmptySlot);
                renderer.window.request_redraw();
                return;
            }
        }
    } else {
        None
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

    let mined_voxel = if intent == BlockActionIntent::Mine {
        let Some(coord) = controller.cursor_id else {
            if controller.first_person {
                grab_cursor(renderer.window);
            }
            return;
        };
        let voxel = planet.get_voxel(coord);
        if !hotbar.can_accept(voxel) {
            hotbar.show_notice(HotbarNotice::Full);
            renderer.window.request_redraw();
            return;
        }
        Some(voxel)
    } else {
        None
    };

    if intent == BlockActionIntent::Place && placement.is_none() {
        hotbar.show_notice(HotbarNotice::InvalidPlacement);
        renderer.window.request_redraw();
        return;
    }

    if let Some(action) =
        BlockInteraction::resolve(intent, controller.cursor_id, placement, active_voxel)
    {
        let edit = BlockInteraction::apply(action, planet);
        let changed = !edit.dirty_chunks.is_empty();
        if changed {
            match intent {
                BlockActionIntent::Mine => {
                    if let Some(voxel) = mined_voxel {
                        hotbar.add(voxel);
                    }
                }
                BlockActionIntent::Place => {
                    hotbar.consume_selected();
                }
            }
            renderer.refresh_dirty_chunks(edit.dirty_chunks);
            renderer.window.request_redraw();
        } else if intent == BlockActionIntent::Mine {
            hotbar.show_notice(HotbarNotice::ProtectedBlock);
            renderer.window.request_redraw();
        }
    } else if controller.cursor_id.is_none() && controller.first_person {
        grab_cursor(renderer.window);
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

    // Quality hotkeys — pure renderer-side toggles, no chunk reload required.
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
    hotbar: &mut Hotbar,
    console: &mut Console,
) {
    console.update_animation(dt);
    hotbar.update(dt);

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

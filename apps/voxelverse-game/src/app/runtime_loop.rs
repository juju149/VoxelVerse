use crate::app::content_bootstrap::load_core_content;
use crate::app::golden_scene::{golden_scene_enabled, GoldenScene};
use crate::app::inventory_events::handle_inventory_window_event;
use crate::ui::InventoryUiState;
use std::sync::Arc;
use std::time::Instant;
use vv_diagnostics::SystemDiagnostics;
use vv_gameplay::{
    BlockAction, BlockActionIntent, BlockInteraction, BlockSelection, BlockSelectionMode, Console,
    Controller, Hotbar, HotbarNotice, Inventory, ItemId, MiningProgress, PlanetResize,
    PlanetResizeIntent, Player, PlayerController,
};
use vv_pack_compiler::{LootRegistry, RecipeRegistry};
use vv_render::{Renderer, StreamingView};
use vv_world::{PlanetData, VoxModelRegistry};
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowBuilder};

pub fn run() {
    SystemDiagnostics::print_startup_info();

    let mut content = load_core_content();
    let golden_scene = golden_scene_enabled().then_some(GoldenScene::DEFAULT);
    if let Some(scene) = golden_scene {
        content.planet = scene.apply_planet(content.planet);
        println!(
            "[engine/golden] enabled seed={} resolution={} fixed_time_s={:.1}",
            scene.seed, scene.resolution, scene.fixed_elapsed_secs
        );
    }
    let event_loop = EventLoop::new().unwrap();
    let window = create_window(&event_loop);
    grab_cursor(&window);

    let mut renderer = pollster::block_on(Renderer::new(
        &window,
        &content.textures,
        content.blocks.material_colors(),
        &content.core_pack_dir,
    ));
    if let Some(scene) = golden_scene {
        renderer.quality = scene.quality;
        renderer.set_fixed_elapsed_secs(Some(scene.fixed_elapsed_secs));
        renderer.set_engine_debug_page(true);
    }
    renderer.render_loading(0.0, "Initialisation planète");
    let mut controller = Controller::new();
    let mut player = Player::new();
    let mut hotbar = Hotbar::new();
    let mut inventory = Inventory::new();
    let mut inventory_ui = InventoryUiState::new();
    let mut mining = MiningProgress::default();
    let mut mining_button_held = false;
    // Latest known shift state — updated on every ModifiersChanged event.
    // Used by the inventory's quick-move (shift + left click) shortcut.
    let mut shift_held = false;

    // Pre-load only the .vox prop models referenced by scatter variant defs.
    renderer.render_loading(0.05, "Chargement des modèles vox…");
    let prop_models = Arc::new(VoxModelRegistry::load_all(
        &content.core_pack_dir,
        &content.vox_asset_paths,
        &content.needed_vox_keys,
    ));

    // Keep loot + tag registries accessible after content is moved into planet.
    let loot = Arc::clone(&content.loot);
    let tags = Arc::clone(&content.tags);
    let recipes = Arc::clone(&content.recipes);

    let mut planet = PlanetData::new_with_progress(
        content.planet,
        content.blocks,
        content.items,
        content.terrain_visuals,
        content.procedural,
        content.procedural_planet_index,
        prop_models,
        |progress, message| renderer.render_loading(progress, message),
    );
    renderer.render_loading(1.0, "Monde prêt");
    renderer.window.set_title("voxelverse");
    let mut console = create_console();

    if let Some(scene) = golden_scene {
        scene.spawn_player(&mut player, &planet);
    } else {
        player.spawn(planet.spawn_position());
    }
    renderer.log_engine_snapshot("startup", &planet);
    let mut last_time = Instant::now();
    let mut cursor_grabbed = false;
    let mut first_scene_snapshot_logged = false;

    event_loop
        .run(move |event, target| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } if !console.is_open && !inventory_ui.is_open => {
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
                        &inventory,
                        &inventory_ui,
                        &recipes,
                        &console,
                    );
                    return;
                }

                if inventory_ui.is_open {
                    handle_inventory_window_event(
                        event,
                        target,
                        &mut renderer,
                        &mut controller,
                        &player,
                        &planet,
                        &mut hotbar,
                        &mut inventory,
                        &mut inventory_ui,
                        &recipes,
                        &tags,
                        &mut shift_held,
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
                    &mut inventory,
                    &mut inventory_ui,
                    &mut mining,
                    &mut mining_button_held,
                    &console,
                    &recipes,
                );
            }
            Event::AboutToWait => {
                let now = Instant::now();
                let dt = (now - last_time).as_secs_f32();
                last_time = now;

                sync_cursor_mode(
                    renderer.window,
                    controller.first_person,
                    console.is_open || inventory_ui.is_open,
                    &mut cursor_grabbed,
                );
                tick_game_frame(
                    dt,
                    &mut renderer,
                    &mut controller,
                    &mut player,
                    &mut planet,
                    &mut hotbar,
                    &mut inventory,
                    &inventory_ui,
                    &mut mining,
                    mining_button_held,
                    &loot,
                    &mut console,
                    &mut first_scene_snapshot_logged,
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

pub(super) fn grab_cursor(window: &Window) {
    let _ = window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
    window.set_cursor_visible(false);
}

pub(super) fn release_cursor(window: &Window) {
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
    inventory: &Inventory,
    inventory_ui: &InventoryUiState,
    recipes: &RecipeRegistry,
    console: &Console,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
        WindowEvent::RedrawRequested => renderer.render(
            controller,
            player,
            planet,
            hotbar,
            inventory,
            inventory_ui,
            recipes,
            console,
        ),
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
    inventory: &mut Inventory,
    inventory_ui: &mut InventoryUiState,
    mining: &mut MiningProgress,
    mining_button_held: &mut bool,
    console: &Console,
    recipes: &RecipeRegistry,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
        WindowEvent::Focused(true) if controller.first_person && !console.is_open => {
            grab_cursor(renderer.window);
        }
        WindowEvent::Focused(true) => {}
        WindowEvent::Focused(false) => release_cursor(renderer.window),
        WindowEvent::MouseInput { state, button, .. } => match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => {
                *mining_button_held = true;
            }
            (MouseButton::Left, ElementState::Released) => {
                *mining_button_held = false;
                mining.cancel();
            }
            (MouseButton::Right, ElementState::Pressed) => {
                handle_place_action(renderer, controller, player, planet, hotbar)
            }
            _ => {}
        },
        WindowEvent::MouseWheel { delta, .. } if controller.first_person => {
            handle_hotbar_scroll(delta, hotbar);
        }
        WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
            if let PhysicalKey::Code(KeyCode::KeyE) = event.physical_key {
                inventory_ui.toggle();
                if inventory_ui.is_open {
                    release_cursor(renderer.window);
                }
                renderer.window.request_redraw();
                return;
            }
            handle_pressed_key(event.physical_key, renderer, player, planet, hotbar);
        }
        WindowEvent::RedrawRequested => renderer.render(
            controller,
            player,
            planet,
            hotbar,
            inventory,
            inventory_ui,
            recipes,
            console,
        ),
        _ => {}
    }
}

fn handle_place_action(
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &Player,
    planet: &mut PlanetData,
    hotbar: &mut Hotbar,
) {
    let selected = hotbar.selected_item_id();
    let active_voxel = match selected.and_then(|id| planet.resolve_item_voxel(id)) {
        Some(voxel) => Some(voxel),
        None => {
            if selected.is_none() {
                hotbar.show_notice(HotbarNotice::EmptySlot);
            } else {
                hotbar.show_notice(HotbarNotice::InvalidPlacement);
            }
            renderer.window.request_redraw();
            return;
        }
    };

    let ray = controller.view_ray(
        player,
        renderer.config.width as f32,
        renderer.config.height as f32,
    );
    let placement = BlockSelection::trace(
        ray,
        controller.interaction_reach(),
        planet,
        BlockSelectionMode::Placement,
    )
    .map(|(id, _)| id);
    if placement.is_none() {
        hotbar.show_notice(HotbarNotice::InvalidPlacement);
        renderer.window.request_redraw();
        return;
    }

    if let Some(action) = BlockInteraction::resolve(
        BlockActionIntent::Place,
        controller.cursor_id,
        placement,
        active_voxel,
    ) {
        let edit = BlockInteraction::apply(action, planet);
        let changed = !edit.dirty_chunks.is_empty();
        if changed {
            hotbar.consume_selected();
            renderer.refresh_dirty_chunks(edit.dirty_chunks);
            renderer.window.request_redraw();
        }
    } else if controller.cursor_id.is_none() && controller.first_person {
        grab_cursor(renderer.window);
    }
}

#[allow(clippy::too_many_arguments)]
fn tick_mining(
    dt: f32,
    renderer: &mut Renderer<'_>,
    controller: &Controller,
    planet: &mut PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    mining: &mut MiningProgress,
    loot: &LootRegistry,
) {
    let coord = controller.cursor_id;
    let voxel = coord.map(|coord| planet.get_voxel(coord));
    let block = voxel.and_then(|voxel| planet.content.block(voxel));
    let outcome = mining.tick(
        dt,
        coord,
        voxel,
        block,
        hotbar.selected_item_id(),
        &planet.items,
    );
    let Some(outcome) = outcome else {
        return;
    };

    let edit = BlockInteraction::apply(BlockAction::Mine(outcome.coord), planet);
    if edit.dirty_chunks.is_empty() {
        hotbar.show_notice(HotbarNotice::ProtectedBlock);
        renderer.window.request_redraw();
        return;
    }

    if outcome.drops_enabled {
        if let Some(block) = planet.content.block(outcome.voxel) {
            for (item_id, count) in roll_block_drops(&block.drops_key, loot) {
                add_drop_to_player(item_id, count, planet, hotbar, inventory);
            }
        }
    }

    renderer.refresh_dirty_chunks(edit.dirty_chunks);
    renderer.window.request_redraw();
}

fn add_drop_to_player(
    item_id: ItemId,
    count: u32,
    planet: &PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
) {
    let item = planet.items.get(item_id);
    let max_stack = item.map(|i| i.stack_size.0).unwrap_or(99);
    if !hotbar.add(item_id, count, max_stack) && !inventory.add(item_id, count, max_stack) {
        hotbar.show_notice(HotbarNotice::Full);
    }
}

/// Roll the loot table identified by `drops_key`, returning `(ItemId, count)` pairs.
/// Falls back to an empty list for unknown tables.
fn roll_block_drops(drops_key: &str, loot: &LootRegistry) -> Vec<(ItemId, u32)> {
    match loot.get_by_key(drops_key) {
        Some(table) => {
            // Simple deterministic RNG for now — just use max chance.
            // Replace with a seeded PRNG when survival stakes are higher.
            table.roll(|| 0.0)
        }
        None => Vec::new(),
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

#[allow(clippy::too_many_arguments)]
fn tick_game_frame(
    dt: f32,
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &mut Player,
    planet: &mut PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    inventory_ui: &InventoryUiState,
    mining: &mut MiningProgress,
    mining_button_held: bool,
    loot: &LootRegistry,
    console: &mut Console,
    first_scene_snapshot_logged: &mut bool,
) {
    console.update_animation(dt);
    hotbar.update(dt);

    if !console.is_open && !inventory_ui.is_open {
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
        if mining_button_held {
            tick_mining(
                dt, renderer, controller, planet, hotbar, inventory, mining, loot,
            );
        } else {
            mining.cancel();
        }
    } else {
        controller.clear_transient_input();
        controller.cursor_id = None;
        mining.cancel();
        release_cursor(renderer.window);
    }

    renderer.update_cursor(planet, controller.cursor_id);
    let width = renderer.config.width as f32;
    let height = renderer.config.height as f32;
    let view_ray = controller.view_ray(player, width, height);
    renderer.update_view(
        StreamingView {
            player_pos: player.position,
            camera_pos: controller.get_camera_pos(player),
            view_dir: view_ray.direction,
            cursor_id: controller.cursor_id,
        },
        planet,
    );
    if !*first_scene_snapshot_logged && renderer.has_active_scene_chunks() {
        renderer.log_engine_snapshot("first-scene", planet);
        *first_scene_snapshot_logged = true;
    }
    renderer.window.request_redraw();
}

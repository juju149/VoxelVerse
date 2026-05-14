use crate::app::content_bootstrap::load_core_content;
use crate::ui::{
    HeldStack, InventoryButton, InventoryLayout, InventoryUiState, UiTheme, UiViewport,
};
use std::sync::Arc;
use std::time::Instant;
use vv_diagnostics::SystemDiagnostics;
use vv_gameplay::{
    craft_recipe, quick_craft_recipe_indices, BlockActionIntent, BlockInteraction, BlockSelection,
    BlockSelectionMode, Console, Controller, Hotbar, HotbarNotice, HotbarSlot, Inventory, ItemId,
    PlanetResize, PlanetResizeIntent, Player, PlayerController, SlotRef,
};
use vv_pack_compiler::{CompiledRecipe, LootRegistry, RecipeRegistry, TagRegistry};
use vv_render::Renderer;
use vv_world::{PlanetData, VoxModelRegistry};
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
        &content.core_pack_dir,
    ));
    renderer.render_loading(0.0, "Initialisation planète");
    let mut controller = Controller::new();
    let mut player = Player::new();
    let mut hotbar = Hotbar::new();
    let mut inventory = Inventory::new();
    let mut inventory_ui = InventoryUiState::new();
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

    player.spawn(planet.spawn_position());
    let mut last_time = Instant::now();
    let mut cursor_grabbed = false;

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
                    &console,
                    &recipes,
                    &loot,
                    &tags,
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
                    &inventory,
                    &inventory_ui,
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
    console: &Console,
    recipes: &RecipeRegistry,
    loot: &LootRegistry,
    tags: &TagRegistry,
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
        } => handle_mouse_action(
            button, renderer, controller, player, planet, hotbar, inventory, loot, tags,
        ),
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

#[allow(clippy::too_many_arguments)]
fn handle_inventory_window_event(
    event: WindowEvent,
    target: &winit::event_loop::EventLoopWindowTarget<()>,
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &Player,
    planet: &PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    inventory_ui: &mut InventoryUiState,
    recipes: &RecipeRegistry,
    tags: &TagRegistry,
    shift_held: &mut bool,
    console: &Console,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
        WindowEvent::Focused(false) => release_cursor(renderer.window),
        WindowEvent::ModifiersChanged(mods) => {
            *shift_held = mods.state().shift_key();
        }
        WindowEvent::CursorMoved { position, .. } => {
            inventory_ui.cursor = (position.x as f32, position.y as f32);
            let theme = UiTheme::VOXELVERSE;
            let vp = UiViewport::new(renderer.config.width as f32, renderer.config.height as f32);
            let layout = InventoryLayout::compute(&theme, vp, inventory_ui.user_zoom);
            let (px, py) = (position.x as f32, position.y as f32);
            inventory_ui.hovered_slot = layout.slot_under_cursor(px, py);
            inventory_ui.hovered_button = layout.button_under_cursor(px, py);
            inventory_ui.hovered_search = layout.search_bar.contains(px, py);
            inventory_ui.hovered_filter = layout.filter_under_cursor(px, py);
            inventory_ui.hovered_recipe = layout
                .recipe_under_cursor(px, py)
                .and_then(|row| quick_craft_recipe_indices(recipes).get(row).copied());
            renderer.window.request_redraw();
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => {
            handle_inventory_left_click(
                hotbar,
                inventory,
                inventory_ui,
                planet,
                recipes,
                tags,
                *shift_held,
            );
            renderer.window.request_redraw();
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Right,
            ..
        } => {
            handle_inventory_right_click(hotbar, inventory, inventory_ui);
            renderer.window.request_redraw();
        }
        WindowEvent::KeyboardInput { event: key, .. } if key.state == ElementState::Pressed => {
            handle_inventory_key(
                key.physical_key,
                key.text.as_deref(),
                renderer,
                controller,
                hotbar,
                inventory,
                inventory_ui,
            );
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

#[allow(clippy::too_many_arguments)]
fn handle_inventory_key(
    key: PhysicalKey,
    text: Option<&str>,
    renderer: &Renderer<'_>,
    controller: &Controller,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    ui: &mut InventoryUiState,
) {
    // Focused search bar: keys go to the query, Escape just unfocuses.
    if ui.search_focused {
        match key {
            PhysicalKey::Code(KeyCode::Escape) => {
                ui.search_focused = false;
            }
            PhysicalKey::Code(KeyCode::Backspace) => {
                ui.search_query.pop();
            }
            _ => {
                if let Some(text) = text {
                    for ch in text.chars() {
                        if ch.is_control() {
                            continue;
                        }
                        if ui.search_query.chars().count() < 32 {
                            ui.search_query.push(ch);
                        }
                    }
                }
            }
        }
        renderer.window.request_redraw();
        return;
    }

    // Unfocused: Minecraft-style shortcuts.
    match key {
        PhysicalKey::Code(KeyCode::Escape) | PhysicalKey::Code(KeyCode::KeyE) => {
            close_inventory(renderer, controller, hotbar, inventory, ui);
        }
        PhysicalKey::Code(KeyCode::KeyQ) => {
            // Q drops one item from the hovered slot (item is discarded —
            // we don't have ground items yet).
            if ui.held.is_some() {
                // While holding, Q drops one from held (Minecraft parity).
                drop_one_from_held(ui);
            } else if let Some(slot_ref) = ui.hovered_slot {
                drop_one_from_slot(hotbar, inventory, slot_ref);
            }
            renderer.window.request_redraw();
        }
        // Number keys 1-9 with cursor over a slot: swap with hotbar[N].
        PhysicalKey::Code(code) => {
            if let Some(idx) = digit_for_keycode(code) {
                if let Some(slot_ref) = ui.hovered_slot {
                    if !matches!(slot_ref, SlotRef::Hotbar(i) if i == idx) {
                        swap_with_hotbar(hotbar, inventory, slot_ref, idx);
                    }
                    renderer.window.request_redraw();
                } else {
                    // Cursor not on a slot: just change selection (same as
                    // in-game number keys).
                    hotbar.select(idx);
                    renderer.window.request_redraw();
                }
            }
        }
        _ => {}
    }
}

fn digit_for_keycode(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Digit1 => Some(0),
        KeyCode::Digit2 => Some(1),
        KeyCode::Digit3 => Some(2),
        KeyCode::Digit4 => Some(3),
        KeyCode::Digit5 => Some(4),
        KeyCode::Digit6 => Some(5),
        KeyCode::Digit7 => Some(6),
        KeyCode::Digit8 => Some(7),
        KeyCode::Digit9 => Some(8),
        _ => None,
    }
}

fn close_inventory(
    renderer: &Renderer<'_>,
    controller: &Controller,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    ui: &mut InventoryUiState,
) {
    // Return any held stack to its source (or anywhere available).
    if let Some(held) = ui.held.take() {
        return_held(hotbar, inventory, held);
    }
    ui.close();
    if controller.first_person {
        grab_cursor(renderer.window);
    }
    renderer.window.request_redraw();
}

fn return_held(hotbar: &mut Hotbar, inventory: &mut Inventory, held: HeldStack) {
    let source_empty = read_slot(hotbar, inventory, held.source).is_none();
    if source_empty {
        place_into(hotbar, inventory, held.source, held.stack);
        return;
    }
    // Source no longer empty (player put something else there) — spill into
    // the inventory one unit at a time.
    for _ in 0..held.stack.quantity {
        if !inventory.add(held.stack.item_id, 1, 99) {
            break;
        }
    }
}

fn handle_inventory_left_click(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    ui: &mut InventoryUiState,
    planet: &PlanetData,
    recipes: &RecipeRegistry,
    tags: &TagRegistry,
    shift: bool,
) {
    // Search bar focus: clicking the bar focuses, clicking elsewhere
    // unfocuses. Buttons take priority over slot/search clicks.
    if let Some(button) = ui.hovered_button {
        if button == InventoryButton::ClearSearch && ui.search_query.is_empty() {
            // No-op: button isn't rendered for an empty query anyway.
        } else {
            ui.search_focused = false;
            match button {
                InventoryButton::Close => {
                    if let Some(held) = ui.held.take() {
                        return_held(hotbar, inventory, held);
                    }
                    ui.close();
                }
                InventoryButton::Sort => inventory.sort(),
                InventoryButton::ClearSearch => ui.search_query.clear(),
                InventoryButton::CraftQuantityDown => {
                    ui.craft_quantity = ui.craft_quantity.saturating_sub(1).max(1);
                }
                InventoryButton::CraftQuantityUp => {
                    ui.craft_quantity = ui.craft_quantity.saturating_add(1).min(99);
                }
                InventoryButton::CraftMax => {
                    ui.craft_quantity = selected_recipe_index(ui, recipes)
                        .and_then(|idx| recipes.recipes().get(idx))
                        .map(|recipe| {
                            max_craft_quantity(recipe, planet, tags, hotbar, inventory).max(1)
                        })
                        .unwrap_or(1);
                }
                InventoryButton::Craft => {
                    if let Some(recipe) = selected_recipe_index(ui, recipes)
                        .and_then(|idx| recipes.recipes().get(idx))
                    {
                        let _ = craft_recipe(
                            recipe,
                            &planet.items,
                            tags,
                            hotbar,
                            inventory,
                            ui.craft_quantity,
                        );
                    }
                }
            }
            return;
        }
    }

    if ui.hovered_search {
        ui.search_focused = true;
        return;
    }
    // Any other click defocuses the search.
    ui.search_focused = false;
    if let Some(filter) = ui.hovered_filter {
        ui.active_filter = filter;
        return;
    }
    if let Some(recipe_index) = ui.hovered_recipe {
        ui.selected_recipe = Some(recipe_index);
        return;
    }
    // Slot logic.
    let Some(target) = ui.hovered_slot else {
        return;
    };

    if shift {
        quick_move(hotbar, inventory, target);
        return;
    }

    // Pick / drop logic.
    match ui.held.take() {
        None => {
            // Pick up the entire stack.
            if let Some(stack) = read_slot(hotbar, inventory, target) {
                place_into_optional(hotbar, inventory, target, None);
                ui.held = Some(HeldStack {
                    stack,
                    source: target,
                });
            }
        }
        Some(held) => {
            match read_slot(hotbar, inventory, target) {
                None => {
                    place_into(hotbar, inventory, target, held.stack);
                }
                Some(existing) if existing.item_id == held.stack.item_id => {
                    let merged = HotbarSlot {
                        item_id: held.stack.item_id,
                        quantity: existing.quantity.saturating_add(held.stack.quantity),
                    };
                    place_into(hotbar, inventory, target, merged);
                }
                Some(existing) => {
                    // Different items: swap.
                    place_into(hotbar, inventory, target, held.stack);
                    ui.held = Some(HeldStack {
                        stack: existing,
                        source: target,
                    });
                }
            }
        }
    }
}

fn handle_inventory_right_click(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    ui: &mut InventoryUiState,
) {
    // Right click on a button cancels it; only slots react.
    let Some(target) = ui.hovered_slot else {
        return;
    };
    match ui.held.take() {
        None => {
            // Pick up half (rounded up).
            if let Some(stack) = read_slot(hotbar, inventory, target) {
                if stack.quantity <= 1 {
                    place_into_optional(hotbar, inventory, target, None);
                    ui.held = Some(HeldStack {
                        stack,
                        source: target,
                    });
                } else {
                    let half_up = stack.quantity.div_ceil(2);
                    let remaining = stack.quantity - half_up;
                    place_into(
                        hotbar,
                        inventory,
                        target,
                        HotbarSlot {
                            item_id: stack.item_id,
                            quantity: remaining,
                        },
                    );
                    ui.held = Some(HeldStack {
                        stack: HotbarSlot {
                            item_id: stack.item_id,
                            quantity: half_up,
                        },
                        source: target,
                    });
                }
            }
        }
        Some(mut held) => {
            match read_slot(hotbar, inventory, target) {
                None => {
                    // Drop one item from held into the empty slot.
                    place_into(
                        hotbar,
                        inventory,
                        target,
                        HotbarSlot {
                            item_id: held.stack.item_id,
                            quantity: 1,
                        },
                    );
                    held.stack.quantity -= 1;
                    if held.stack.quantity > 0 {
                        ui.held = Some(held);
                    }
                }
                Some(existing) if existing.item_id == held.stack.item_id => {
                    let merged = HotbarSlot {
                        item_id: held.stack.item_id,
                        quantity: existing.quantity.saturating_add(1),
                    };
                    place_into(hotbar, inventory, target, merged);
                    held.stack.quantity -= 1;
                    if held.stack.quantity > 0 {
                        ui.held = Some(held);
                    }
                }
                Some(_) => {
                    // Different item: treat right click as a swap (Minecraft
                    // parity).
                    let existing = read_slot(hotbar, inventory, target).unwrap();
                    place_into(hotbar, inventory, target, held.stack);
                    ui.held = Some(HeldStack {
                        stack: existing,
                        source: target,
                    });
                }
            }
        }
    }
}

fn selected_recipe_index(ui: &InventoryUiState, recipes: &RecipeRegistry) -> Option<usize> {
    let indices = quick_craft_recipe_indices(recipes);
    ui.selected_recipe
        .filter(|selected| indices.contains(selected))
        .or_else(|| indices.first().copied())
}

fn max_craft_quantity(
    recipe: &CompiledRecipe,
    planet: &PlanetData,
    tags: &TagRegistry,
    hotbar: &Hotbar,
    inventory: &Inventory,
) -> u32 {
    let mut max = 0;
    for quantity in 1..=99 {
        let mut trial_hotbar = hotbar.clone();
        let mut trial_inventory = inventory.clone();
        if craft_recipe(
            recipe,
            &planet.items,
            tags,
            &mut trial_hotbar,
            &mut trial_inventory,
            quantity,
        )
        .is_ok()
        {
            max = quantity;
        } else {
            break;
        }
    }
    max
}

fn quick_move(hotbar: &mut Hotbar, inventory: &mut Inventory, source: SlotRef) {
    let Some(stack) = read_slot(hotbar, inventory, source) else {
        return;
    };
    match source {
        SlotRef::Inventory(_) => {
            // Inventory → hotbar: stack into matching slot, else first empty.
            place_into_optional(hotbar, inventory, source, None);
            let mut slots = *hotbar.slots();
            if let Some(slot) = slots
                .iter_mut()
                .flatten()
                .find(|s| s.item_id == stack.item_id)
            {
                slot.quantity = slot.quantity.saturating_add(stack.quantity);
            } else if let Some(slot) = slots.iter_mut().find(|s| s.is_none()) {
                *slot = Some(stack);
            } else {
                // No room — put back.
                place_into(hotbar, inventory, source, stack);
                return;
            }
            hotbar.set_slots(slots);
        }
        SlotRef::Hotbar(_) => {
            // Hotbar → inventory: stack into matching slot, else first empty.
            place_into_optional(hotbar, inventory, source, None);
            let mut placed = false;
            for slot in inventory
                .slots()
                .iter()
                .enumerate()
                .filter_map(|(i, s)| s.map(|s| (i, s)))
                .collect::<Vec<_>>()
            {
                let (idx, s) = slot;
                if s.item_id == stack.item_id {
                    inventory.set(
                        idx,
                        Some(HotbarSlot {
                            item_id: stack.item_id,
                            quantity: s.quantity.saturating_add(stack.quantity),
                        }),
                    );
                    placed = true;
                    break;
                }
            }
            if !placed {
                for (i, s) in inventory.slots().iter().enumerate() {
                    if s.is_none() {
                        inventory.set(i, Some(stack));
                        placed = true;
                        break;
                    }
                }
            }
            if !placed {
                // No room — put back.
                place_into(hotbar, inventory, source, stack);
            }
        }
    }
}

fn swap_with_hotbar(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    target: SlotRef,
    hotbar_index: usize,
) {
    let a = read_slot(hotbar, inventory, target);
    let b = hotbar.slots()[hotbar_index];
    place_into_optional(hotbar, inventory, target, b);
    let mut slots = *hotbar.slots();
    slots[hotbar_index] = a;
    hotbar.set_slots(slots);
}

fn drop_one_from_slot(hotbar: &mut Hotbar, inventory: &mut Inventory, slot: SlotRef) {
    let Some(stack) = read_slot(hotbar, inventory, slot) else {
        return;
    };
    if stack.quantity <= 1 {
        place_into_optional(hotbar, inventory, slot, None);
    } else {
        place_into(
            hotbar,
            inventory,
            slot,
            HotbarSlot {
                item_id: stack.item_id,
                quantity: stack.quantity - 1,
            },
        );
    }
}

fn drop_one_from_held(ui: &mut InventoryUiState) {
    if let Some(mut held) = ui.held.take() {
        held.stack.quantity = held.stack.quantity.saturating_sub(1);
        if held.stack.quantity > 0 {
            ui.held = Some(held);
        }
    }
}

fn read_slot(hotbar: &Hotbar, inventory: &Inventory, slot: SlotRef) -> Option<HotbarSlot> {
    match slot {
        SlotRef::Hotbar(i) => hotbar.slots()[i],
        SlotRef::Inventory(i) => inventory.slot(i),
    }
}

fn place_into(hotbar: &mut Hotbar, inventory: &mut Inventory, slot: SlotRef, stack: HotbarSlot) {
    place_into_optional(hotbar, inventory, slot, Some(stack));
}

fn place_into_optional(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    slot: SlotRef,
    stack: Option<HotbarSlot>,
) {
    match slot {
        SlotRef::Hotbar(i) => {
            let mut new_slots = *hotbar.slots();
            new_slots[i] = stack;
            hotbar.set_slots(new_slots);
        }
        SlotRef::Inventory(i) => {
            inventory.set(i, stack);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_mouse_action(
    button: MouseButton,
    renderer: &mut Renderer<'_>,
    controller: &mut Controller,
    player: &Player,
    planet: &mut PlanetData,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    loot: &LootRegistry,
    _tags: &TagRegistry,
) {
    let intent = match button {
        MouseButton::Right => Some(BlockActionIntent::Place),
        MouseButton::Left => Some(BlockActionIntent::Mine),
        _ => None,
    };

    let Some(intent) = intent else {
        return;
    };

    // Resolve which voxel the selected item would place (if placing).
    let active_voxel = if intent == BlockActionIntent::Place {
        let selected = hotbar.selected_item_id();
        match selected.and_then(|id| planet.resolve_item_voxel(id)) {
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
        Some(planet.get_voxel(coord))
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
                        // Roll the loot table for this block and hand items to
                        // the hotbar first, then the inventory as overflow.
                        let block = planet.content.block(voxel);
                        let drops = if let Some(block) = block {
                            roll_block_drops(&block.drops_key, loot)
                        } else {
                            Vec::new()
                        };

                        for (item_id, count) in drops {
                            let item = planet.items.get(item_id);
                            let max_stack = item.map(|i| i.stack_size.0).unwrap_or(99);
                            if !hotbar.add(item_id, count, max_stack) {
                                if !inventory.add(item_id, count, max_stack) {
                                    hotbar.show_notice(HotbarNotice::Full);
                                    break;
                                }
                            }
                        }
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
    _inventory: &Inventory,
    inventory_ui: &InventoryUiState,
    console: &mut Console,
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
    } else {
        controller.clear_transient_input();
        controller.cursor_id = None;
        release_cursor(renderer.window);
    }

    renderer.update_cursor(planet, controller.cursor_id);
    renderer.update_view(player.position, planet);
    renderer.window.request_redraw();
}

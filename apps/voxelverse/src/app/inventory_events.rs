use crate::app::cursor::{grab_cursor, release_cursor};
use crate::app::render_snapshot::frame_from_inventory_context;
use crate::app::runtime_state::InventoryInputContext;
use crate::ui::{
    HeldStack, InventoryButton, InventoryLayout, InventoryUiState, UiTheme, UiViewport,
};
use vv_gameplay::{craft_recipe, quick_craft_recipe_indices, Hotbar, HotbarSlot, Inventory};
use vv_pack_compiler::{CompiledRecipe, RecipeRegistry, TagRegistry};
use vv_render::{RenderItemStack, RenderSlotRef, Renderer};
use vv_world::PlanetData;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn handle_inventory_window_event(
    event: WindowEvent,
    target: &winit::event_loop::EventLoopWindowTarget<()>,
    renderer: &mut Renderer<'_>,
    ctx: &mut InventoryInputContext<'_>,
) {
    match event {
        WindowEvent::CloseRequested => target.exit(),
        WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
        WindowEvent::Focused(false) => release_cursor(renderer.window),
        WindowEvent::ModifiersChanged(mods) => {
            *ctx.shift_held = mods.state().shift_key();
        }
        WindowEvent::CursorMoved { position, .. } => {
            ctx.inventory_ui.cursor = (position.x as f32, position.y as f32);
            let theme = UiTheme::VOXELVERSE;
            let vp = UiViewport::new(renderer.config.width as f32, renderer.config.height as f32);
            let layout = InventoryLayout::compute(&theme, vp, ctx.inventory_ui.user_zoom);
            let (px, py) = (position.x as f32, position.y as f32);
            ctx.inventory_ui.hovered_slot = layout.slot_under_cursor(px, py);
            ctx.inventory_ui.hovered_button = layout.button_under_cursor(px, py);
            ctx.inventory_ui.hovered_search = layout.search_bar.contains(px, py);
            ctx.inventory_ui.hovered_filter = layout.filter_under_cursor(px, py);
            ctx.inventory_ui.hovered_recipe = layout
                .recipe_under_cursor(px, py)
                .and_then(|row| quick_craft_recipe_indices(ctx.recipes).get(row).copied());
            renderer.window.request_redraw();
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => {
            handle_inventory_left_click(
                ctx.hotbar,
                ctx.inventory,
                ctx.inventory_ui,
                ctx.planet,
                ctx.recipes,
                ctx.tags,
                *ctx.shift_held,
            );
            renderer.window.request_redraw();
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Right,
            ..
        } => {
            handle_inventory_right_click(ctx.hotbar, ctx.inventory, ctx.inventory_ui);
            renderer.window.request_redraw();
        }
        WindowEvent::KeyboardInput { event: key, .. } if key.state == ElementState::Pressed => {
            handle_inventory_key(
                key.physical_key,
                key.text.as_deref(),
                renderer,
                ctx.controller.first_person,
                ctx.hotbar,
                ctx.inventory,
                ctx.inventory_ui,
            );
        }
        WindowEvent::RedrawRequested => {
            let w = renderer.config.width as f32;
            let h = renderer.config.height as f32;
            let frame = frame_from_inventory_context(ctx, w, h);
            renderer.render(&frame);
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_inventory_key(
    key: PhysicalKey,
    text: Option<&str>,
    renderer: &Renderer<'_>,
    is_first_person: bool,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    ui: &mut InventoryUiState,
) {
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

    match key {
        PhysicalKey::Code(KeyCode::Escape) | PhysicalKey::Code(KeyCode::KeyE) => {
            close_inventory(renderer, is_first_person, hotbar, inventory, ui);
        }
        PhysicalKey::Code(KeyCode::KeyQ) => {
            if ui.held.is_some() {
                drop_one_from_held(ui);
            } else if let Some(slot_ref) = ui.hovered_slot {
                drop_one_from_slot(hotbar, inventory, slot_ref);
            }
            renderer.window.request_redraw();
        }
        PhysicalKey::Code(code) => {
            if let Some(idx) = digit_for_keycode(code) {
                if let Some(slot_ref) = ui.hovered_slot {
                    if !matches!(slot_ref, RenderSlotRef::Hotbar(i) if i == idx) {
                        swap_with_hotbar(hotbar, inventory, slot_ref, idx);
                    }
                    renderer.window.request_redraw();
                } else {
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
    is_first_person: bool,
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    ui: &mut InventoryUiState,
) {
    if let Some(held) = ui.held.take() {
        return_held(hotbar, inventory, held);
    }
    ui.close();
    if is_first_person {
        grab_cursor(renderer.window);
    }
    renderer.window.request_redraw();
}

fn return_held(hotbar: &mut Hotbar, inventory: &mut Inventory, held: HeldStack) {
    let source_empty = read_slot(hotbar, inventory, held.source).is_none();
    if source_empty {
        place_into(hotbar, inventory, held.source, gameplay_stack(held.stack));
        return;
    }
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
    if let Some(button) = ui.hovered_button {
        if button == InventoryButton::ClearSearch && ui.search_query.is_empty() {
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
    ui.search_focused = false;
    if let Some(filter) = ui.hovered_filter {
        ui.active_filter = filter;
        return;
    }
    if let Some(recipe_index) = ui.hovered_recipe {
        ui.selected_recipe = Some(recipe_index);
        return;
    }
    let Some(target) = ui.hovered_slot else {
        return;
    };

    if shift {
        quick_move(hotbar, inventory, target);
        return;
    }

    match ui.held.take() {
        None => {
            if let Some(stack) = read_slot(hotbar, inventory, target) {
                place_into_optional(hotbar, inventory, target, None);
                ui.held = Some(HeldStack {
                    stack: render_stack(stack),
                    source: target,
                });
            }
        }
        Some(held) => match read_slot(hotbar, inventory, target) {
            None => {
                place_into(hotbar, inventory, target, gameplay_stack(held.stack));
            }
            Some(existing) if existing.item_id == held.stack.item_id => {
                let merged = HotbarSlot {
                    item_id: held.stack.item_id,
                    quantity: existing.quantity.saturating_add(held.stack.quantity),
                };
                place_into(hotbar, inventory, target, merged);
            }
            Some(existing) => {
                place_into(hotbar, inventory, target, gameplay_stack(held.stack));
                ui.held = Some(HeldStack {
                    stack: render_stack(existing),
                    source: target,
                });
            }
        },
    }
}

fn handle_inventory_right_click(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    ui: &mut InventoryUiState,
) {
    let Some(target) = ui.hovered_slot else {
        return;
    };
    match ui.held.take() {
        None => {
            if let Some(stack) = read_slot(hotbar, inventory, target) {
                if stack.quantity <= 1 {
                    place_into_optional(hotbar, inventory, target, None);
                    ui.held = Some(HeldStack {
                        stack: render_stack(stack),
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
                        stack: RenderItemStack {
                            item_id: stack.item_id,
                            quantity: half_up,
                        },
                        source: target,
                    });
                }
            }
        }
        Some(mut held) => match read_slot(hotbar, inventory, target) {
            None => {
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
                let existing = read_slot(hotbar, inventory, target).unwrap();
                place_into(hotbar, inventory, target, gameplay_stack(held.stack));
                ui.held = Some(HeldStack {
                    stack: render_stack(existing),
                    source: target,
                });
            }
        },
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

fn quick_move(hotbar: &mut Hotbar, inventory: &mut Inventory, source: RenderSlotRef) {
    let Some(stack) = read_slot(hotbar, inventory, source) else {
        return;
    };
    match source {
        RenderSlotRef::Inventory(_) => {
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
                place_into(hotbar, inventory, source, stack);
                return;
            }
            hotbar.set_slots(slots);
        }
        RenderSlotRef::Hotbar(_) => {
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
                place_into(hotbar, inventory, source, stack);
            }
        }
    }
}

fn swap_with_hotbar(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    target: RenderSlotRef,
    hotbar_index: usize,
) {
    let a = read_slot(hotbar, inventory, target);
    let b = hotbar.slots()[hotbar_index];
    place_into_optional(hotbar, inventory, target, b);
    let mut slots = *hotbar.slots();
    slots[hotbar_index] = a;
    hotbar.set_slots(slots);
}

fn drop_one_from_slot(hotbar: &mut Hotbar, inventory: &mut Inventory, slot: RenderSlotRef) {
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

fn read_slot(hotbar: &Hotbar, inventory: &Inventory, slot: RenderSlotRef) -> Option<HotbarSlot> {
    match slot {
        RenderSlotRef::Hotbar(i) => hotbar.slots()[i],
        RenderSlotRef::Inventory(i) => inventory.slot(i),
    }
}

fn place_into(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    slot: RenderSlotRef,
    stack: HotbarSlot,
) {
    place_into_optional(hotbar, inventory, slot, Some(stack));
}

fn place_into_optional(
    hotbar: &mut Hotbar,
    inventory: &mut Inventory,
    slot: RenderSlotRef,
    stack: Option<HotbarSlot>,
) {
    match slot {
        RenderSlotRef::Hotbar(i) => {
            let mut new_slots = *hotbar.slots();
            new_slots[i] = stack;
            hotbar.set_slots(new_slots);
        }
        RenderSlotRef::Inventory(i) => {
            inventory.set(i, stack);
        }
    }
}

fn render_stack(stack: HotbarSlot) -> RenderItemStack {
    RenderItemStack {
        item_id: stack.item_id,
        quantity: stack.quantity,
    }
}

fn gameplay_stack(stack: RenderItemStack) -> HotbarSlot {
    HotbarSlot {
        item_id: stack.item_id,
        quantity: stack.quantity,
    }
}

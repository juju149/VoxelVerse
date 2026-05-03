use vv_gameplay::can_craft_hand_recipe;
use vv_registry::RecipeId;
use vv_ui::{
    UiBorder, UiColor, UiEdgeInsets, UiFrame, UiGradient, UiLayer, UiRect, UiShadow, UiTextAlign,
};

use crate::{
    ingredient_visuals, item_visual, screens::hotbar::draw_item_slot, GameplayUiContext,
    InventoryUiLayout,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct InventoryScreen;

impl InventoryScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let mut layout = InventoryUiLayout::inventory(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        layout.add_hand_recipes(ctx.content.recipes.recipes_for_station(None));

        frame.rect(
            UiLayer::Menu,
            UiRect::new(0.0, 0.0, ctx.screen_width, ctx.screen_height),
            ctx.theme.background_dim,
        );

        draw_title(frame, ctx, &layout);
        draw_panel(frame, layout.equipment_panel, ctx);
        draw_panel(frame, layout.backpack_panel, ctx);
        draw_panel(frame, layout.crafting_panel, ctx);

        draw_equipment_panel(frame, ctx, &layout);
        draw_backpack_panel(frame, ctx, &layout);
        draw_crafting_panel(frame, ctx, &layout);
        draw_bottom_hotbar(frame, ctx, &layout);
        draw_footer_hint(frame, ctx, &layout);
    }
}

fn draw_title(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let s = layout.scale;
    let title = layout.title_bar;

    let icon = UiRect::new(title.x, title.y + 4.0 * s, 62.0 * s, 62.0 * s);

    draw_square_button(frame, icon, "▣", false, ctx, s);

    frame.text(
        UiLayer::Menu,
        UiRect::new(
            icon.right() + 24.0 * s,
            title.y + 2.0 * s,
            420.0 * s,
            42.0 * s,
        ),
        "INVENTAIRE",
        32.0 * s,
        ctx.theme.text_primary,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(
            icon.right() + 24.0 * s,
            title.y + 44.0 * s,
            520.0 * s,
            26.0 * s,
        ),
        "Sac, équipement et ressources",
        15.0 * s,
        ctx.theme.text_muted,
    );

    let close_size = 54.0 * s;
    let close = UiRect::new(
        title.right() - close_size,
        title.y + 4.0 * s,
        close_size,
        close_size,
    );

    draw_square_button(frame, close, "×", false, ctx, s);
}

fn draw_panel(frame: &mut UiFrame, rect: UiRect, ctx: &GameplayUiContext<'_>) {
    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        ctx.theme.panel,
        12.0,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::new(0.0, 16.0, 34.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.38)),
    );
}

fn draw_equipment_panel(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
) {
    let rect = layout.equipment_panel;
    let s = layout.scale;
    let pad = 24.0 * s;

    frame.text_aligned(
        UiLayer::Menu,
        UiRect::new(rect.x, rect.y + 18.0 * s, rect.width, 28.0 * s),
        "ÉQUIPEMENT",
        18.0 * s,
        ctx.theme.accent,
        UiTextAlign::Center,
    );

    let avatar_h = (rect.height * 0.48).min(420.0 * s);
    let avatar = UiRect::new(
        rect.x + rect.width * 0.31,
        rect.y + 86.0 * s,
        rect.width * 0.38,
        avatar_h,
    );

    frame.gradient_rect(
        UiLayer::Menu,
        avatar,
        UiGradient::vertical(
            UiColor::rgba(0.55, 0.36, 0.18, 0.94),
            UiColor::rgba(0.17, 0.24, 0.19, 0.94),
        ),
        18.0 * s,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::new(0.0, 12.0, 24.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.25)),
    );

    draw_centered_text(frame, avatar, "PLAYER", 20.0 * s, ctx.theme.text_primary);

    let small = (58.0 * s).clamp(38.0, 72.0);
    let left_x = rect.x + pad;
    let right_x = rect.right() - pad - small;
    let y0 = rect.y + 72.0 * s;
    let available_h = (rect.height - 260.0 * s).max(small * 5.0);
    let step = ((available_h - small) / 4.0).clamp(small + 10.0 * s, small + 34.0 * s);

    for (i, label) in ["Casque", "Plastron", "Gants", "Pantalon", "Bottes"]
        .iter()
        .enumerate()
    {
        let slot = UiRect::new(left_x, y0 + i as f32 * step, small, small);
        draw_empty_equipment_slot(frame, slot, label, ctx, s);
    }

    for (i, label) in ["Amulette", "Sac", "Cape", "Torche"].iter().enumerate() {
        let slot = UiRect::new(right_x, y0 + i as f32 * step, small, small);
        draw_empty_equipment_slot(frame, slot, label, ctx, s);
    }

    frame.text_aligned(
        UiLayer::Menu,
        UiRect::new(rect.x, rect.bottom() - 128.0 * s, rect.width, 24.0 * s),
        "OUTILS RAPIDES",
        16.0 * s,
        ctx.theme.accent,
        UiTextAlign::Center,
    );

    let quick_slot = (58.0 * s).clamp(38.0, 68.0);
    let quick_gap = 28.0 * s;
    let total_quick_w = quick_slot * 3.0 + quick_gap * 2.0;
    let quick_x = rect.x + (rect.width - total_quick_w) * 0.5;
    let quick_y = rect.bottom() - 88.0 * s;

    for i in 0..3 {
        let tool_slot = UiRect::new(
            quick_x + i as f32 * (quick_slot + quick_gap),
            quick_y,
            quick_slot,
            quick_slot,
        );

        let stack = ctx
            .gameplay
            .inventory
            .slots()
            .get(i)
            .and_then(|slot| slot.stack);
        let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

        draw_item_slot(
            frame,
            tool_slot,
            false,
            visual.as_ref(),
            ctx.gameplay.inventory_drag.source_slot == Some(i),
            ctx.theme,
            UiLayer::Menu,
        );
    }
}

fn draw_empty_equipment_slot(
    frame: &mut UiFrame,
    rect: UiRect,
    label: &str,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    frame.text_aligned(
        UiLayer::Menu,
        UiRect::new(
            rect.x - 8.0 * scale,
            rect.y - 19.0 * scale,
            rect.width + 16.0 * scale,
            18.0 * scale,
        ),
        label,
        10.0 * scale,
        ctx.theme.text_muted,
        UiTextAlign::Center,
    );

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        ctx.theme.panel_subtle,
        7.0 * scale,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::NONE,
    );
}

fn draw_backpack_panel(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
) {
    let rect = layout.backpack_panel;
    let s = layout.scale;
    let pad = 24.0 * s;

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + pad, rect.y + 20.0 * s, 260.0 * s, 30.0 * s),
        "SAC À DOS",
        18.0 * s,
        ctx.theme.accent,
    );

    let search = UiRect::new(
        rect.x + pad,
        rect.y + 58.0 * s,
        rect.width - pad * 2.0,
        44.0 * s,
    );

    draw_search_field(frame, search, "Rechercher un objet...", ctx, s);
    draw_category_tabs(frame, ctx, layout);

    for slot in &layout.inventory_slots {
        if slot.index < ctx.gameplay.inventory.hotbar_len() {
            continue;
        }

        let stack = ctx.gameplay.inventory.slots()[slot.index].stack;
        let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

        draw_item_slot(
            frame,
            slot.rect,
            false,
            visual.as_ref(),
            ctx.gameplay.inventory_drag.source_slot == Some(slot.index),
            ctx.theme,
            UiLayer::Menu,
        );
    }

    draw_weight_and_sort(frame, ctx, layout);
}

fn draw_category_tabs(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
) {
    let s = layout.scale;
    let tab_y = layout.backpack_panel.y + 114.0 * s;
    let tab_h = 42.0 * s;
    let mut x = layout.backpack_panel.x + 24.0 * s;
    let max_right = layout.backpack_panel.right() - 24.0 * s;

    for (i, label) in ["Tout", "Ressources", "Outils", "Nourriture", "Divers"]
        .iter()
        .enumerate()
    {
        let preferred_w = match i {
            0 => 84.0 * s,
            1 => 132.0 * s,
            2 => 102.0 * s,
            3 => 132.0 * s,
            _ => 96.0 * s,
        };

        let remaining = max_right - x;
        if remaining < 58.0 * s {
            break;
        }

        let w = preferred_w.min(remaining);
        let rect = UiRect::new(x, tab_y, w, tab_h);

        draw_pill_button(frame, rect, label, i == 0, ctx, s);

        x += w + 8.0 * s;
    }
}

fn draw_weight_and_sort(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
) {
    let s = layout.scale;
    let rect = layout.backpack_panel;
    let y = rect.bottom() - 70.0 * s;

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + 48.0 * s, y, 180.0 * s, 24.0 * s),
        "46,2 / 120 kg",
        15.0 * s,
        ctx.theme.text_primary,
    );

    frame.rounded_rect(
        UiLayer::Menu,
        UiRect::new(rect.x + 24.0 * s, y + 34.0 * s, 180.0 * s, 10.0 * s),
        ctx.theme.panel_subtle,
        999.0,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::NONE,
    );

    frame.rounded_rect(
        UiLayer::Menu,
        UiRect::new(rect.x + 24.0 * s, y + 34.0 * s, 74.0 * s, 10.0 * s),
        ctx.theme.accent,
        999.0,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    let sort = UiRect::new(rect.right() - 138.0 * s, y - 2.0 * s, 114.0 * s, 48.0 * s);
    draw_pill_button(frame, sort, "↕ Trier", false, ctx, s);
}

fn draw_crafting_panel(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
) {
    let rect = layout.crafting_panel;
    let s = layout.scale;
    let pad = 24.0 * s;

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + pad, rect.y + 20.0 * s, 320.0 * s, 28.0 * s),
        "ARTISANAT RAPIDE",
        18.0 * s,
        ctx.theme.text_primary,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + pad, rect.y + 48.0 * s, 360.0 * s, 24.0 * s),
        "Fabrication à la main uniquement",
        12.0 * s,
        ctx.theme.text_muted,
    );

    let selected_recipe = layout.recipe_slots.first().map(|slot| slot.recipe);

    for slot in &layout.recipe_slots {
        draw_recipe_row(
            frame,
            ctx,
            layout,
            slot.recipe,
            slot.rect,
            Some(slot.recipe) == selected_recipe,
        );
    }

    if let Some(recipe) = selected_recipe {
        draw_recipe_detail(frame, ctx, layout, recipe);
    }
}

fn draw_recipe_row(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    recipe: RecipeId,
    rect: UiRect,
    selected: bool,
) {
    let Some(recipe_data) = ctx.content.recipes.get(recipe) else {
        return;
    };

    let enabled = can_craft_hand_recipe(&ctx.gameplay.inventory, recipe, ctx.content);
    let visual = item_visual(
        ctx.content,
        recipe_data.result_item,
        recipe_data.result_count,
    );

    draw_pill_surface(frame, rect, selected, ctx, layout.scale);

    let icon = UiRect::new(
        rect.x + 14.0 * layout.scale,
        rect.y + rect.height * 0.24,
        rect.height * 0.52,
        rect.height * 0.52,
    );

    frame.gradient_rect(
        UiLayer::Menu,
        icon,
        UiGradient::vertical(visual.color.lighten(0.18), visual.color.darken(0.18)),
        5.0 * layout.scale,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    draw_left_centered_text(
        frame,
        UiRect::new(
            icon.right() + 14.0 * layout.scale,
            rect.y,
            rect.width - icon.width - 32.0 * layout.scale,
            rect.height,
        ),
        visual.label,
        13.0 * layout.scale,
        if enabled {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_disabled
        },
    );
}

fn draw_recipe_detail(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    recipe: RecipeId,
) {
    let Some(recipe_data) = ctx.content.recipes.get(recipe) else {
        return;
    };

    let s = layout.scale;
    let detail = layout.recipe_detail;
    let visual = item_visual(
        ctx.content,
        recipe_data.result_item,
        recipe_data.result_count,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(detail.x, detail.y + 8.0 * s, detail.width, 34.0 * s),
        visual.label.to_uppercase(),
        19.0 * s,
        ctx.theme.accent,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(detail.x, detail.y + 40.0 * s, detail.width, 24.0 * s),
        "Utilitaire",
        12.0 * s,
        ctx.theme.text_muted,
    );

    let preview_size = (detail.width * 0.34).clamp(54.0 * s, 140.0 * s);
    let preview = UiRect::new(
        detail.right() - preview_size - 10.0 * s,
        detail.y + 76.0 * s,
        preview_size,
        preview_size,
    );

    frame.gradient_rect(
        UiLayer::Menu,
        preview,
        UiGradient::vertical(visual.color.lighten(0.28), visual.color.darken(0.18)),
        12.0 * s,
        UiBorder::new(1.0, UiColor::rgba(1.0, 1.0, 1.0, 0.14)),
        UiShadow::new(0.0, 16.0, 30.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.34)),
    );

    let ingredients_panel = UiRect::new(detail.x, detail.y + 220.0 * s, detail.width, 150.0 * s);

    frame.rounded_rect(
        UiLayer::Menu,
        ingredients_panel,
        ctx.theme.panel_subtle,
        8.0 * s,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::NONE,
    );

    for (i, ingredient) in ingredient_visuals(ctx.content, &ctx.gameplay.inventory, recipe)
        .iter()
        .take(4)
        .enumerate()
    {
        let y = ingredients_panel.y + 16.0 * s + i as f32 * 34.0 * s;

        frame.gradient_rect(
            UiLayer::Menu,
            UiRect::new(ingredients_panel.x + 16.0 * s, y, 22.0 * s, 22.0 * s),
            UiGradient::vertical(
                ingredient.color.lighten(0.16),
                ingredient.color.darken(0.16),
            ),
            4.0 * s,
            UiBorder::NONE,
            UiShadow::NONE,
        );

        draw_left_centered_text(
            frame,
            UiRect::new(ingredients_panel.x + 52.0 * s, y, 150.0 * s, 24.0 * s),
            &ingredient.label,
            12.0 * s,
            ctx.theme.text_primary,
        );

        let ok = ingredient.available >= ingredient.required;

        draw_right_centered_text(
            frame,
            UiRect::new(ingredients_panel.right() - 86.0 * s, y, 64.0 * s, 24.0 * s),
            format!("{} / {}", ingredient.available, ingredient.required),
            12.0 * s,
            if ok {
                ctx.theme.success
            } else {
                ctx.theme.danger
            },
        );
    }

    let quantity_y = ingredients_panel.bottom() + 24.0 * s;

    frame.text(
        UiLayer::Menu,
        UiRect::new(detail.x, quantity_y - 26.0 * s, 160.0 * s, 22.0 * s),
        "QUANTITÉ",
        13.0 * s,
        ctx.theme.accent,
    );

    let mut x = detail.x;

    for (i, label) in ["−", "1", "+", "Max"].iter().enumerate() {
        let w = if i == 3 { 78.0 * s } else { 48.0 * s };
        let rect = UiRect::new(x, quantity_y, w, 44.0 * s);

        draw_pill_button(frame, rect, label, false, ctx, s);

        x += w + 8.0 * s;
    }

    let craft = UiRect::new(detail.x, quantity_y + 66.0 * s, detail.width, 58.0 * s);

    let can_craft = can_craft_hand_recipe(&ctx.gameplay.inventory, recipe, ctx.content);

    draw_action_button(frame, craft, "FABRIQUER", can_craft, ctx, s);
}

fn draw_bottom_hotbar(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
) {
    for slot in &layout.hotbar_slots {
        let stack = ctx.gameplay.inventory.slots()[slot.index].stack;
        let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

        draw_item_slot(
            frame,
            slot.rect,
            slot.index == ctx.gameplay.selected_hotbar_slot,
            visual.as_ref(),
            ctx.gameplay.inventory_drag.source_slot == Some(slot.index),
            ctx.theme,
            UiLayer::Menu,
        );

        frame.text(
            UiLayer::Menu,
            UiRect::new(slot.rect.x + 6.0, slot.rect.y + 4.0, 22.0, 18.0),
            (slot.index + 1).to_string(),
            12.0 * layout.scale,
            ctx.theme.text_primary,
        );
    }
}

fn draw_footer_hint(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let s = layout.scale;
    let key_w = 70.0 * s;
    let label_w = 84.0 * s;
    let h = 38.0 * s;
    let gap = 8.0 * s;
    let total_w = key_w + label_w + gap;

    let x = layout.title_bar.right() - total_w;
    let y = ctx.screen_height - 64.0 * s;

    draw_pill_surface(frame, UiRect::new(x, y, key_w, h), false, ctx, s);
    draw_centered_text(
        frame,
        UiRect::new(x, y, key_w, h),
        "ÉCHAP",
        12.0 * s,
        ctx.theme.text_primary,
    );

    draw_pill_surface(
        frame,
        UiRect::new(x + key_w + gap, y, label_w, h),
        false,
        ctx,
        s,
    );
    draw_centered_text(
        frame,
        UiRect::new(x + key_w + gap, y, label_w, h),
        "Fermer",
        12.0 * s,
        ctx.theme.text_muted,
    );
}

fn draw_search_field(
    frame: &mut UiFrame,
    rect: UiRect,
    placeholder: &str,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    let radius = 8.0 * scale;
    let border = UiColor::rgba(0.58, 0.36, 0.16, 0.62);
    let fill = UiColor::rgba(0.010, 0.030, 0.036, 0.82);

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        border,
        radius,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    frame.rounded_rect(
        UiLayer::Menu,
        rect.inset(UiEdgeInsets::all(1.5 * scale)),
        fill,
        (radius - 1.5 * scale).max(4.0),
        UiBorder::NONE,
        UiShadow::NONE,
    );

    draw_left_centered_text(
        frame,
        rect.inset(UiEdgeInsets::symmetric(16.0 * scale, 0.0)),
        placeholder,
        15.0 * scale,
        ctx.theme.text_muted,
    );

    draw_centered_text(
        frame,
        UiRect::new(
            rect.right() - 44.0 * scale,
            rect.y,
            34.0 * scale,
            rect.height,
        ),
        "⌕",
        20.0 * scale,
        ctx.theme.accent,
    );
}

fn draw_square_button(
    frame: &mut UiFrame,
    rect: UiRect,
    label: &str,
    active: bool,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    draw_pill_surface(frame, rect, active, ctx, scale);
    draw_centered_text(
        frame,
        rect,
        label,
        if label == "×" {
            32.0 * scale
        } else {
            24.0 * scale
        },
        ctx.theme.accent,
    );
}

fn draw_pill_button(
    frame: &mut UiFrame,
    rect: UiRect,
    label: &str,
    active: bool,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    draw_pill_surface(frame, rect, active, ctx, scale);
    draw_centered_text(
        frame,
        rect,
        label,
        13.0 * scale,
        if active {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_muted
        },
    );
}

fn draw_action_button(
    frame: &mut UiFrame,
    rect: UiRect,
    label: &str,
    active: bool,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    let radius = 9.0 * scale;
    let border = if active {
        ctx.theme.border.with_alpha(0.95)
    } else {
        ctx.theme.border_soft.with_alpha(0.52)
    };

    let top = if active {
        UiColor::rgba(0.78, 0.50, 0.15, 0.94)
    } else {
        UiColor::rgba(0.020, 0.043, 0.050, 0.90)
    };

    let bottom = if active {
        UiColor::rgba(0.42, 0.25, 0.07, 0.96)
    } else {
        UiColor::rgba(0.006, 0.020, 0.025, 0.94)
    };

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        border,
        radius,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    frame.gradient_rect(
        UiLayer::Menu,
        rect.inset(UiEdgeInsets::all(1.5 * scale)),
        UiGradient::vertical(top, bottom),
        (radius - 1.5 * scale).max(4.0),
        UiBorder::NONE,
        UiShadow::NONE,
    );

    draw_centered_text(
        frame,
        rect,
        label,
        17.0 * scale,
        if active {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_disabled
        },
    );
}

fn draw_pill_surface(
    frame: &mut UiFrame,
    rect: UiRect,
    active: bool,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    let radius = (rect.height * 0.22).clamp(6.0 * scale, 11.0 * scale);
    let border = if active {
        ctx.theme.border.with_alpha(0.90)
    } else {
        UiColor::rgba(0.45, 0.28, 0.12, 0.66)
    };

    let top = if active {
        UiColor::rgba(0.73, 0.48, 0.16, 0.92)
    } else {
        UiColor::rgba(0.018, 0.043, 0.050, 0.88)
    };

    let bottom = if active {
        UiColor::rgba(0.42, 0.26, 0.08, 0.94)
    } else {
        UiColor::rgba(0.006, 0.020, 0.024, 0.90)
    };

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        border,
        radius,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    frame.gradient_rect(
        UiLayer::Menu,
        rect.inset(UiEdgeInsets::all(1.4 * scale)),
        UiGradient::vertical(top, bottom),
        (radius - 1.4 * scale).max(4.0),
        UiBorder::NONE,
        UiShadow::NONE,
    );
}

fn draw_centered_text(
    frame: &mut UiFrame,
    rect: UiRect,
    text: impl Into<String>,
    size: f32,
    color: UiColor,
) {
    let text_rect = vertically_centered_text_rect(rect, size);

    frame.text_aligned(
        UiLayer::Menu,
        text_rect,
        text,
        size,
        color,
        UiTextAlign::Center,
    );
}

fn draw_left_centered_text(
    frame: &mut UiFrame,
    rect: UiRect,
    text: impl Into<String>,
    size: f32,
    color: UiColor,
) {
    let text_rect = vertically_centered_text_rect(rect, size);

    frame.text_aligned(
        UiLayer::Menu,
        text_rect,
        text,
        size,
        color,
        UiTextAlign::Left,
    );
}

fn draw_right_centered_text(
    frame: &mut UiFrame,
    rect: UiRect,
    text: impl Into<String>,
    size: f32,
    color: UiColor,
) {
    let text_rect = vertically_centered_text_rect(rect, size);

    frame.text_aligned(
        UiLayer::Menu,
        text_rect,
        text,
        size,
        color,
        UiTextAlign::Right,
    );
}

fn vertically_centered_text_rect(rect: UiRect, size: f32) -> UiRect {
    // glyphon places the visible glyph mass slightly above the theoretical text box.
    // This compensates visually so labels sit in the optical center of buttons.
    let line_h = (size * 1.18).max(size + 2.0);
    let optical_y = rect.y + (rect.height - line_h) * 0.5 + size * 0.20;

    UiRect::new(rect.x, optical_y, rect.width, line_h)
}

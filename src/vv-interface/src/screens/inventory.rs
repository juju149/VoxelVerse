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

        draw_title(frame, ctx);
        draw_panel(frame, layout.equipment_panel, ctx);
        draw_panel(frame, layout.backpack_panel, ctx);
        draw_panel(frame, layout.crafting_panel, ctx);

        draw_equipment_panel(frame, ctx, &layout);
        draw_backpack_panel(frame, ctx, &layout);
        draw_crafting_panel(frame, ctx, &layout);
        draw_bottom_hotbar(frame, ctx, &layout);
        draw_footer_hint(frame, ctx);
    }
}

fn draw_title(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
    let s = (ctx.screen_height.min(ctx.screen_width) / 1080.0).clamp(0.72, 1.10);
    let x = 32.0 * s;
    let y = 26.0 * s;

    frame.rounded_rect(
        UiLayer::Menu,
        UiRect::new(x, y, 70.0 * s, 70.0 * s),
        ctx.theme.panel_subtle,
        10.0 * s,
        UiBorder::new(1.0, ctx.theme.border),
        UiShadow::NONE,
    );

    frame.text_aligned(
        UiLayer::Menu,
        UiRect::new(x, y + 6.0 * s, 70.0 * s, 32.0 * s),
        "▣",
        24.0 * s,
        ctx.theme.accent,
        UiTextAlign::Center,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(x + 88.0 * s, y + 2.0 * s, 420.0 * s, 42.0 * s),
        "INVENTAIRE",
        34.0 * s,
        ctx.theme.text_primary,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(x + 88.0 * s, y + 44.0 * s, 520.0 * s, 26.0 * s),
        "Sac, équipement et ressources",
        16.0 * s,
        ctx.theme.text_muted,
    );

    let close_size = 58.0 * s;
    frame.rounded_rect(
        UiLayer::Menu,
        UiRect::new(ctx.screen_width - x - close_size, y, close_size, close_size),
        ctx.theme.panel_subtle,
        10.0 * s,
        UiBorder::new(1.0, ctx.theme.border),
        UiShadow::NONE,
    );

    frame.text_aligned(
        UiLayer::Menu,
        UiRect::new(
            ctx.screen_width - x - close_size,
            y + 2.0 * s,
            close_size,
            close_size,
        ),
        "×",
        34.0 * s,
        ctx.theme.accent,
        UiTextAlign::Center,
    );
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

    let avatar = UiRect::new(
        rect.x + rect.width * 0.30,
        rect.y + 88.0 * s,
        rect.width * 0.40,
        rect.height * 0.47,
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

    frame.text_aligned(
        UiLayer::Menu,
        avatar,
        "PLAYER",
        22.0 * s,
        ctx.theme.text_primary,
        UiTextAlign::Center,
    );

    let small = (66.0 * s).clamp(42.0, 70.0);
    let left_x = rect.x + pad;
    let right_x = rect.right() - pad - small;
    let y0 = rect.y + 58.0 * s;
    let step = small + 22.0 * s;

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
        UiRect::new(rect.x, rect.bottom() - 122.0 * s, rect.width, 24.0 * s),
        "OUTILS RAPIDES",
        17.0 * s,
        ctx.theme.accent,
        UiTextAlign::Center,
    );

    for i in 0..3 {
        let tool_slot = UiRect::new(
            rect.x + rect.width * 0.20 + i as f32 * (small + 40.0 * s),
            rect.bottom() - 82.0 * s,
            small,
            small,
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
            rect.y - 20.0 * scale,
            rect.width + 16.0 * scale,
            18.0 * scale,
        ),
        label,
        11.0 * scale,
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
        19.0 * s,
        ctx.theme.accent,
    );

    let search = UiRect::new(
        rect.x + pad,
        rect.y + 58.0 * s,
        rect.width - pad * 2.0,
        44.0 * s,
    );

    frame.rounded_rect(
        UiLayer::Menu,
        search,
        ctx.theme.panel_subtle,
        7.0 * s,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::NONE,
    );

    frame.text(
        UiLayer::Menu,
        search.inset(UiEdgeInsets::symmetric(14.0 * s, 0.0)),
        "Rechercher un objet...",
        16.0 * s,
        ctx.theme.text_muted,
    );

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

    for (i, label) in ["Tout", "Ressources", "Outils", "Nourriture", "Divers"]
        .iter()
        .enumerate()
    {
        let w = match i {
            0 => 84.0 * s,
            1 => 134.0 * s,
            2 => 104.0 * s,
            3 => 136.0 * s,
            _ => 100.0 * s,
        };

        frame.rounded_rect(
            UiLayer::Menu,
            UiRect::new(x, tab_y, w, tab_h),
            if i == 0 {
                ctx.theme.panel_active
            } else {
                ctx.theme.panel_subtle
            },
            8.0 * s,
            UiBorder::new(1.0, ctx.theme.border_soft),
            UiShadow::NONE,
        );

        frame.text_aligned(
            UiLayer::Menu,
            UiRect::new(x, tab_y, w, tab_h),
            *label,
            13.0 * s,
            if i == 0 {
                ctx.theme.text_primary
            } else {
                ctx.theme.text_muted
            },
            UiTextAlign::Center,
        );

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
        17.0 * s,
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
    frame.rounded_rect(
        UiLayer::Menu,
        sort,
        ctx.theme.panel_subtle,
        8.0 * s,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::NONE,
    );
    frame.text_aligned(
        UiLayer::Menu,
        sort,
        "↕ Trier",
        14.0 * s,
        ctx.theme.text_primary,
        UiTextAlign::Center,
    );
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
        20.0 * s,
        ctx.theme.text_primary,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + pad, rect.y + 48.0 * s, 360.0 * s, 24.0 * s),
        "Fabrication à la main uniquement",
        13.0 * s,
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

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        if selected {
            ctx.theme.panel_active
        } else {
            ctx.theme.panel_subtle
        },
        8.0 * layout.scale,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::NONE,
    );

    let icon = UiRect::new(
        rect.x + 16.0 * layout.scale,
        rect.y + rect.height * 0.23,
        rect.height * 0.54,
        rect.height * 0.54,
    );

    frame.gradient_rect(
        UiLayer::Menu,
        icon,
        UiGradient::vertical(visual.color.lighten(0.18), visual.color.darken(0.18)),
        5.0 * layout.scale,
        UiBorder::NONE,
        UiShadow::NONE,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(
            icon.right() + 16.0 * layout.scale,
            rect.y + rect.height * 0.30,
            rect.width - icon.width - 34.0 * layout.scale,
            rect.height * 0.40,
        ),
        visual.label,
        15.0 * layout.scale,
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
        21.0 * s,
        ctx.theme.accent,
    );

    frame.text(
        UiLayer::Menu,
        UiRect::new(detail.x, detail.y + 42.0 * s, detail.width, 24.0 * s),
        "Utilitaire",
        13.0 * s,
        ctx.theme.text_muted,
    );

    let preview = UiRect::new(
        detail.x + detail.width * 0.55,
        detail.y + 76.0 * s,
        detail.width * 0.34,
        detail.width * 0.34,
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
        let y = ingredients_panel.y + 16.0 * s + i as f32 * 38.0 * s;

        frame.gradient_rect(
            UiLayer::Menu,
            UiRect::new(ingredients_panel.x + 16.0 * s, y, 24.0 * s, 24.0 * s),
            UiGradient::vertical(
                ingredient.color.lighten(0.16),
                ingredient.color.darken(0.16),
            ),
            4.0 * s,
            UiBorder::NONE,
            UiShadow::NONE,
        );

        frame.text(
            UiLayer::Menu,
            UiRect::new(ingredients_panel.x + 54.0 * s, y, 160.0 * s, 24.0 * s),
            &ingredient.label,
            13.0 * s,
            ctx.theme.text_primary,
        );

        let ok = ingredient.available >= ingredient.required;
        frame.text_aligned(
            UiLayer::Menu,
            UiRect::new(ingredients_panel.right() - 86.0 * s, y, 64.0 * s, 24.0 * s),
            format!("{} / {}", ingredient.available, ingredient.required),
            13.0 * s,
            if ok {
                ctx.theme.success
            } else {
                ctx.theme.danger
            },
            UiTextAlign::Right,
        );
    }

    let quantity = UiRect::new(
        detail.x,
        ingredients_panel.bottom() + 28.0 * s,
        detail.width,
        48.0 * s,
    );
    frame.text(
        UiLayer::Menu,
        UiRect::new(quantity.x, quantity.y - 28.0 * s, 160.0 * s, 24.0 * s),
        "QUANTITÉ",
        14.0 * s,
        ctx.theme.accent,
    );

    for (i, label) in ["−", "1", "+", "Max"].iter().enumerate() {
        let w = if i == 3 { 82.0 * s } else { 50.0 * s };
        let x = quantity.x + i as f32 * 58.0 * s + if i == 3 { 18.0 * s } else { 0.0 };

        frame.rounded_rect(
            UiLayer::Menu,
            UiRect::new(x, quantity.y, w, quantity.height),
            ctx.theme.panel_subtle,
            7.0 * s,
            UiBorder::new(1.0, ctx.theme.border_soft),
            UiShadow::NONE,
        );

        frame.text_aligned(
            UiLayer::Menu,
            UiRect::new(x, quantity.y, w, quantity.height),
            *label,
            15.0 * s,
            ctx.theme.text_primary,
            UiTextAlign::Center,
        );
    }

    let craft = UiRect::new(
        detail.x,
        quantity.bottom() + 26.0 * s,
        detail.width,
        64.0 * s,
    );
    let can_craft = can_craft_hand_recipe(&ctx.gameplay.inventory, recipe, ctx.content);

    frame.rounded_rect(
        UiLayer::Menu,
        craft,
        if can_craft {
            ctx.theme.panel_active
        } else {
            ctx.theme.panel_subtle
        },
        9.0 * s,
        UiBorder::new(1.0, ctx.theme.border),
        UiShadow::new(
            0.0,
            0.0,
            18.0,
            0.0,
            ctx.theme
                .accent
                .multiply_alpha(if can_craft { 0.25 } else { 0.0 }),
        ),
    );

    frame.text_aligned(
        UiLayer::Menu,
        craft,
        "FABRIQUER",
        19.0 * s,
        if can_craft {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_disabled
        },
        UiTextAlign::Center,
    );
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
            13.0,
            ctx.theme.text_primary,
        );
    }
}

fn draw_footer_hint(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
    let s = (ctx.screen_width.min(ctx.screen_height) / 1080.0).clamp(0.72, 1.10);
    let rect = UiRect::new(
        ctx.screen_width - 208.0 * s,
        ctx.screen_height - 78.0 * s,
        180.0 * s,
        44.0 * s,
    );

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        ctx.theme.panel_subtle,
        8.0 * s,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::NONE,
    );

    frame.text_aligned(
        UiLayer::Menu,
        rect,
        "ÉCHAP    Fermer",
        13.0 * s,
        ctx.theme.text_primary,
        UiTextAlign::Center,
    );
}

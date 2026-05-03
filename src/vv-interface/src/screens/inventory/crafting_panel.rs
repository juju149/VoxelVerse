use vv_gameplay::can_craft_hand_recipe;
use vv_registry::RecipeId;
use vv_ui::{
    UiBorder, UiButton, UiColor, UiFrame, UiGradient, UiInput, UiLayer, UiPanel, UiRect, UiShadow,
    UiStyle, UiSurface, UiWidgetId,
};

use crate::{ingredient_visuals, item_visual, GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let rect = layout.crafting_panel;
    let s = layout.scale;
    let pad = 24.0 * s;
    let styles = UiStyle::from_theme(ctx.theme);
    let input = UiInput::default();

    UiPanel::new(rect, styles.glass_panel).draw(frame);

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

    for recipe_slot in &layout.recipe_slots {
        draw_recipe_row(
            frame,
            ctx,
            layout,
            &styles,
            &input,
            recipe_slot.recipe,
            recipe_slot.rect,
            Some(recipe_slot.recipe) == selected_recipe,
        );
    }

    if let Some(recipe) = selected_recipe {
        draw_recipe_detail(frame, ctx, layout, &styles, &input, recipe);
    }
}

fn draw_recipe_row(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    styles: &UiStyle,
    input: &UiInput,
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

    UiButton::new(
        UiWidgetId::new(3_000 + recipe.raw() as u64),
        rect,
        "",
        if selected {
            styles.primary_button
        } else {
            styles.button
        },
    )
    .disabled(!enabled)
    .draw(frame, input, None);

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

    frame.text_left_centered(
        UiLayer::Menu,
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
    styles: &UiStyle,
    input: &UiInput,
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

    UiSurface::new(ctx.theme.panel_subtle)
        .border(ctx.theme.border_soft, 1.0)
        .radius(8.0 * s)
        .draw(frame, UiLayer::Menu, ingredients_panel);

    draw_ingredients(frame, ctx, recipe, ingredients_panel, s);

    let quantity_y = ingredients_panel.bottom() + 24.0 * s;

    frame.text(
        UiLayer::Menu,
        UiRect::new(detail.x, quantity_y - 26.0 * s, 160.0 * s, 22.0 * s),
        "QUANTITÉ",
        13.0 * s,
        ctx.theme.accent,
    );

    draw_quantity_stepper(frame, styles, input, detail.x, quantity_y, s);

    let craft = UiRect::new(detail.x, quantity_y + 66.0 * s, detail.width, 58.0 * s);
    let can_craft = can_craft_hand_recipe(&ctx.gameplay.inventory, recipe, ctx.content);

    UiButton::new(
        UiWidgetId::new(3_900),
        craft,
        "FABRIQUER",
        if can_craft {
            styles.primary_button
        } else {
            styles.button
        },
    )
    .disabled(!can_craft)
    .text_size(17.0 * s)
    .draw(frame, input, None);
}

fn draw_ingredients(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    recipe: RecipeId,
    rect: UiRect,
    scale: f32,
) {
    for (i, ingredient) in ingredient_visuals(ctx.content, &ctx.gameplay.inventory, recipe)
        .iter()
        .take(4)
        .enumerate()
    {
        let y = rect.y + 16.0 * scale + i as f32 * 34.0 * scale;

        frame.gradient_rect(
            UiLayer::Menu,
            UiRect::new(rect.x + 16.0 * scale, y, 22.0 * scale, 22.0 * scale),
            UiGradient::vertical(
                ingredient.color.lighten(0.16),
                ingredient.color.darken(0.16),
            ),
            4.0 * scale,
            UiBorder::NONE,
            UiShadow::NONE,
        );

        frame.text_left_centered(
            UiLayer::Menu,
            UiRect::new(rect.x + 52.0 * scale, y, 150.0 * scale, 24.0 * scale),
            &ingredient.label,
            12.0 * scale,
            ctx.theme.text_primary,
        );

        let ok = ingredient.available >= ingredient.required;

        frame.text_right_centered(
            UiLayer::Menu,
            UiRect::new(rect.right() - 86.0 * scale, y, 64.0 * scale, 24.0 * scale),
            format!("{} / {}", ingredient.available, ingredient.required),
            12.0 * scale,
            if ok {
                ctx.theme.success
            } else {
                ctx.theme.danger
            },
        );
    }
}

fn draw_quantity_stepper(
    frame: &mut UiFrame,
    styles: &UiStyle,
    input: &UiInput,
    start_x: f32,
    y: f32,
    scale: f32,
) {
    let mut x = start_x;

    for (i, label) in ["−", "1", "+", "Max"].iter().enumerate() {
        let w = if i == 3 { 78.0 * scale } else { 48.0 * scale };
        let rect = UiRect::new(x, y, w, 44.0 * scale);

        UiButton::new(
            UiWidgetId::new(3_700 + i as u64),
            rect,
            *label,
            styles.button,
        )
        .text_size(14.0 * scale)
        .draw(frame, input, None);

        x += w + 8.0 * scale;
    }
}

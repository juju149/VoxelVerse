use vv_ui::{UiBorder, UiFrame, UiLayer, UiRect, UiShadow, UiTextAlign};

use crate::{
    components::{button, panel, search_field, slot},
    item_visual, GameplayUiContext, InventoryUiLayout,
};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let rect = layout.backpack_panel;
    let s = layout.scale;
    let pad = 24.0 * s;

    panel::glass(frame, rect, ctx);

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + pad, rect.y + 20.0 * s, 260.0 * s, 30.0 * s),
        "SAC À DOS",
        18.0 * s,
        ctx.theme.accent,
    );

    search_field::draw(
        frame,
        UiRect::new(
            rect.x + pad,
            rect.y + 58.0 * s,
            rect.width - pad * 2.0,
            44.0 * s,
        ),
        "Rechercher un objet...",
        ctx,
        s,
    );

    draw_category_tabs(frame, ctx, layout);
    draw_inventory_grid(frame, ctx, layout);
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

        button::pill(
            frame,
            UiRect::new(x, tab_y, w, tab_h),
            label,
            if i == 0 {
                button::VvButtonVariant::Active
            } else {
                button::VvButtonVariant::Default
            },
            ctx,
            s,
        );

        x += w + 8.0 * s;
    }
}

fn draw_inventory_grid(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
) {
    for inventory_slot in &layout.inventory_slots {
        if inventory_slot.index < ctx.gameplay.inventory.hotbar_len() {
            continue;
        }

        let stack = ctx.gameplay.inventory.slots()[inventory_slot.index].stack;
        let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));

        slot::item_slot(
            frame,
            inventory_slot.rect,
            false,
            visual.as_ref(),
            ctx.gameplay.inventory_drag.source_slot == Some(inventory_slot.index),
            ctx.theme,
            UiLayer::Menu,
        );
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

    button::pill(
        frame,
        UiRect::new(rect.right() - 138.0 * s, y - 2.0 * s, 114.0 * s, 48.0 * s),
        "↕ Trier",
        button::VvButtonVariant::Default,
        ctx,
        s,
    );
}

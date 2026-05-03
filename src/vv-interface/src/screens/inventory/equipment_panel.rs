use vv_ui::{
    UiBorder, UiColor, UiFrame, UiGradient, UiInput, UiLayer, UiPanel, UiRect, UiShadow, UiSlot,
    UiSlotContent, UiStyle, UiWidgetId,
};

use crate::{item_visual, GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let rect = layout.equipment_panel;
    let s = layout.scale;
    let pad = 24.0 * s;
    let styles = UiStyle::from_theme(ctx.theme);
    let input = UiInput::default();

    UiPanel::new(rect, styles.glass_panel).draw(frame);

    frame.text_centered(
        UiLayer::Menu,
        UiRect::new(rect.x, rect.y + 18.0 * s, rect.width, 28.0 * s),
        "ÉQUIPEMENT",
        18.0 * s,
        ctx.theme.accent,
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

    frame.text_centered(
        UiLayer::Menu,
        avatar,
        "PLAYER",
        20.0 * s,
        ctx.theme.text_primary,
    );

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
        let slot_rect = UiRect::new(left_x, y0 + i as f32 * step, small, small);
        draw_equipment_slot(
            frame,
            &input,
            &styles,
            slot_rect,
            label,
            ctx,
            s,
            1_100 + i as u64,
        );
    }

    for (i, label) in ["Amulette", "Sac", "Cape", "Torche"].iter().enumerate() {
        let slot_rect = UiRect::new(right_x, y0 + i as f32 * step, small, small);
        draw_equipment_slot(
            frame,
            &input,
            &styles,
            slot_rect,
            label,
            ctx,
            s,
            1_200 + i as u64,
        );
    }

    frame.text_centered(
        UiLayer::Menu,
        UiRect::new(rect.x, rect.bottom() - 128.0 * s, rect.width, 24.0 * s),
        "OUTILS RAPIDES",
        16.0 * s,
        ctx.theme.accent,
    );

    draw_quick_tools(frame, ctx, layout, &styles, &input);
}

fn draw_equipment_slot(
    frame: &mut UiFrame,
    input: &UiInput,
    styles: &UiStyle,
    rect: UiRect,
    label: &str,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
    id: u64,
) {
    frame.text_centered(
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
    );

    UiSlot::new(UiWidgetId::new(id), rect, styles.slot).draw(frame, input, None);
}

fn draw_quick_tools(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    styles: &UiStyle,
    input: &UiInput,
) {
    let rect = layout.equipment_panel;
    let s = layout.scale;
    let quick_slot = (58.0 * s).clamp(38.0, 68.0);
    let quick_gap = 28.0 * s;
    let total_quick_w = quick_slot * 3.0 + quick_gap * 2.0;
    let quick_x = rect.x + (rect.width - total_quick_w) * 0.5;
    let quick_y = rect.bottom() - 88.0 * s;

    for i in 0..3 {
        let slot_rect = UiRect::new(
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
        let hidden = ctx.gameplay.inventory_drag.source_slot == Some(i);

        let mut slot = UiSlot::new(UiWidgetId::new(1_300 + i as u64), slot_rect, styles.slot);

        if let Some(visual) = visual.as_ref().filter(|_| !hidden) {
            slot = slot
                .content(UiSlotContent::Color(visual.color))
                .count(Some(visual.count));
        }

        slot.draw(frame, input, None);
    }
}

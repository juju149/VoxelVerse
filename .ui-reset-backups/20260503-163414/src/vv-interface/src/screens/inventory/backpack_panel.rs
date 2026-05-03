use vv_ui::{
    UiButton, UiFrame, UiInput, UiLayer, UiPanel, UiProgressBar, UiRect, UiSearchField, UiSlot,
    UiSlotContent, UiStyle, UiTab, UiTabs, UiWidgetId,
};

use crate::{item_visual, GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let rect = layout.backpack_panel;
    let s = layout.scale;
    let pad = 44.0 * s;
    let styles = UiStyle::from_theme(ctx.theme);
    let input = UiInput::default();

    UiPanel::new(rect, styles.glass_panel).draw(frame);

    frame.text(
        UiLayer::Menu,
        UiRect::new(
            rect.x + pad,
            rect.y + 28.0 * s,
            rect.width - pad * 2.0,
            36.0 * s,
        ),
        "SAC À DOS",
        24.0 * s,
        ctx.theme.accent,
    );

    draw_search_and_sort(frame, &input, &styles, rect, pad, s);
    draw_category_tabs(frame, &input, &styles, layout, pad, s);
    draw_inventory_grid(frame, ctx, layout, &styles, &input);
    draw_footer(frame, ctx, layout, &styles, &input, pad, s);
}

fn draw_search_and_sort(
    frame: &mut UiFrame,
    input: &UiInput,
    styles: &UiStyle,
    rect: UiRect,
    pad: f32,
    scale: f32,
) {
    let y = rect.y + 92.0 * scale;
    let height = 56.0 * scale;
    let gap = 24.0 * scale;
    let sort_w = 170.0 * scale;

    let search = UiRect::new(
        rect.x + pad,
        y,
        rect.width - pad * 2.0 - sort_w - gap,
        height,
    );

    UiSearchField::new(
        UiWidgetId::new(2_000),
        search,
        "",
        "Rechercher un objet...",
        styles.search,
    )
    .draw(frame, input, false);

    UiButton::new(
        UiWidgetId::new(2_001),
        UiRect::new(search.right() + gap, y, sort_w, height),
        "Trier",
        styles.button,
    )
    .text_size(17.0 * scale)
    .draw(frame, input, None);
}

fn draw_category_tabs(
    frame: &mut UiFrame,
    input: &UiInput,
    styles: &UiStyle,
    layout: &InventoryUiLayout,
    pad: f32,
    scale: f32,
) {
    let tabs = vec![
        UiTab::new(UiWidgetId::new(2_100), "Tout"),
        UiTab::new(UiWidgetId::new(2_101), "Ressources"),
        UiTab::new(UiWidgetId::new(2_102), "Outils"),
        UiTab::new(UiWidgetId::new(2_103), "Nourriture"),
        UiTab::new(UiWidgetId::new(2_104), "Matériaux"),
        UiTab::new(UiWidgetId::new(2_105), "Divers"),
    ];

    UiTabs::new(
        UiRect::new(
            layout.backpack_panel.x + pad,
            layout.backpack_panel.y + 174.0 * scale,
            layout.backpack_panel.width - pad * 2.0,
            52.0 * scale,
        ),
        tabs,
        UiWidgetId::new(2_100),
        styles.tabs,
    )
    .draw(frame, input);
}

fn draw_inventory_grid(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    styles: &UiStyle,
    input: &UiInput,
) {
    let main_start = ctx.gameplay.inventory.main_start();
    let slot_count = ctx.gameplay.inventory.slot_count();

    for (cell_index, cell_rect) in layout.backpack_cells.iter().copied().enumerate() {
        let slot_index = main_start + cell_index;
        let hidden = ctx.gameplay.inventory_drag.source_slot == Some(slot_index);

        let visual = if slot_index < slot_count && !hidden {
            ctx.gameplay.inventory.slots()[slot_index]
                .stack
                .map(|stack| item_visual(ctx.content, stack.item, stack.count))
        } else {
            None
        };

        let mut slot = UiSlot::new(
            UiWidgetId::new(2_500 + cell_index as u64),
            cell_rect,
            styles.slot,
        );

        if let Some(visual) = visual {
            slot = slot
                .content(UiSlotContent::Color(visual.color))
                .count(Some(visual.count));
        }

        slot.draw(frame, input, None);
    }
}

fn draw_footer(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    styles: &UiStyle,
    input: &UiInput,
    pad: f32,
    scale: f32,
) {
    let rect = layout.backpack_panel;
    let divider_y = rect.bottom() - 164.0 * scale;
    let footer_y = rect.bottom() - 118.0 * scale;

    frame.rect(
        UiLayer::Menu,
        UiRect::new(
            rect.x + pad,
            divider_y,
            rect.width - pad * 2.0,
            1.0_f32.max(scale),
        ),
        ctx.theme.border_soft.with_alpha(0.42),
    );

    let left_w = 320.0 * scale;
    let progress_w = 300.0 * scale;

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + pad + 42.0 * scale, footer_y, left_w, 30.0 * scale),
        "46,2 / 120 kg",
        20.0 * scale,
        ctx.theme.text_primary,
    );

    UiProgressBar::new(
        UiRect::new(
            rect.x + pad,
            footer_y + 48.0 * scale,
            progress_w,
            12.0 * scale,
        ),
        0.385,
        styles.progress,
    )
    .draw(frame);

    let button_h = 46.0 * scale;
    let left_block_right = rect.x + pad + 360.0 * scale;
    let available_right = rect.right() - pad - left_block_right;
    let preferred_total = 230.0 + 112.0 + 170.0 + 122.0;
    let button_scale = (available_right / (preferred_total * scale)).clamp(0.72, 1.0);

    let buttons = [
        ("Maj + Clic : transférer", 230.0),
        ("Trier", 112.0),
        ("Tout prendre", 170.0),
        ("Diviser", 122.0),
    ];

    let total_w = buttons
        .iter()
        .map(|(_, w)| *w * scale * button_scale)
        .sum::<f32>();

    let mut x = (rect.right() - pad - total_w).max(left_block_right);
    let y = footer_y + 2.0 * scale;

    for (i, (label, w)) in buttons.iter().enumerate() {
        let width = *w * scale * button_scale;

        UiButton::new(
            UiWidgetId::new(2_900 + i as u64),
            UiRect::new(x, y, width, button_h),
            *label,
            styles.button,
        )
        .text_size(if i == 0 { 13.0 * scale } else { 13.0 * scale })
        .draw(frame, input, None);

        x += width;
    }
}

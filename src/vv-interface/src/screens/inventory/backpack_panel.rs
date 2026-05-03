use vv_ui::{
    UiButton, UiFrame, UiInput, UiLayer, UiProgressBar, UiRect, UiSearchField, UiSlot,
    UiSlotContent, UiStyle, UiTab, UiTabs, UiWidgetId,
};

use crate::{item_visual, GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let rect = layout.backpack_panel;
    let s = layout.scale;
    let pad = 24.0 * s;
    let styles = UiStyle::from_theme(ctx.theme);
    let input = UiInput::default();

    vv_ui::UiPanel::new(rect, styles.glass_panel).draw(frame);

    frame.text(
        UiLayer::Menu,
        UiRect::new(rect.x + pad, rect.y + 20.0 * s, 260.0 * s, 30.0 * s),
        "SAC À DOS",
        18.0 * s,
        ctx.theme.accent,
    );

    let search_rect = UiRect::new(
        rect.x + pad,
        rect.y + 58.0 * s,
        rect.width - pad * 2.0 - 138.0 * s,
        44.0 * s,
    );

    UiSearchField::new(
        UiWidgetId::new(2_000),
        search_rect,
        "",
        "Rechercher un objet...",
        styles.search,
    )
    .draw(frame, &input, false);

    UiButton::new(
        UiWidgetId::new(2_001),
        UiRect::new(
            rect.right() - 138.0 * s - pad,
            rect.y + 58.0 * s,
            138.0 * s,
            44.0 * s,
        ),
        "Trier",
        styles.button,
    )
    .text_size(13.0 * s)
    .draw(frame, &input, None);

    draw_category_tabs(frame, &input, &styles, layout, s);
    draw_inventory_grid(frame, ctx, layout, &styles, &input);
    draw_weight(frame, ctx, layout, &styles);
}

fn draw_category_tabs(
    frame: &mut UiFrame,
    input: &UiInput,
    styles: &UiStyle,
    layout: &InventoryUiLayout,
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
            layout.backpack_panel.x + 24.0 * scale,
            layout.backpack_panel.y + 114.0 * scale,
            layout.backpack_panel.width - 48.0 * scale,
            42.0 * scale,
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
    for inventory_slot in &layout.inventory_slots {
        if inventory_slot.index < ctx.gameplay.inventory.hotbar_len() {
            continue;
        }

        let stack = ctx.gameplay.inventory.slots()[inventory_slot.index].stack;
        let visual = stack.map(|stack| item_visual(ctx.content, stack.item, stack.count));
        let hidden = ctx.gameplay.inventory_drag.source_slot == Some(inventory_slot.index);

        let mut slot = UiSlot::new(
            UiWidgetId::new(2_500 + inventory_slot.index as u64),
            inventory_slot.rect,
            styles.slot,
        );

        if let Some(visual) = visual.as_ref().filter(|_| !hidden) {
            slot = slot
                .content(UiSlotContent::Color(visual.color))
                .count(Some(visual.count));
        }

        slot.draw(frame, input, None);
    }
}

fn draw_weight(
    frame: &mut UiFrame,
    ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    styles: &UiStyle,
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

    UiProgressBar::new(
        UiRect::new(rect.x + 24.0 * s, y + 34.0 * s, 180.0 * s, 10.0 * s),
        0.385,
        styles.progress,
    )
    .draw(frame);
}

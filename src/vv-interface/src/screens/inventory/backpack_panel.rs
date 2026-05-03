use vv_ui::{UiFrame, UiLayer, UiRect};

use crate::{design::InventoryUiTokens, GameplayUiContext, InventoryUiLayout};

const BACKPACK_TABS: [&str; 5] = ["Tout", "Ressources", "Outils", "Nourriture", "Divers"];

pub fn draw(
    frame: &mut UiFrame,
    _ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    tokens: InventoryUiTokens,
) {
    tokens
        .panel_surface()
        .draw(frame, UiLayer::Menu, layout.backpack_panel);

    draw_title(frame, layout, tokens);
    draw_search(frame, layout, tokens);
    draw_sort_button(frame, layout, tokens);
    draw_tabs(frame, layout, tokens);
    draw_empty_grid(frame, layout, tokens);
}

fn draw_title(frame: &mut UiFrame, layout: &InventoryUiLayout, tokens: InventoryUiTokens) {
    frame.text_left_centered(
        UiLayer::Menu,
        layout.backpack_title,
        "SAC À DOS",
        tokens.text.panel_title * layout.scale,
        tokens.colors.title,
    );
}

fn draw_search(frame: &mut UiFrame, layout: &InventoryUiLayout, tokens: InventoryUiTokens) {
    tokens
        .input_surface()
        .draw(frame, UiLayer::Menu, layout.backpack_search);

    let text_pad = 18.0 * layout.scale;

    frame.text_left_centered(
        UiLayer::Menu,
        UiRect::new(
            layout.backpack_search.x + text_pad,
            layout.backpack_search.y,
            (layout.backpack_search.width - text_pad * 2.0).max(0.0),
            layout.backpack_search.height,
        ),
        "Rechercher un objet...",
        tokens.text.body * layout.scale,
        tokens.colors.text_secondary,
    );
}

fn draw_sort_button(frame: &mut UiFrame, layout: &InventoryUiLayout, tokens: InventoryUiTokens) {
    tokens
        .button_surface()
        .draw(frame, UiLayer::Menu, layout.backpack_sort);

    frame.text_centered(
        UiLayer::Menu,
        layout.backpack_sort,
        "Trier",
        tokens.text.button * layout.scale,
        tokens.colors.text_primary,
    );
}

fn draw_tabs(frame: &mut UiFrame, layout: &InventoryUiLayout, tokens: InventoryUiTokens) {
    let gap = tokens.layout.tab_gap * layout.scale;
    let tab_count = BACKPACK_TABS.len();

    if tab_count == 0 || layout.backpack_tabs.width <= 0.0 {
        return;
    }

    let total_gap = gap * tab_count.saturating_sub(1) as f32;
    let tab_w = ((layout.backpack_tabs.width - total_gap) / tab_count as f32).max(0.0);

    for (index, label) in BACKPACK_TABS.iter().enumerate() {
        let active = index == 0;
        let tab = UiRect::new(
            layout.backpack_tabs.x + index as f32 * (tab_w + gap),
            layout.backpack_tabs.y,
            tab_w,
            layout.backpack_tabs.height,
        );

        tokens.tab_surface(active).draw(frame, UiLayer::Menu, tab);

        frame.text_centered(
            UiLayer::Menu,
            tab,
            *label,
            tokens.text.tab * layout.scale,
            if active {
                tokens.colors.text_active
            } else {
                tokens.colors.text_primary
            },
        );
    }
}

fn draw_empty_grid(frame: &mut UiFrame, layout: &InventoryUiLayout, tokens: InventoryUiTokens) {
    for rect in &layout.backpack_cells {
        tokens.slot_surface().draw(frame, UiLayer::Menu, *rect);
    }
}

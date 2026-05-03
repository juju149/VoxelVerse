use vv_ui::{UiFrame, UiLayer, UiRect};

use crate::{design::InventoryUiTokens, GameplayUiContext, InventoryUiLayout};

pub fn draw(
    frame: &mut UiFrame,
    _ctx: &GameplayUiContext<'_>,
    layout: &InventoryUiLayout,
    tokens: InventoryUiTokens,
) {
    let rect = layout.backpack_panel;
    let s = layout.scale;

    tokens.panel_surface().draw(frame, UiLayer::Menu, rect);

    frame.text_left_centered(
        UiLayer::Menu,
        layout.backpack_title,
        "SAC À DOS",
        tokens.text.panel_title * s,
        tokens.colors.title,
    );

    draw_search(frame, layout, tokens);
    draw_sort_button(frame, layout, tokens);
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
            layout.backpack_search.width - text_pad * 2.0,
            layout.backpack_search.height,
        ),
        "Rechercher un objet...",
        tokens.text.body * layout.scale,
        tokens.colors.text_secondary,
    );
}

fn draw_sort_button(frame: &mut UiFrame, layout: &InventoryUiLayout, tokens: InventoryUiTokens) {
    tokens
        .control_surface()
        .draw(frame, UiLayer::Menu, layout.backpack_sort);

    frame.text_centered(
        UiLayer::Menu,
        layout.backpack_sort,
        "Trier",
        tokens.text.button * layout.scale,
        tokens.colors.text_primary,
    );
}

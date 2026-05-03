mod backpack_panel;

use vv_ui::{UiFrame, UiLayer, UiRect};

use crate::{design::InventoryUiTokens, GameplayUiContext, InventoryUiLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct InventoryScreen;

impl InventoryScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let layout = InventoryUiLayout::inventory(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        let tokens = InventoryUiTokens::current();

        frame.rect(
            UiLayer::Menu,
            UiRect::new(0.0, 0.0, ctx.screen_width, ctx.screen_height),
            tokens.colors.screen_dim,
        );

        draw_empty_panel(frame, layout.equipment_panel, "PERSONNAGE", tokens);
        backpack_panel::draw(frame, ctx, &layout, tokens);
        draw_empty_panel(frame, layout.crafting_panel, "CRAFT", tokens);
    }
}

fn draw_empty_panel(frame: &mut UiFrame, rect: UiRect, title: &str, tokens: InventoryUiTokens) {
    tokens.panel_surface().draw(frame, UiLayer::Menu, rect);

    let pad = tokens.layout.panel_padding * 0.75;
    frame.text_left_centered(
        UiLayer::Menu,
        UiRect::new(rect.x + pad, rect.y + 22.0, rect.width - pad * 2.0, 34.0),
        title,
        20.0,
        tokens.colors.title,
    );
}

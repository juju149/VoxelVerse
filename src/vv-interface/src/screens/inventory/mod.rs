mod backpack_panel;

use vv_ui::{UiFrame, UiLayer, UiRect};

use crate::{
    design::InventoryUiTokens, screens::hotbar::draw_hotbar, GameplayUiContext, InventoryUiLayout,
};

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

        draw_empty_panel(
            frame,
            layout.equipment_panel,
            "PERSONNAGE",
            tokens,
            layout.scale,
        );
        backpack_panel::draw(frame, ctx, &layout, tokens);
        draw_empty_panel(frame, layout.crafting_panel, "CRAFT", tokens, layout.scale);

        draw_hotbar(frame, ctx, &layout, UiLayer::Menu);
    }
}

fn draw_empty_panel(
    frame: &mut UiFrame,
    rect: UiRect,
    title: &str,
    tokens: InventoryUiTokens,
    scale: f32,
) {
    tokens.panel_surface().draw(frame, UiLayer::Menu, rect);

    let pad = tokens.layout.panel_padding * scale;

    frame.text_left_centered(
        UiLayer::Menu,
        UiRect::new(
            rect.x + pad,
            rect.y + tokens.layout.title_top * scale,
            rect.width - pad * 2.0,
            34.0 * scale,
        ),
        title,
        tokens.text.panel_title * scale,
        tokens.colors.title,
    );
}

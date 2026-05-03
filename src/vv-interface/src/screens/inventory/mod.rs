use vv_ui::{UiBorder, UiFrame, UiLayer, UiRect, UiShadow};

use crate::{design::VvInventoryUiTokens, GameplayUiContext, InventoryUiLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct InventoryScreen;

impl InventoryScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let tokens = VvInventoryUiTokens::current();

        let layout = InventoryUiLayout::inventory(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        draw_background(frame, ctx, &tokens);

        draw_panel_shell(
            frame,
            layout.equipment_panel,
            "ÉQUIPEMENT",
            &tokens,
            layout.scale,
        );

        draw_panel_shell(
            frame,
            layout.backpack_panel,
            "SAC À DOS",
            &tokens,
            layout.scale,
        );

        draw_backpack_header(frame, layout.backpack_panel, &tokens, layout.scale);

        draw_panel_shell(
            frame,
            layout.crafting_panel,
            "ARTISANAT RAPIDE",
            &tokens,
            layout.scale,
        );
    }
}

fn draw_background(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, tokens: &VvInventoryUiTokens) {
    frame.rect(
        UiLayer::Menu,
        UiRect::new(0.0, 0.0, ctx.screen_width, ctx.screen_height),
        tokens.colors.screen_dim,
    );
}

fn draw_panel_shell(
    frame: &mut UiFrame,
    rect: UiRect,
    title: &str,
    tokens: &VvInventoryUiTokens,
    scale: f32,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let radius = (tokens.panel.radius * scale).max(7.0);
    let border_width = (tokens.panel.border_width * scale).clamp(1.0, 2.0);

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        tokens.colors.panel_fill,
        radius,
        UiBorder::new(border_width, tokens.colors.panel_border),
        UiShadow::NONE,
    );

    let pad_x = tokens.panel.padding_x * scale;
    let title_y = rect.y + tokens.panel.title_top * scale;

    frame.text(
        UiLayer::Menu,
        UiRect::new(
            rect.x + pad_x,
            title_y,
            rect.width - pad_x * 2.0,
            34.0 * scale,
        ),
        title,
        tokens.text.panel_title_size * scale,
        tokens.colors.panel_title,
    );
}

fn draw_backpack_header(
    frame: &mut UiFrame,
    panel: UiRect,
    tokens: &VvInventoryUiTokens,
    scale: f32,
) {
    let pad = tokens.panel.padding_x * scale;
    let y = panel.y + tokens.controls.search_top * scale;
    let h = tokens.controls.control_height * scale;
    let gap = tokens.controls.search_sort_gap * scale;
    let sort_w = tokens.controls.sort_button_width * scale;

    let search_x = panel.x + pad;
    let search_w = (panel.width - pad * 2.0 - gap - sort_w).max(0.0);

    let search_rect = UiRect::new(search_x, y, search_w, h);
    let sort_rect = UiRect::new(search_rect.right() + gap, y, sort_w, h);

    draw_search_field(frame, search_rect, tokens, scale);
    draw_sort_button(frame, sort_rect, tokens, scale);
}

fn draw_search_field(frame: &mut UiFrame, rect: UiRect, tokens: &VvInventoryUiTokens, scale: f32) {
    let radius = tokens.controls.control_radius * scale;
    let border_width = (tokens.controls.control_border_width * scale).clamp(1.0, 1.75);
    let pad_x = tokens.controls.search_padding_x * scale;

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        tokens.colors.control_fill,
        radius,
        UiBorder::new(border_width, tokens.colors.control_border),
        UiShadow::NONE,
    );

    frame.text_left_centered(
        UiLayer::Menu,
        UiRect::new(
            rect.x + pad_x,
            rect.y,
            (rect.width - pad_x * 2.0).max(0.0),
            rect.height,
        ),
        "Rechercher un objet...",
        tokens.text.control_text_size * scale,
        tokens.colors.control_placeholder,
    );
}

fn draw_sort_button(frame: &mut UiFrame, rect: UiRect, tokens: &VvInventoryUiTokens, scale: f32) {
    let radius = tokens.controls.control_radius * scale;
    let border_width = (tokens.controls.control_border_width * scale).clamp(1.0, 1.75);

    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        tokens.colors.control_fill_hoverless,
        radius,
        UiBorder::new(border_width, tokens.colors.control_border),
        UiShadow::NONE,
    );

    frame.text_centered(
        UiLayer::Menu,
        rect,
        "Trier",
        tokens.text.control_text_size * scale,
        tokens.colors.control_text,
    );
}

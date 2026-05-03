use vv_ui::{UiBorder, UiColor, UiFrame, UiLayer, UiRect, UiShadow};

use crate::{GameplayUiContext, InventoryUiLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct InventoryScreen;

impl InventoryScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let layout = InventoryUiLayout::inventory(
            ctx.screen_width,
            ctx.screen_height,
            &ctx.gameplay.inventory,
        );

        draw_background(frame, ctx);
        draw_panel_shell(
            frame,
            layout.equipment_panel,
            "ÉQUIPEMENT",
            ctx,
            layout.scale,
        );
        draw_panel_shell(frame, layout.backpack_panel, "SAC À DOS", ctx, layout.scale);
        draw_panel_shell(
            frame,
            layout.crafting_panel,
            "ARTISANAT RAPIDE",
            ctx,
            layout.scale,
        );
    }
}

fn draw_background(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
    frame.rect(
        UiLayer::Menu,
        UiRect::new(0.0, 0.0, ctx.screen_width, ctx.screen_height),
        UiColor::rgba(0.001, 0.006, 0.010, 0.52),
    );
}

fn draw_panel_shell(
    frame: &mut UiFrame,
    rect: UiRect,
    title: &str,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let panel_fill = UiColor::rgb(6.0 / 255.0, 22.0 / 255.0, 34.0 / 255.0);
    let panel_border = UiColor::rgba(166.0 / 255.0, 106.0 / 255.0, 24.0 / 255.0, 0.82);
    let title_color = UiColor::rgb(242.0 / 255.0, 165.0 / 255.0, 31.0 / 255.0);

    let radius = (10.0 * scale).max(7.0);
    let border_width = (1.5 * scale).clamp(1.0, 2.0);

    // Une seule vraie bordure. Pas de rect externe + rect interne.
    // Pas de shadow, pas de glow, pas de highlight parasite.
    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        panel_fill,
        radius,
        UiBorder::new(border_width, panel_border),
        UiShadow::NONE,
    );

    let pad_x = 24.0 * scale;
    let title_y = rect.y + 24.0 * scale;

    frame.text(
        UiLayer::Menu,
        UiRect::new(
            rect.x + pad_x,
            title_y,
            rect.width - pad_x * 2.0,
            34.0 * scale,
        ),
        title,
        20.0 * scale,
        title_color,
    );

    let _ = ctx;
}

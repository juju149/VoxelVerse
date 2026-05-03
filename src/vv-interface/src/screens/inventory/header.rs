use vv_ui::{UiFrame, UiRect};

use crate::{
    components::{button, text},
    GameplayUiContext, InventoryUiLayout,
};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let s = layout.scale;
    let title = layout.title_bar;

    let icon = UiRect::new(title.x, title.y + 4.0 * s, 62.0 * s, 62.0 * s);
    button::square_icon(frame, icon, "▣", ctx, s);

    text::left_centered(
        frame,
        vv_ui::UiLayer::Menu,
        UiRect::new(
            icon.right() + 24.0 * s,
            title.y + 2.0 * s,
            420.0 * s,
            42.0 * s,
        ),
        "INVENTAIRE",
        32.0 * s,
        ctx.theme.text_primary,
    );

    text::left_centered(
        frame,
        vv_ui::UiLayer::Menu,
        UiRect::new(
            icon.right() + 24.0 * s,
            title.y + 44.0 * s,
            520.0 * s,
            26.0 * s,
        ),
        "Sac, équipement et ressources",
        15.0 * s,
        ctx.theme.text_muted,
    );

    let close_size = 54.0 * s;
    let close = UiRect::new(
        title.right() - close_size,
        title.y + 4.0 * s,
        close_size,
        close_size,
    );

    button::square_icon(frame, close, "×", ctx, s);
}

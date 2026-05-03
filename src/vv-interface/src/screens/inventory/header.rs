use vv_ui::{UiButton, UiFrame, UiInput, UiLayer, UiRect, UiStyle, UiWidgetId};

use crate::{GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let s = layout.scale;
    let title = layout.title_bar;
    let styles = UiStyle::from_theme(ctx.theme);
    let input = UiInput::default();

    let icon = UiRect::new(title.x, title.y + 4.0 * s, 62.0 * s, 62.0 * s);

    UiButton::new(UiWidgetId::new(100), icon, "▣", styles.button)
        .text_size(24.0 * s)
        .draw(frame, &input, None);

    frame.text_left_centered(
        UiLayer::Menu,
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

    frame.text_left_centered(
        UiLayer::Menu,
        UiRect::new(
            icon.right() + 24.0 * s,
            title.y + 44.0 * s,
            520.0 * s,
            26.0 * s,
        ),
        "Sac, équipement et artisanat",
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

    UiButton::new(UiWidgetId::new(101), close, "×", styles.button)
        .text_size(32.0 * s)
        .draw(frame, &input, None);
}

use vv_ui::{UiButton, UiFrame, UiInput, UiRect, UiStyle, UiWidgetId};

use crate::{GameplayUiContext, InventoryUiLayout};

pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>, layout: &InventoryUiLayout) {
    let s = layout.scale;
    let styles = UiStyle::from_theme(ctx.theme);
    let input = UiInput::default();

    let key_w = 70.0 * s;
    let label_w = 84.0 * s;
    let h = 38.0 * s;
    let gap = 8.0 * s;
    let total_w = key_w + label_w + gap;

    let x = layout.title_bar.right() - total_w;
    let y = ctx.screen_height - 64.0 * s;

    UiButton::new(
        UiWidgetId::new(5_000),
        UiRect::new(x, y, key_w, h),
        "ÉCHAP",
        styles.button,
    )
    .text_size(12.0 * s)
    .draw(frame, &input, None);

    UiButton::new(
        UiWidgetId::new(5_001),
        UiRect::new(x + key_w + gap, y, label_w, h),
        "Fermer",
        styles.button,
    )
    .text_size(12.0 * s)
    .draw(frame, &input, None);
}

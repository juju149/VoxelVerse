use vv_ui::{UiFrame, UiLayer, UiRect};

use crate::{
    components::{button, text},
    GameplayUiContext,
};

pub fn draw_pair(
    frame: &mut UiFrame,
    x: f32,
    y: f32,
    key: &str,
    label: &str,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    let key_w = 70.0 * scale;
    let label_w = 84.0 * scale;
    let h = 38.0 * scale;
    let gap = 8.0 * scale;

    let key_rect = UiRect::new(x, y, key_w, h);
    let label_rect = UiRect::new(x + key_w + gap, y, label_w, h);

    button::pill(
        frame,
        key_rect,
        "",
        button::VvButtonVariant::Default,
        ctx,
        scale,
    );
    text::centered(
        frame,
        UiLayer::Menu,
        key_rect,
        key,
        12.0 * scale,
        ctx.theme.text_primary,
    );

    button::pill(
        frame,
        label_rect,
        "",
        button::VvButtonVariant::Default,
        ctx,
        scale,
    );
    text::centered(
        frame,
        UiLayer::Menu,
        label_rect,
        label,
        12.0 * scale,
        ctx.theme.text_muted,
    );
}

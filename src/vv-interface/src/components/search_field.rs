use vv_ui::{UiColor, UiEdgeInsets, UiFrame, UiLayer, UiRect};

use crate::{
    components::{surface, text},
    GameplayUiContext,
};

pub fn draw(
    frame: &mut UiFrame,
    rect: UiRect,
    placeholder: &str,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    surface::filled(
        frame,
        UiLayer::Menu,
        rect,
        UiColor::rgba(0.010, 0.030, 0.036, 0.82),
        UiColor::rgba(0.58, 0.36, 0.16, 0.62),
        1.5 * scale,
        8.0 * scale,
    );

    text::left_centered(
        frame,
        UiLayer::Menu,
        rect.inset(UiEdgeInsets::symmetric(16.0 * scale, 0.0)),
        placeholder,
        15.0 * scale,
        ctx.theme.text_muted,
    );

    text::centered(
        frame,
        UiLayer::Menu,
        UiRect::new(
            rect.right() - 44.0 * scale,
            rect.y,
            34.0 * scale,
            rect.height,
        ),
        "⌕",
        20.0 * scale,
        ctx.theme.accent,
    );
}

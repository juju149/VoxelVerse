use vv_ui::{UiBorder, UiColor, UiFrame, UiLayer, UiRect, UiShadow};

use crate::GameplayUiContext;

pub fn glass(frame: &mut UiFrame, rect: UiRect, ctx: &GameplayUiContext<'_>) {
    frame.rounded_rect(
        UiLayer::Menu,
        rect,
        ctx.theme.panel,
        12.0,
        UiBorder::new(1.0, ctx.theme.border_soft),
        UiShadow::new(0.0, 16.0, 34.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.38)),
    );
}

pub fn section_title(
    frame: &mut UiFrame,
    rect: UiRect,
    title: &str,
    ctx: &GameplayUiContext<'_>,
    size: f32,
) {
    frame.text(UiLayer::Menu, rect, title, size, ctx.theme.accent);
}

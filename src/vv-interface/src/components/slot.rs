use vv_ui::{UiBorder, UiColor, UiEdgeInsets, UiFrame, UiGradient, UiLayer, UiRect, UiShadow};

use crate::{
    components::{surface, text},
    GameplayUiContext, ItemVisual,
};

pub fn item_slot(
    frame: &mut UiFrame,
    rect: UiRect,
    selected: bool,
    visual: Option<&ItemVisual>,
    hidden_by_drag: bool,
    theme: &vv_ui::UiTheme,
    layer: UiLayer,
) {
    let radius = (rect.width * 0.13).clamp(7.0, 13.0);
    let border_width = if selected { 2.5 } else { 2.0 };

    let border_color = if selected {
        theme.accent.with_alpha(0.90)
    } else {
        UiColor::rgba(0.48, 0.29, 0.12, 0.78)
    };

    let fill_color = if selected {
        UiColor::rgba(0.018, 0.044, 0.050, 0.98)
    } else {
        UiColor::rgba(0.008, 0.025, 0.030, 0.98)
    };

    surface::filled(
        frame,
        layer,
        rect,
        fill_color,
        border_color,
        border_width,
        radius,
    );

    if hidden_by_drag {
        return;
    }

    let Some(visual) = visual else {
        return;
    };

    let item_rect = rect.inset(UiEdgeInsets::all(rect.width * 0.27));

    frame.gradient_rect(
        layer,
        item_rect,
        UiGradient::vertical(visual.color.lighten(0.20), visual.color.darken(0.18)),
        (radius * 0.42).max(4.0),
        UiBorder::new(1.0, UiColor::rgba(1.0, 0.94, 0.72, 0.12)),
        UiShadow::NONE,
    );

    if visual.count > 1 {
        text::right_centered(
            frame,
            layer,
            UiRect::new(
                rect.x + rect.width * 0.52,
                rect.y + rect.height * 0.66,
                rect.width * 0.38,
                rect.height * 0.24,
            ),
            visual.count,
            (rect.height * 0.20).clamp(10.0, 15.0),
            theme.text_primary,
        );
    }
}

pub fn equipment_slot(
    frame: &mut UiFrame,
    rect: UiRect,
    label: &str,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    text::centered(
        frame,
        UiLayer::Menu,
        UiRect::new(
            rect.x - 8.0 * scale,
            rect.y - 19.0 * scale,
            rect.width + 16.0 * scale,
            18.0 * scale,
        ),
        label,
        10.0 * scale,
        ctx.theme.text_muted,
    );

    surface::filled(
        frame,
        UiLayer::Menu,
        rect,
        ctx.theme.panel_subtle,
        ctx.theme.border_soft,
        1.0,
        7.0 * scale,
    );
}

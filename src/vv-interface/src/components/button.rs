use vv_ui::{UiColor, UiFrame, UiGradient, UiLayer, UiRect};

use crate::{
    components::{surface, text},
    GameplayUiContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VvButtonVariant {
    Default,
    Active,
    Action,
    Disabled,
}

pub fn pill(
    frame: &mut UiFrame,
    rect: UiRect,
    label: impl ToString,
    variant: VvButtonVariant,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    let radius = (rect.height * 0.22).clamp(6.0 * scale, 11.0 * scale);
    let active = matches!(variant, VvButtonVariant::Active | VvButtonVariant::Action);
    let disabled = matches!(variant, VvButtonVariant::Disabled);

    let border = if active {
        ctx.theme.border.with_alpha(0.90)
    } else {
        UiColor::rgba(0.45, 0.28, 0.12, 0.66)
    };

    let top = if active {
        UiColor::rgba(0.73, 0.48, 0.16, 0.92)
    } else {
        UiColor::rgba(0.018, 0.043, 0.050, 0.88)
    };

    let bottom = if active {
        UiColor::rgba(0.42, 0.26, 0.08, 0.94)
    } else {
        UiColor::rgba(0.006, 0.020, 0.024, 0.90)
    };

    surface::gradient(
        frame,
        UiLayer::Menu,
        rect,
        UiGradient::vertical(top, bottom),
        border,
        1.4 * scale,
        radius,
    );

    text::centered(
        frame,
        UiLayer::Menu,
        rect,
        label,
        13.0 * scale,
        if disabled {
            ctx.theme.text_disabled
        } else if active {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_muted
        },
    );
}

pub fn square_icon(
    frame: &mut UiFrame,
    rect: UiRect,
    label: impl ToString,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    pill(frame, rect, "", VvButtonVariant::Default, ctx, scale);

    let label = label.to_string();
    let size = if label == "×" {
        32.0 * scale
    } else {
        24.0 * scale
    };

    text::centered(frame, UiLayer::Menu, rect, label, size, ctx.theme.accent);
}

pub fn action(
    frame: &mut UiFrame,
    rect: UiRect,
    label: impl ToString,
    enabled: bool,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    let radius = 9.0 * scale;
    let border = if enabled {
        ctx.theme.border.with_alpha(0.95)
    } else {
        ctx.theme.border_soft.with_alpha(0.52)
    };

    let top = if enabled {
        UiColor::rgba(0.78, 0.50, 0.15, 0.94)
    } else {
        UiColor::rgba(0.020, 0.043, 0.050, 0.90)
    };

    let bottom = if enabled {
        UiColor::rgba(0.42, 0.25, 0.07, 0.96)
    } else {
        UiColor::rgba(0.006, 0.020, 0.025, 0.94)
    };

    surface::gradient(
        frame,
        UiLayer::Menu,
        rect,
        UiGradient::vertical(top, bottom),
        border,
        1.5 * scale,
        radius,
    );

    text::centered(
        frame,
        UiLayer::Menu,
        rect,
        label,
        17.0 * scale,
        if enabled {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_disabled
        },
    );
}

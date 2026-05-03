use vv_ui::{UiFrame, UiGradient, UiLayer, UiRect};

use crate::{
    components::{surface, text},
    design::VvDesignTokens,
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
    let tokens = VvDesignTokens::current();
    let active = matches!(variant, VvButtonVariant::Active | VvButtonVariant::Action);
    let disabled = matches!(variant, VvButtonVariant::Disabled);

    let radius = scaled_radius(
        rect.height,
        scale,
        tokens.button.radius_factor,
        tokens.button.radius_min,
        tokens.button.radius_max,
    );

    let border = if active {
        tokens.colors.button_border_active
    } else {
        tokens.colors.button_border
    };

    let border_width = if active {
        tokens.button.active_border_width
    } else {
        tokens.button.border_width
    } * scale;

    surface::gradient(
        frame,
        UiLayer::Menu,
        rect,
        button_gradient(variant, &tokens),
        border,
        border_width,
        radius,
    );

    let label = label.to_string();

    if !label.is_empty() {
        text::centered(
            frame,
            UiLayer::Menu,
            rect,
            label,
            tokens.text.button_size * scale,
            if disabled {
                ctx.theme.text_disabled
            } else if active {
                ctx.theme.text_primary
            } else {
                ctx.theme.text_muted
            },
        );
    }
}

pub fn square_icon(
    frame: &mut UiFrame,
    rect: UiRect,
    label: impl ToString,
    ctx: &GameplayUiContext<'_>,
    scale: f32,
) {
    pill(frame, rect, "", VvButtonVariant::Default, ctx, scale);

    let tokens = VvDesignTokens::current();
    let label = label.to_string();
    let size = if label == "×" {
        tokens.text.button_size_large * 1.9 * scale
    } else {
        tokens.text.button_size_large * 1.4 * scale
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
    let tokens = VvDesignTokens::current();
    let variant = if enabled {
        VvButtonVariant::Action
    } else {
        VvButtonVariant::Disabled
    };

    let radius = scaled_radius(
        rect.height,
        scale,
        tokens.button.radius_factor,
        tokens.button.radius_min,
        tokens.button.radius_max,
    );

    let border = if enabled {
        tokens.colors.button_border_active
    } else {
        tokens.colors.button_border
    };

    surface::gradient(
        frame,
        UiLayer::Menu,
        rect,
        button_gradient(variant, &tokens),
        border,
        tokens.button.active_border_width * scale,
        radius,
    );

    text::centered(
        frame,
        UiLayer::Menu,
        rect,
        label,
        tokens.text.button_size_large * scale,
        if enabled {
            ctx.theme.text_primary
        } else {
            ctx.theme.text_disabled
        },
    );
}

fn button_gradient(variant: VvButtonVariant, tokens: &VvDesignTokens) -> UiGradient {
    match variant {
        VvButtonVariant::Default => {
            UiGradient::vertical(tokens.colors.button_top, tokens.colors.button_bottom)
        }
        VvButtonVariant::Active | VvButtonVariant::Action => UiGradient::vertical(
            tokens.colors.button_active_top,
            tokens.colors.button_active_bottom,
        ),
        VvButtonVariant::Disabled => UiGradient::vertical(
            tokens.colors.button_disabled_top,
            tokens.colors.button_disabled_bottom,
        ),
    }
}

fn scaled_radius(height: f32, scale: f32, factor: f32, min_radius: f32, max_radius: f32) -> f32 {
    (height * factor).clamp(min_radius * scale, max_radius * scale)
}

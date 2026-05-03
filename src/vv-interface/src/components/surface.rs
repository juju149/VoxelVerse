use vv_ui::{UiBorder, UiColor, UiEdgeInsets, UiFrame, UiGradient, UiLayer, UiRect, UiShadow};

pub fn filled(
    frame: &mut UiFrame,
    layer: UiLayer,
    rect: UiRect,
    fill: UiColor,
    border: UiColor,
    border_width: f32,
    radius: f32,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    if border_width > 0.0 && border.a > 0.001 {
        frame.rounded_rect(layer, rect, border, radius, UiBorder::NONE, UiShadow::NONE);

        frame.rounded_rect(
            layer,
            rect.inset(UiEdgeInsets::all(border_width)),
            fill,
            (radius - border_width).max(3.0),
            UiBorder::NONE,
            UiShadow::NONE,
        );
    } else {
        frame.rounded_rect(layer, rect, fill, radius, UiBorder::NONE, UiShadow::NONE);
    }
}

pub fn gradient(
    frame: &mut UiFrame,
    layer: UiLayer,
    rect: UiRect,
    gradient: UiGradient,
    border: UiColor,
    border_width: f32,
    radius: f32,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    if border_width > 0.0 && border.a > 0.001 {
        frame.rounded_rect(layer, rect, border, radius, UiBorder::NONE, UiShadow::NONE);

        frame.gradient_rect(
            layer,
            rect.inset(UiEdgeInsets::all(border_width)),
            gradient,
            (radius - border_width).max(3.0),
            UiBorder::NONE,
            UiShadow::NONE,
        );
    } else {
        frame.gradient_rect(
            layer,
            rect,
            gradient,
            radius,
            UiBorder::NONE,
            UiShadow::NONE,
        );
    }
}

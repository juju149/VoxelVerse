use vv_ui::{UiColor, UiFrame, UiLayer, UiRect, UiTextAlign};

pub fn centered(
    frame: &mut UiFrame,
    layer: UiLayer,
    rect: UiRect,
    text: impl ToString,
    size: f32,
    color: UiColor,
) {
    frame.text_aligned(
        layer,
        vertically_centered_text_rect(rect, size),
        text.to_string(),
        size,
        color,
        UiTextAlign::Center,
    );
}

pub fn left_centered(
    frame: &mut UiFrame,
    layer: UiLayer,
    rect: UiRect,
    text: impl ToString,
    size: f32,
    color: UiColor,
) {
    frame.text_aligned(
        layer,
        vertically_centered_text_rect(rect, size),
        text.to_string(),
        size,
        color,
        UiTextAlign::Left,
    );
}

pub fn right_centered(
    frame: &mut UiFrame,
    layer: UiLayer,
    rect: UiRect,
    text: impl ToString,
    size: f32,
    color: UiColor,
) {
    frame.text_aligned(
        layer,
        vertically_centered_text_rect(rect, size),
        text.to_string(),
        size,
        color,
        UiTextAlign::Right,
    );
}

pub fn vertically_centered_text_rect(rect: UiRect, size: f32) -> UiRect {
    let line_h = (size * 1.18).max(size + 2.0);
    let optical_y = rect.y + (rect.height - line_h) * 0.5 + size * 0.20;

    UiRect::new(rect.x, optical_y, rect.width, line_h)
}

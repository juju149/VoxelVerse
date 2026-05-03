use crate::{
    UiBorder, UiButtonStyle, UiEdgeInsets, UiFrame, UiGradient, UiIconId, UiInput, UiLayer, UiRect,
    UiResponse, UiShadow, UiSurface, UiWidgetId,
};

pub type UiButtonResponse = UiResponse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiButtonIconPlacement {
    Leading,
    Trailing,
}

impl Default for UiButtonIconPlacement {
    fn default() -> Self {
        Self::Leading
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiButtonContentAlign {
    Left,
    Center,
    Right,
}

impl Default for UiButtonContentAlign {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Debug, Clone)]
pub struct UiButton {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub label: String,
    pub icon: Option<UiIconId>,
    pub icon_placement: UiButtonIconPlacement,
    pub content_align: UiButtonContentAlign,
    pub style: UiButtonStyle,
    pub layer: UiLayer,
    pub disabled: bool,
    pub text_size: Option<f32>,
    pub icon_size: Option<f32>,
    pub icon_gap: Option<f32>,
    pub padding_x: Option<f32>,
}

impl UiButton {
    pub fn new(
        id: UiWidgetId,
        rect: UiRect,
        label: impl Into<String>,
        style: UiButtonStyle,
    ) -> Self {
        Self {
            id,
            rect,
            label: label.into(),
            icon: None,
            icon_placement: UiButtonIconPlacement::Leading,
            content_align: UiButtonContentAlign::Center,
            style,
            layer: UiLayer::Menu,
            disabled: false,
            text_size: None,
            icon_size: None,
            icon_gap: None,
            padding_x: None,
        }
    }

    pub fn icon(mut self, icon: UiIconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_placement(mut self, placement: UiButtonIconPlacement) -> Self {
        self.icon_placement = placement;
        self
    }

    pub fn content_align(mut self, align: UiButtonContentAlign) -> Self {
        self.content_align = align;
        self
    }

    pub fn layer(mut self, layer: UiLayer) -> Self {
        self.layer = layer;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size.max(1.0));
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = Some(size.max(1.0));
        self
    }

    pub fn icon_gap(mut self, gap: f32) -> Self {
        self.icon_gap = Some(gap.max(0.0));
        self
    }

    pub fn padding_x(mut self, padding: f32) -> Self {
        self.padding_x = Some(padding.max(0.0));
        self
    }

    pub fn draw(
        self,
        frame: &mut UiFrame,
        input: &UiInput,
        active: Option<UiWidgetId>,
    ) -> UiButtonResponse {
        let response = UiResponse::from_input(self.id, self.rect, input, active, self.disabled);

        if self.rect.width <= 0.0 || self.rect.height <= 0.0 {
            return response;
        }

        self.draw_surface(frame, &response);
        self.draw_content(frame);

        response
    }

    fn draw_surface(&self, frame: &mut UiFrame, response: &UiResponse) {
        let fill = if self.disabled {
            self.style.background.darken(0.35)
        } else if response.pressed {
            self.style.background_pressed
        } else if response.hovered {
            self.style.background_hover
        } else {
            self.style.background
        };

        let gradient = if self.disabled {
            self.style.background_gradient.map(|gradient| {
                UiGradient::vertical(gradient.top.darken(0.35), gradient.bottom.darken(0.35))
            })
        } else if response.pressed {
            self.style.background_pressed_gradient
        } else if response.hovered {
            self.style.background_hover_gradient
        } else {
            self.style.background_gradient
        };

        if let Some(gradient) = gradient {
            draw_gradient_surface(
                frame,
                self.layer,
                self.rect,
                gradient,
                self.style.border,
                self.style.radius,
                self.style.shadow,
            );
        } else {
            frame.surface(
                self.layer,
                self.rect,
                UiSurface::new(fill)
                    .border(self.style.border.color, self.style.border.width)
                    .radius(self.style.radius)
                    .shadow(self.style.shadow),
            );
        }
    }

    fn draw_content(&self, frame: &mut UiFrame) {
        let label = self.label.trim();
        let has_text = !label.is_empty();
        let has_icon = self.icon.is_some();

        if !has_text && !has_icon {
            return;
        }

        let text_size = self
            .text_size
            .unwrap_or_else(|| (self.rect.height * 0.34).clamp(13.0, 22.0));

        let padding_x = self
            .padding_x
            .unwrap_or_else(|| (self.rect.height * 0.34).clamp(10.0, 22.0));

        let content_rect = self.rect.inset(UiEdgeInsets::symmetric(padding_x, 0.0));

        if content_rect.width <= 0.0 || content_rect.height <= 0.0 {
            return;
        }

        let content_color = if self.disabled {
            self.style.text_disabled
        } else {
            self.style.text
        };

        if !has_icon {
            draw_text_manual(
                frame,
                self.layer,
                content_rect,
                label,
                text_size,
                content_color,
                self.content_align,
            );
            return;
        }

        let icon = self.icon.expect("has_icon checked above");
        let icon_size = self
            .icon_size
            .unwrap_or_else(|| (self.rect.height * 0.46).clamp(12.0, 28.0))
            .min(content_rect.height * 0.72)
            .min(content_rect.width);

        if !has_text {
            let icon_rect = center_rect(content_rect, icon_size, icon_size);
            frame.icon(self.layer, icon_rect, icon, content_color);
            return;
        }

        let gap = self
            .icon_gap
            .unwrap_or_else(|| (self.rect.height * 0.16).clamp(6.0, 12.0));

        let available_text_width = (content_rect.width - icon_size - gap).max(0.0);

        if available_text_width <= 0.0 {
            let icon_rect = center_rect(content_rect, icon_size, icon_size);
            frame.icon(self.layer, icon_rect, icon, content_color);
            return;
        }

        let desired_text_width = estimate_text_width(label, text_size);
        let text_width = desired_text_width.min(available_text_width);
        let group_width = icon_size + gap + text_width;
        let group_x = aligned_x(content_rect, group_width, self.content_align);

        match self.icon_placement {
            UiButtonIconPlacement::Leading => {
                let icon_rect = UiRect::new(
                    group_x,
                    content_rect.y + (content_rect.height - icon_size) * 0.5,
                    icon_size,
                    icon_size,
                );

                let text_rect = UiRect::new(
                    icon_rect.right() + gap,
                    content_rect.y,
                    text_width,
                    content_rect.height,
                );

                frame.icon(self.layer, icon_rect, icon, content_color);
                frame.text_left_centered(self.layer, text_rect, label, text_size, content_color);
            }
            UiButtonIconPlacement::Trailing => {
                let text_rect =
                    UiRect::new(group_x, content_rect.y, text_width, content_rect.height);

                let icon_rect = UiRect::new(
                    text_rect.right() + gap,
                    content_rect.y + (content_rect.height - icon_size) * 0.5,
                    icon_size,
                    icon_size,
                );

                frame.text_left_centered(self.layer, text_rect, label, text_size, content_color);
                frame.icon(self.layer, icon_rect, icon, content_color);
            }
        }
    }
}

fn draw_gradient_surface(
    frame: &mut UiFrame,
    layer: UiLayer,
    rect: UiRect,
    gradient: UiGradient,
    border: UiBorder,
    radius: f32,
    shadow: UiShadow,
) {
    if border.width > 0.0 && border.color.a > 0.001 {
        frame.rounded_rect(layer, rect, border.color, radius, UiBorder::NONE, shadow);

        let inner = rect.inset(UiEdgeInsets::all(border.width));

        if inner.width > 0.0 && inner.height > 0.0 {
            frame.gradient_rect(
                layer,
                inner,
                gradient,
                (radius - border.width).max(0.0),
                UiBorder::NONE,
                UiShadow::NONE,
            );
        }
    } else {
        frame.gradient_rect(layer, rect, gradient, radius, UiBorder::NONE, shadow);
    }
}

fn draw_text_manual(
    frame: &mut UiFrame,
    layer: UiLayer,
    bounds: UiRect,
    label: &str,
    size: f32,
    color: crate::UiColor,
    align: UiButtonContentAlign,
) {
    let text_width = estimate_text_width(label, size).min(bounds.width.max(0.0));
    let x = aligned_x(bounds, text_width, align);
    let rect = UiRect::new(x.round(), bounds.y, text_width.ceil() + 1.0, bounds.height);

    frame.text_left_centered(layer, rect, label, size, color);
}

fn center_rect(bounds: UiRect, width: f32, height: f32) -> UiRect {
    UiRect::new(
        bounds.x + (bounds.width - width) * 0.5,
        bounds.y + (bounds.height - height) * 0.5,
        width,
        height,
    )
}

fn aligned_x(bounds: UiRect, content_width: f32, align: UiButtonContentAlign) -> f32 {
    match align {
        UiButtonContentAlign::Left => bounds.x,
        UiButtonContentAlign::Center => bounds.x + (bounds.width - content_width) * 0.5,
        UiButtonContentAlign::Right => bounds.right() - content_width,
    }
}

fn estimate_text_width(text: &str, size: f32) -> f32 {
    text.chars().map(|ch| glyph_width(ch, size)).sum()
}

fn glyph_width(ch: char, size: f32) -> f32 {
    match ch {
        ' ' => size * 0.35,
        'i' | 'l' | 'I' | '!' | '|' | '.' | ',' | ':' | ';' => size * 0.36,
        'm' | 'w' | 'M' | 'W' => size * 0.82,
        '×' | '+' | '−' | '-' | '↕' | '⌕' | '▣' => size * 0.72,
        c if c.is_ascii_digit() => size * 0.56,
        c if c.is_ascii_uppercase() => size * 0.68,
        _ => size * 0.58,
    }
}

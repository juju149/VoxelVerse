use crate::{UiFrame, UiInput, UiLayer, UiRect, UiResponse, UiSurface, UiTabsStyle, UiWidgetId};

#[derive(Debug, Clone)]
pub struct UiTab {
    pub id: UiWidgetId,
    pub label: String,
}

impl UiTab {
    pub fn new(id: UiWidgetId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiTabs {
    pub rect: UiRect,
    pub tabs: Vec<UiTab>,
    pub active: UiWidgetId,
    pub style: UiTabsStyle,
    pub layer: UiLayer,
}

impl UiTabs {
    pub fn new(rect: UiRect, tabs: Vec<UiTab>, active: UiWidgetId, style: UiTabsStyle) -> Self {
        Self {
            rect,
            tabs,
            active,
            style,
            layer: UiLayer::Menu,
        }
    }

    pub fn draw(self, frame: &mut UiFrame, input: &UiInput) -> Option<UiWidgetId> {
        if self.tabs.is_empty() || self.rect.width <= 0.0 || self.rect.height <= 0.0 {
            return None;
        }

        let gap = (self.rect.height * 0.16).clamp(7.0, 11.0);
        let text_size = (self.rect.height * 0.34).clamp(12.0, 17.0);
        let min_w = (self.rect.height * 1.46).clamp(64.0, 92.0);
        let max_w = (self.rect.height * 3.18).clamp(116.0, 160.0);
        let pad_x = (self.rect.height * 0.52).clamp(16.0, 26.0);

        let mut widths = self
            .tabs
            .iter()
            .map(|tab| {
                (estimate_text_width(&tab.label, text_size) + pad_x * 2.0).clamp(min_w, max_w)
            })
            .collect::<Vec<_>>();

        fit_widths(&mut widths, self.rect.width, gap, min_w * 0.78);

        let total_width = widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32;
        let mut x = self.rect.x + (self.rect.width - total_width).max(0.0) * 0.5;
        let mut clicked = None;

        for (index, tab) in self.tabs.iter().enumerate() {
            let width = widths[index];

            if width <= 0.0 {
                continue;
            }

            let rect = UiRect::new(
                x.round(),
                self.rect.y.round(),
                width.round(),
                self.rect.height.round(),
            );

            let response = UiResponse::from_input(tab.id, rect, input, None, false);
            let is_active = tab.id == self.active;

            let background = if is_active {
                self.style.active_background
            } else {
                self.style.background
            };

            let border_color = if is_active {
                self.style.border.color.lighten(0.20)
            } else {
                self.style.border.color
            };

            frame.surface(
                self.layer,
                rect,
                UiSurface::new(background)
                    .border(border_color, self.style.border.width)
                    .radius(self.style.radius),
            );

            draw_text_centered_manual(
                frame,
                self.layer,
                rect,
                &tab.label,
                text_size,
                if is_active {
                    self.style.active_text
                } else {
                    self.style.text
                },
            );

            if response.clicked {
                clicked = Some(tab.id);
            }

            x += width + gap;
        }

        clicked
    }
}

fn draw_text_centered_manual(
    frame: &mut UiFrame,
    layer: UiLayer,
    rect: UiRect,
    label: &str,
    size: f32,
    color: crate::UiColor,
) {
    let text_w = estimate_text_width(label, size).min(rect.width.max(0.0));
    let x = rect.x + (rect.width - text_w) * 0.5;

    frame.text_left_centered(
        layer,
        UiRect::new(x.round(), rect.y, text_w.ceil() + 1.0, rect.height),
        label,
        size,
        color,
    );
}

fn fit_widths(widths: &mut [f32], available_width: f32, gap: f32, min_width: f32) {
    if widths.is_empty() {
        return;
    }

    let total_gap = gap * widths.len().saturating_sub(1) as f32;
    let total_width = widths.iter().sum::<f32>() + total_gap;

    if total_width <= available_width {
        return;
    }

    let overflow = total_width - available_width;
    let shrinkable = widths
        .iter()
        .map(|width| (*width - min_width).max(0.0))
        .sum::<f32>();

    if shrinkable <= f32::EPSILON {
        let equal = ((available_width - total_gap) / widths.len() as f32).max(0.0);

        for width in widths {
            *width = equal;
        }

        return;
    }

    for width in widths {
        let share = (*width - min_width).max(0.0) / shrinkable;
        *width = (*width - overflow * share).max(min_width);
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
        c if c.is_ascii_digit() => size * 0.56,
        c if c.is_ascii_uppercase() => size * 0.68,
        _ => size * 0.58,
    }
}

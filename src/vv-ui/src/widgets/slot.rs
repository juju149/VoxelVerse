use crate::{
    UiColor, UiFrame, UiIconId, UiImageId, UiInput, UiLayer, UiRect, UiResponse, UiSlotStyle,
    UiSurface, UiWidgetId,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiSlotContent {
    Empty,
    Color(UiColor),
    Icon(UiIconId),
    Image(UiImageId),
}

#[derive(Debug, Clone, Copy)]
pub struct UiSlot {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub style: UiSlotStyle,
    pub content: UiSlotContent,
    pub count: Option<u32>,
    pub selected: bool,
    pub layer: UiLayer,
}

impl UiSlot {
    pub fn new(id: UiWidgetId, rect: UiRect, style: UiSlotStyle) -> Self {
        Self {
            id,
            rect,
            style,
            content: UiSlotContent::Empty,
            count: None,
            selected: false,
            layer: UiLayer::Menu,
        }
    }

    pub fn content(mut self, content: UiSlotContent) -> Self {
        self.content = content;
        self
    }

    pub fn count(mut self, count: Option<u32>) -> Self {
        self.count = count;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn layer(mut self, layer: UiLayer) -> Self {
        self.layer = layer;
        self
    }

    pub fn draw(
        self,
        frame: &mut UiFrame,
        input: &UiInput,
        active: Option<UiWidgetId>,
    ) -> UiResponse {
        let response = UiResponse::from_input(self.id, self.rect, input, active, false);

        let background = if self.selected {
            self.style.background_selected
        } else if response.hovered {
            self.style.background_hover
        } else {
            self.style.background
        };

        let border = if self.selected {
            self.style.selected_border
        } else {
            self.style.border
        };

        frame.surface(
            self.layer,
            self.rect,
            UiSurface::new(background)
                .border(border.color, border.width)
                .radius(self.style.radius),
        );

        let content_rect = self
            .rect
            .inset(crate::UiEdgeInsets::all(self.rect.width * 0.15));

        match self.content {
            UiSlotContent::Empty => {}
            UiSlotContent::Color(color) => {
                frame.rounded_rect(
                    self.layer,
                    content_rect,
                    color,
                    self.style.radius * 0.5,
                    crate::UiBorder::NONE,
                    crate::UiShadow::NONE,
                );
            }
            UiSlotContent::Icon(icon) => frame.icon(self.layer, content_rect, icon, UiColor::WHITE),
            UiSlotContent::Image(image) => frame.image(
                self.layer,
                content_rect,
                image,
                UiColor::WHITE,
                self.style.radius * 0.5,
            ),
        }

        if let Some(count) = self.count.filter(|count| *count > 1) {
            let text = count.to_string();
            let size = (self.rect.height * 0.24).clamp(10.0, 18.0);
            let text_w = estimate_text_width(&text, size);
            let x = self.rect.right() - self.rect.width * 0.10 - text_w;
            let y = self.rect.y + self.rect.height * 0.63;

            frame.text_left_centered(
                self.layer,
                UiRect::new(x.round(), y, text_w.ceil() + 1.0, self.rect.height * 0.28),
                text,
                size,
                UiColor::WHITE,
            );
        }

        response
    }
}

fn estimate_text_width(text: &str, size: f32) -> f32 {
    text.chars().map(|ch| glyph_width(ch, size)).sum()
}

fn glyph_width(ch: char, size: f32) -> f32 {
    match ch {
        ' ' => size * 0.35,
        '1' => size * 0.42,
        c if c.is_ascii_digit() => size * 0.56,
        _ => size * 0.58,
    }
}

use crate::{
    UiColor, UiFrame, UiIconId, UiImageId, UiInput, UiInteraction, UiLayer, UiRect, UiSlotStyle,
    UiTextAlign, UiWidgetId,
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
    ) -> UiInteraction {
        let interaction = UiInteraction::from_input(self.id, self.rect, input, active);

        let background = if self.selected {
            self.style.background_selected
        } else if interaction.hovered {
            self.style.background_hover
        } else {
            self.style.background
        };

        let border = if self.selected {
            self.style.selected_border
        } else {
            self.style.border
        };

        frame.rounded_rect(
            self.layer,
            self.rect,
            background,
            self.style.radius,
            border,
            crate::UiShadow::NONE,
        );

        let content_rect = self
            .rect
            .inset(crate::UiEdgeInsets::all(self.rect.width * 0.18));

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
            let text_rect = UiRect::new(
                self.rect.x + self.rect.width * 0.42,
                self.rect.y + self.rect.height * 0.62,
                self.rect.width * 0.52,
                self.rect.height * 0.32,
            );

            frame.text_aligned(
                self.layer,
                text_rect,
                count.to_string(),
                (self.rect.height * 0.22).clamp(10.0, 16.0),
                UiColor::WHITE,
                UiTextAlign::Right,
            );
        }

        interaction
    }
}

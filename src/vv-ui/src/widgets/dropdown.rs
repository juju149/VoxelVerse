use crate::{
    UiDropdownStyle, UiFrame, UiInput, UiInteraction, UiLayer, UiMouseButton, UiRect, UiTextAlign,
    UiWidgetId,
};

#[derive(Debug, Clone)]
pub struct UiDropdown {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub label: String,
    pub style: UiDropdownStyle,
    pub layer: UiLayer,
    pub disabled: bool,
}

impl UiDropdown {
    pub fn new(
        id: UiWidgetId,
        rect: UiRect,
        label: impl Into<String>,
        style: UiDropdownStyle,
    ) -> Self {
        Self {
            id,
            rect,
            label: label.into(),
            style,
            layer: UiLayer::Menu,
            disabled: false,
        }
    }

    pub fn draw(self, frame: &mut UiFrame, input: &UiInput, active: Option<UiWidgetId>) -> bool {
        let interaction = UiInteraction::from_input(self.id, self.rect, input, active);
        let hovered = interaction.hovered && !self.disabled;

        frame.rounded_rect(
            self.layer,
            self.rect,
            if hovered {
                self.style.background_hover
            } else {
                self.style.background
            },
            self.style.radius,
            self.style.border,
            crate::UiShadow::NONE,
        );

        frame.text_aligned(
            self.layer,
            self.rect.inset(crate::UiEdgeInsets::symmetric(12.0, 0.0)),
            self.label,
            (self.rect.height * 0.34).clamp(12.0, 18.0),
            crate::UiColor::WHITE,
            UiTextAlign::Left,
        );

        frame.text_aligned(
            self.layer,
            UiRect::new(
                self.rect.right() - 32.0,
                self.rect.y,
                24.0,
                self.rect.height,
            ),
            "⌄",
            (self.rect.height * 0.38).clamp(12.0, 20.0),
            crate::UiColor::WHITE,
            UiTextAlign::Center,
        );

        hovered && input.pointer_released(UiMouseButton::Primary)
    }
}

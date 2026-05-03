use crate::{
    UiColor, UiDropdownStyle, UiFrame, UiInput, UiLayer, UiRect, UiResponse, UiSurface, UiWidgetId,
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn draw(self, frame: &mut UiFrame, input: &UiInput, active: Option<UiWidgetId>) -> bool {
        let response = UiResponse::from_input(self.id, self.rect, input, active, self.disabled);

        let background = if response.hovered {
            self.style.background_hover
        } else {
            self.style.background
        };

        frame.surface(
            self.layer,
            self.rect,
            UiSurface::new(background)
                .border(self.style.border.color, self.style.border.width)
                .radius(self.style.radius),
        );

        frame.text_left_centered(
            self.layer,
            self.rect.inset(crate::UiEdgeInsets::symmetric(12.0, 0.0)),
            self.label,
            (self.rect.height * 0.34).clamp(12.0, 18.0),
            UiColor::WHITE,
        );

        frame.text_centered(
            self.layer,
            UiRect::new(
                self.rect.right() - 32.0,
                self.rect.y,
                24.0,
                self.rect.height,
            ),
            "⌄",
            (self.rect.height * 0.38).clamp(12.0, 20.0),
            UiColor::WHITE,
        );

        response.clicked
    }
}

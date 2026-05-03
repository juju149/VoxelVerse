use crate::{UiFrame, UiInput, UiLayer, UiRect, UiSearchStyle, UiSurface, UiWidgetId};

#[derive(Debug, Clone)]
pub struct UiSearchField {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub value: String,
    pub placeholder: String,
    pub style: UiSearchStyle,
    pub layer: UiLayer,
}

impl UiSearchField {
    pub fn new(
        id: UiWidgetId,
        rect: UiRect,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        style: UiSearchStyle,
    ) -> Self {
        Self {
            id,
            rect,
            value: value.into(),
            placeholder: placeholder.into(),
            style,
            layer: UiLayer::Menu,
        }
    }

    pub fn draw(self, frame: &mut UiFrame, _input: &UiInput, focused: bool) {
        let value_empty = self.value.is_empty();

        let border_color = if focused {
            self.style.border.color.lighten(0.22)
        } else {
            self.style.border.color
        };

        frame.surface(
            self.layer,
            self.rect,
            UiSurface::new(self.style.background)
                .border(border_color, self.style.border.width)
                .radius(self.style.radius),
        );

        let text = if value_empty {
            self.placeholder
        } else {
            self.value
        };

        let color = if value_empty {
            self.style.placeholder
        } else {
            self.style.text
        };

        frame.text_left_centered(
            self.layer,
            self.rect.inset(crate::UiEdgeInsets::symmetric(12.0, 0.0)),
            text,
            (self.rect.height * 0.34).clamp(12.0, 18.0),
            color,
        );
    }
}

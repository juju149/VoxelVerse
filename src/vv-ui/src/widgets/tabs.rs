use crate::{
    UiFrame, UiInput, UiLayer, UiMouseButton, UiRect, UiTabsStyle, UiTextAlign, UiWidgetId,
};

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
        if self.tabs.is_empty() {
            return None;
        }

        let gap = 6.0;
        let total_gap = gap * self.tabs.len().saturating_sub(1) as f32;
        let tab_w = ((self.rect.width - total_gap) / self.tabs.len() as f32).max(0.0);

        let mut clicked = None;

        for (index, tab) in self.tabs.iter().enumerate() {
            let rect = UiRect::new(
                self.rect.x + index as f32 * (tab_w + gap),
                self.rect.y,
                tab_w,
                self.rect.height,
            );

            let hovered = input
                .pointer_position
                .map(|point| rect.contains(point))
                .unwrap_or(false);

            let is_active = tab.id == self.active;
            let background = if is_active {
                self.style.active_background
            } else {
                self.style.background
            };

            frame.rounded_rect(
                self.layer,
                rect,
                background,
                self.style.radius,
                self.style.border,
                crate::UiShadow::NONE,
            );

            frame.text_aligned(
                self.layer,
                rect,
                tab.label.clone(),
                (rect.height * 0.32).clamp(11.0, 16.0),
                if is_active {
                    self.style.active_text
                } else {
                    self.style.text
                },
                UiTextAlign::Center,
            );

            if hovered && input.pointer_released(UiMouseButton::Primary) {
                clicked = Some(tab.id);
            }
        }

        clicked
    }
}

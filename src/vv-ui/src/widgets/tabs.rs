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

            let response = UiResponse::from_input(tab.id, rect, input, None, false);
            let is_active = tab.id == self.active;

            let background = if is_active {
                self.style.active_background
            } else {
                self.style.background
            };

            frame.surface(
                self.layer,
                rect,
                UiSurface::new(background)
                    .border(self.style.border.color, self.style.border.width)
                    .radius(self.style.radius),
            );

            frame.text_centered(
                self.layer,
                rect,
                tab.label.clone(),
                (rect.height * 0.32).clamp(11.0, 16.0),
                if is_active {
                    self.style.active_text
                } else {
                    self.style.text
                },
            );

            if response.clicked {
                clicked = Some(tab.id);
            }
        }

        clicked
    }
}

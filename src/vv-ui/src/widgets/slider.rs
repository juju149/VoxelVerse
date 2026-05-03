use crate::{
    UiFrame, UiInput, UiInteraction, UiLayer, UiMouseButton, UiRect, UiSliderStyle, UiWidgetId,
};

#[derive(Debug, Clone, Copy)]
pub struct UiSlider {
    pub id: UiWidgetId,
    pub rect: UiRect,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub style: UiSliderStyle,
    pub layer: UiLayer,
}

impl UiSlider {
    pub fn new(
        id: UiWidgetId,
        rect: UiRect,
        value: f32,
        min: f32,
        max: f32,
        style: UiSliderStyle,
    ) -> Self {
        Self {
            id,
            rect,
            value,
            min,
            max,
            style,
            layer: UiLayer::Menu,
        }
    }

    pub fn draw(
        self,
        frame: &mut UiFrame,
        input: &UiInput,
        active: Option<UiWidgetId>,
    ) -> (f32, bool) {
        let interaction = UiInteraction::from_input(self.id, self.rect, input, active);
        let mut changed = false;
        let mut value = self.value.clamp(self.min, self.max);

        if interaction.hovered && input.pointer_pressed(UiMouseButton::Primary) {
            if let Some(point) = input.pointer_position {
                let t = ((point.x - self.rect.x) / self.rect.width.max(1.0)).clamp(0.0, 1.0);
                value = self.min + (self.max - self.min) * t;
                changed = true;
            }
        }

        let t = if (self.max - self.min).abs() <= f32::EPSILON {
            0.0
        } else {
            ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        };

        let track_h = (self.rect.height * 0.22).max(4.0);
        let track_rect = UiRect::new(
            self.rect.x,
            self.rect.y + (self.rect.height - track_h) * 0.5,
            self.rect.width,
            track_h,
        );

        frame.rounded_rect(
            self.layer,
            track_rect,
            self.style.track,
            self.style.radius,
            self.style.border,
            crate::UiShadow::NONE,
        );

        frame.rounded_rect(
            self.layer,
            UiRect::new(
                track_rect.x,
                track_rect.y,
                track_rect.width * t,
                track_rect.height,
            ),
            self.style.fill,
            self.style.radius,
            crate::UiBorder::NONE,
            crate::UiShadow::NONE,
        );

        let thumb_size = self.rect.height.min(22.0).max(12.0);
        let thumb_rect = UiRect::new(
            self.rect.x + self.rect.width * t - thumb_size * 0.5,
            self.rect.y + (self.rect.height - thumb_size) * 0.5,
            thumb_size,
            thumb_size,
        );

        frame.rounded_rect(
            self.layer,
            thumb_rect,
            self.style.thumb,
            self.style.radius,
            crate::UiBorder::NONE,
            crate::UiShadow::NONE,
        );

        (value, changed)
    }
}

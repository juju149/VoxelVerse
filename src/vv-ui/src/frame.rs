use crate::surface::UiSurface;
use crate::text::{vertical_text_rect, UiTextVAlign};
use crate::{
    UiBorder, UiColor, UiCommand, UiGradient, UiIconId, UiImageId, UiLayer, UiRect, UiShadow,
    UiTextAlign, UiTextCommand, UiTextOverflow, UiTextStyleId,
};

#[derive(Debug, Clone)]
pub struct UiFrame {
    screen: UiRect,
    commands: Vec<UiCommand>,
}

impl UiFrame {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            screen: UiRect::new(0.0, 0.0, width.max(0.0), height.max(0.0)),
            commands: Vec::new(),
        }
    }

    pub fn screen(&self) -> UiRect {
        self.screen
    }

    pub fn commands(&self) -> &[UiCommand] {
        &self.commands
    }

    pub fn into_commands(self) -> Vec<UiCommand> {
        self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn push(&mut self, command: UiCommand) {
        self.commands.push(command);
    }

    pub fn rect(&mut self, layer: UiLayer, rect: UiRect, color: UiColor) {
        self.push(UiCommand::Rect {
            layer,
            rect,
            color,
            radius: 0.0,
            border: UiBorder::NONE,
            shadow: UiShadow::NONE,
        });
    }

    pub fn rounded_rect(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        color: UiColor,
        radius: f32,
        border: UiBorder,
        shadow: UiShadow,
    ) {
        self.push(UiCommand::Rect {
            layer,
            rect,
            color,
            radius,
            border,
            shadow,
        });
    }

    pub fn bordered_rounded_rect(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        fill: UiColor,
        border: UiColor,
        border_width: f32,
        radius: f32,
    ) {
        UiSurface::new(fill)
            .border(border, border_width)
            .radius(radius)
            .draw(self, layer, rect);
    }

    pub fn surface(&mut self, layer: UiLayer, rect: UiRect, surface: UiSurface) {
        surface.draw(self, layer, rect);
    }

    pub fn gradient_rect(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        gradient: UiGradient,
        radius: f32,
        border: UiBorder,
        shadow: UiShadow,
    ) {
        self.push(UiCommand::GradientRect {
            layer,
            rect,
            gradient,
            radius,
            border,
            shadow,
        });
    }

    pub fn image(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        image: UiImageId,
        tint: UiColor,
        radius: f32,
    ) {
        self.push(UiCommand::Image {
            layer,
            rect,
            image,
            tint,
            radius,
        });
    }

    pub fn icon(&mut self, layer: UiLayer, rect: UiRect, icon: UiIconId, color: UiColor) {
        self.push(UiCommand::Icon {
            layer,
            rect,
            icon,
            color,
        });
    }

    pub fn text(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        text: impl Into<String>,
        size: f32,
        color: UiColor,
    ) {
        self.push(UiCommand::Text {
            layer,
            command: UiTextCommand {
                rect,
                text: text.into(),
                size,
                color,
                align: UiTextAlign::Left,
                overflow: UiTextOverflow::Clip,
                style_id: None,
            },
        });
    }

    pub fn text_aligned(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        text: impl Into<String>,
        size: f32,
        color: UiColor,
        align: UiTextAlign,
    ) {
        self.push(UiCommand::Text {
            layer,
            command: UiTextCommand {
                rect,
                text: text.into(),
                size,
                color,
                align,
                overflow: UiTextOverflow::Clip,
                style_id: None,
            },
        });
    }

    pub fn text_in_rect(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        text: impl Into<String>,
        size: f32,
        color: UiColor,
        align: UiTextAlign,
        vertical_align: UiTextVAlign,
    ) {
        self.text_aligned(
            layer,
            vertical_text_rect(rect, size, vertical_align),
            text,
            size,
            color,
            align,
        );
    }

    pub fn text_centered(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        text: impl Into<String>,
        size: f32,
        color: UiColor,
    ) {
        self.text_in_rect(
            layer,
            rect,
            text,
            size,
            color,
            UiTextAlign::Center,
            UiTextVAlign::Center,
        );
    }

    pub fn text_left_centered(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        text: impl Into<String>,
        size: f32,
        color: UiColor,
    ) {
        self.text_in_rect(
            layer,
            rect,
            text,
            size,
            color,
            UiTextAlign::Left,
            UiTextVAlign::Center,
        );
    }

    pub fn text_right_centered(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        text: impl Into<String>,
        size: f32,
        color: UiColor,
    ) {
        self.text_in_rect(
            layer,
            rect,
            text,
            size,
            color,
            UiTextAlign::Right,
            UiTextVAlign::Center,
        );
    }

    pub fn styled_text(
        &mut self,
        layer: UiLayer,
        rect: UiRect,
        text: impl Into<String>,
        size: f32,
        color: UiColor,
        style_id: UiTextStyleId,
    ) {
        self.push(UiCommand::Text {
            layer,
            command: UiTextCommand {
                rect,
                text: text.into(),
                size,
                color,
                align: UiTextAlign::Left,
                overflow: UiTextOverflow::Clip,
                style_id: Some(style_id),
            },
        });
    }

    pub fn clip_start(&mut self, layer: UiLayer, rect: UiRect) {
        self.push(UiCommand::ClipStart { layer, rect });
    }

    pub fn clip_end(&mut self, layer: UiLayer) {
        self.push(UiCommand::ClipEnd { layer });
    }
}

use vv_ui::{UiBorder, UiColor, UiFrame, UiLayer, UiRect, UiShadow};

use crate::GameplayUiContext;

#[derive(Debug, Default, Clone, Copy)]
pub struct ConsoleScreen;

impl ConsoleScreen {
    pub fn draw(frame: &mut UiFrame, ctx: &GameplayUiContext<'_>) {
        let console = ctx.console;

        if console.height_fraction <= 0.001 {
            return;
        }

        let console_h = (ctx.screen_height / 2.0) * console.height_fraction;

        frame.rounded_rect(
            UiLayer::Popup,
            UiRect::new(0.0, 0.0, ctx.screen_width, console_h),
            UiColor::rgba(0.04, 0.045, 0.07, 0.92),
            0.0,
            UiBorder::NONE,
            UiShadow::new(0.0, 14.0, 28.0, 0.0, UiColor::rgba(0.0, 0.0, 0.0, 0.35)),
        );

        let start_y = console_h - 44.0;
        let line_h = 20.0;

        for (i, (line, color)) in console.history.iter().rev().enumerate() {
            let y = start_y - i as f32 * line_h;

            if y < 8.0 {
                break;
            }

            frame.text(
                UiLayer::Popup,
                UiRect::new(12.0, y, ctx.screen_width - 24.0, line_h),
                line,
                16.0,
                UiColor::rgb(color[0], color[1], color[2]),
            );
        }

        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let cursor = if (ms / 500) % 2 == 0 { "_" } else { " " };

        frame.text(
            UiLayer::Popup,
            UiRect::new(12.0, console_h - 24.0, ctx.screen_width - 24.0, 22.0),
            format!("> {}{}", console.input_buffer, cursor),
            16.0,
            ctx.theme.accent,
        );
    }
}

use vv_gameplay::{Console, PlayerGameplayState};
use vv_input::Controller;
use vv_registry::CompiledContent;
use vv_ui::{UiFrame, UiTheme};

use vv_interface::{build_gameplay_ui_frame, GameplayUiContext};

use super::Renderer;

impl<'a> Renderer<'a> {
    pub(super) fn build_renderer_ui_frame(
        &self,
        controller: &Controller,
        console: &Console,
        gameplay: &PlayerGameplayState,
        content: &CompiledContent,
    ) -> UiFrame {
        let theme = UiTheme::default();

        build_gameplay_ui_frame(GameplayUiContext {
            screen_width: self.config.width as f32,
            screen_height: self.config.height as f32,
            first_person: controller.first_person,
            current_fps: self.current_fps,
            console,
            gameplay,
            content,
            theme: &theme,
        })
    }
}

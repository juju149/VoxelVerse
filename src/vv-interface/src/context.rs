use vv_gameplay::{Console, PlayerGameplayState};
use vv_registry::CompiledContent;
use vv_ui::UiTheme;

#[derive(Clone, Copy)]
pub struct GameplayUiContext<'a> {
    pub screen_width: f32,
    pub screen_height: f32,
    pub first_person: bool,
    pub current_fps: u32,
    pub console: &'a Console,
    pub gameplay: &'a PlayerGameplayState,
    pub content: &'a CompiledContent,
    pub theme: &'a UiTheme,
}

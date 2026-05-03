use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MoveIntent {
    pub direction: Vec3,
    pub jump: bool,
    pub sprint: bool,
    pub fly_toggle: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LookIntent {
    pub mouse_delta: Vec2,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameplayIntent {
    pub mine_held: bool,
    pub place_pressed: bool,
    pub hotbar_delta: i32,
    pub hotbar_slot: Option<usize>,
    pub toggle_inventory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiPointerIntent {
    PrimaryPressed(Vec2),
    PrimaryReleased(Vec2),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputFrameIntent {
    pub movement: MoveIntent,
    pub look: LookIntent,
    pub gameplay: GameplayIntent,
    pub ui_pointer_events: Vec<UiPointerIntent>,
}

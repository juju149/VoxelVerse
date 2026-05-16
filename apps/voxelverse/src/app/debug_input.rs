use crate::app::action_result::FrameCommand;
use vv_gameplay::PlanetResizeIntent;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Decode a key press into zero or more dev/debug `FrameCommand`s.
///
/// Returns an empty vec for every key that is not a dev hotkey.
/// The caller is responsible for checking `dev_mode` before calling this function.
pub(super) fn decode_dev_key(key: PhysicalKey) -> Vec<FrameCommand> {
    if let Some(cmd) = planet_resize_command(key) {
        return vec![cmd, FrameCommand::ForceReloadWorld, FrameCommand::Redraw];
    }
    if let Some(cmd) = quality_command(key) {
        return vec![cmd];
    }
    vec![]
}

fn planet_resize_command(key: PhysicalKey) -> Option<FrameCommand> {
    let intent = match key {
        PhysicalKey::Code(KeyCode::BracketRight) => PlanetResizeIntent::Grow,
        PhysicalKey::Code(KeyCode::BracketLeft) => PlanetResizeIntent::Shrink,
        _ => return None,
    };
    Some(FrameCommand::ResizePlanet(intent))
}

fn quality_command(key: PhysicalKey) -> Option<FrameCommand> {
    match key {
        PhysicalKey::Code(KeyCode::F2) => Some(FrameCommand::ToggleDebugPage),
        PhysicalKey::Code(KeyCode::F3) | PhysicalKey::Code(KeyCode::Fn) => {
            Some(FrameCommand::ToggleColorOnlyMode)
        }
        PhysicalKey::Code(KeyCode::F5) => Some(FrameCommand::ToggleTriplanarGrain),
        PhysicalKey::Code(KeyCode::F6) => Some(FrameCommand::CyclePcfQuality),
        _ => None,
    }
}

use winit::window::{CursorGrabMode, Window};

pub(super) fn grab_cursor(window: &Window) {
    let _ = window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
    window.set_cursor_visible(false);
}

pub(super) fn release_cursor(window: &Window) {
    let _ = window.set_cursor_grab(CursorGrabMode::None);
    window.set_cursor_visible(true);
}

pub(super) fn sync_cursor_mode(
    window: &Window,
    first_person: bool,
    ui_captures_input: bool,
    cursor_grabbed: &mut bool,
) {
    let should_grab = first_person && !ui_captures_input;
    if should_grab == *cursor_grabbed {
        return;
    }

    *cursor_grabbed = should_grab;
    if should_grab {
        grab_cursor(window);
    } else {
        release_cursor(window);
    }
}

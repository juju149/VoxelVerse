use vv_gameplay::ControllerFrameInput;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Accumulates raw winit input events within a frame and converts them to
/// `ControllerFrameInput` on flush.
///
/// The accumulator owns the authoritative key-hold state. One-shot toggles and
/// per-frame deltas are cleared after each flush.
pub(super) struct InputAccumulator {
    /// WASD + Space held state: [forward(W), left(A), back(S), right(D), jump(Space)]
    keys: [bool; 5],
    sprint: bool,
    /// Raw mouse delta accumulated this frame (DeviceEvent::MouseMotion).
    raw_mouse_delta: (f32, f32),
    /// Latest cursor screen position (pixels).
    cursor_pos: Option<(f32, f32)>,
    /// Previous cursor position for computing orbit delta.
    prev_cursor_pos: Option<(f32, f32)>,
    /// Cursor position delta for orbit camera (non-first-person).
    cursor_delta: (f32, f32),
    /// Scroll wheel delta accumulated this frame.
    scroll_delta: f32,
    /// Middle mouse button held (orbit mode).
    orbit_active: bool,
    /// One-shot: K key pressed this frame — toggles first_person.
    toggle_camera_mode: bool,
    /// One-shot: F key pressed this frame — toggles fly_mode.
    toggle_fly: bool,
}

impl InputAccumulator {
    pub(super) fn new() -> Self {
        Self {
            keys: [false; 5],
            sprint: false,
            raw_mouse_delta: (0.0, 0.0),
            cursor_pos: None,
            prev_cursor_pos: None,
            cursor_delta: (0.0, 0.0),
            scroll_delta: 0.0,
            orbit_active: false,
            toggle_camera_mode: false,
            toggle_fly: false,
        }
    }

    /// Push a raw winit window event into the accumulator.
    /// Only consumes movement/camera-relevant events — all other events are ignored.
    pub(super) fn push_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = (position.x as f32, position.y as f32);
                if let Some(prev) = self.prev_cursor_pos {
                    self.cursor_delta = (new_pos.0 - prev.0, new_pos.1 - prev.1);
                }
                self.prev_cursor_pos = Some(new_pos);
                self.cursor_pos = Some(new_pos);
            }

            WindowEvent::MouseInput {
                state,
                button: MouseButton::Middle,
                ..
            } => {
                self.orbit_active = *state == ElementState::Pressed;
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
                };
                self.scroll_delta += y;
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.keys[0] = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.keys[1] = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.keys[2] = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.keys[3] = pressed,
                    PhysicalKey::Code(KeyCode::Space) => self.keys[4] = pressed,
                    PhysicalKey::Code(KeyCode::ControlLeft) => self.sprint = pressed,
                    PhysicalKey::Code(KeyCode::KeyK) if pressed => {
                        self.toggle_camera_mode = true;
                    }
                    PhysicalKey::Code(KeyCode::KeyF) if pressed => {
                        self.toggle_fly = true;
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    /// Push raw mouse motion from a device event (for first-person look).
    pub(super) fn push_mouse_motion(&mut self, delta: (f64, f64)) {
        self.raw_mouse_delta.0 += delta.0 as f32;
        self.raw_mouse_delta.1 += delta.1 as f32;
    }

    /// Produce a `ControllerFrameInput` snapshot and reset per-frame state.
    /// Key hold states are preserved across frames.
    pub(super) fn flush(&mut self) -> ControllerFrameInput {
        let input = ControllerFrameInput {
            keys: self.keys,
            sprint: self.sprint,
            raw_mouse_delta: self.raw_mouse_delta,
            cursor_pos: self.cursor_pos,
            cursor_delta: self.cursor_delta,
            scroll_delta: self.scroll_delta,
            orbit_active: self.orbit_active,
            toggle_camera_mode: self.toggle_camera_mode,
            toggle_fly: self.toggle_fly,
        };
        self.raw_mouse_delta = (0.0, 0.0);
        self.cursor_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
        self.toggle_camera_mode = false;
        self.toggle_fly = false;
        input
    }
}

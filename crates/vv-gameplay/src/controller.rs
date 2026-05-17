//engine controller

use crate::{Player, PlayerInput};
use glam::{Mat4, Vec2, Vec3};
use vv_math::Ray;
use vv_physics::Physics;
use vv_voxel::VoxelCoord;

/// Pure input state for one frame, produced by the app layer from winit events.
/// Contains no winit types — safe to keep in vv-gameplay.
pub struct ControllerFrameInput {
    /// Current held state: [forward(W), left(A), back(S), right(D), jump(Space)]
    pub keys: [bool; 5],
    /// LeftCtrl sprint held state.
    pub sprint: bool,
    /// Raw mouse delta from device events (accumulated this frame). First-person only.
    pub raw_mouse_delta: (f32, f32),
    /// Screen-space cursor position (pixels), updated from WindowEvent::CursorMoved.
    pub cursor_pos: Option<(f32, f32)>,
    /// Cursor position delta for orbit camera (non-first-person mode).
    pub cursor_delta: (f32, f32),
    /// Scroll wheel delta this frame (positive = zoom out). Non-first-person only.
    pub scroll_delta: f32,
    /// Middle mouse button held (orbit mode).
    pub orbit_active: bool,
    /// One-shot: K key was pressed this frame — toggles first_person / cam_dist.
    pub toggle_camera_mode: bool,
    /// One-shot: F key was pressed this frame — toggles fly_mode (first_person only).
    pub toggle_fly: bool,
}

pub struct Controller {
    pub cam_dist: f32,

    /// Screen-space cursor position (pixels), used for orbit ray casting.
    pub mouse_pos: Vec2,
    pub mouse_delta: (f32, f32),
    pub is_orbiting: bool,
    pub fly_mode: bool,
    pub sprint: bool,
    pub cursor_id: Option<VoxelCoord>,

    pub first_person: bool,

    keys: [bool; 5], // W, A, S, D, Space
}

impl Controller {
    pub fn new() -> Self {
        Self {
            cam_dist: 200.0,
            mouse_pos: Vec2::ZERO,
            mouse_delta: (0.0, 0.0),
            is_orbiting: false,
            cursor_id: None,
            fly_mode: false,
            sprint: false,
            first_person: true,
            keys: [false; 5],
        }
    }

    /// Apply accumulated frame input produced by the app-layer `InputAccumulator`.
    /// Replaces the old winit-aware `process_events` + `process_mouse_motion` methods.
    pub fn apply_input(&mut self, input: ControllerFrameInput) {
        self.keys = input.keys;
        self.sprint = input.sprint;
        self.is_orbiting = input.orbit_active;

        if let Some(pos) = input.cursor_pos {
            self.mouse_pos = Vec2::new(pos.0, pos.1);
        }

        if self.first_person {
            self.mouse_delta.0 += input.raw_mouse_delta.0;
            self.mouse_delta.1 += input.raw_mouse_delta.1;
        } else {
            if input.cursor_delta != (0.0, 0.0) {
                self.mouse_delta = input.cursor_delta;
            }
            if input.scroll_delta != 0.0 {
                self.cam_dist = (self.cam_dist - input.scroll_delta * 50.0).clamp(10.0, 10000.0);
            }
        }

        if input.toggle_camera_mode {
            self.first_person = !self.first_person;
            self.cam_dist = if self.first_person { 40.0 } else { 100.0 };
        }

        if input.toggle_fly && self.first_person {
            self.fly_mode = !self.fly_mode;
            println!("Fly Mode: {}", self.fly_mode);
        }
    }

    pub fn clear_transient_input(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn sample_player_input(&mut self) -> PlayerInput {
        let mut input = Vec3::ZERO;
        if self.keys[0] {
            input.z -= 1.0;
        } // W
        if self.keys[1] {
            input.x -= 1.0;
        } // A
        if self.keys[2] {
            input.z += 1.0;
        } // S
        if self.keys[3] {
            input.x += 1.0;
        } // D
        let jump = self.keys[4]; // space

        let rotation_delta = if self.first_person {
            self.mouse_delta
        } else {
            (0.0, 0.0)
        };

        let player_input = PlayerInput {
            movement: input,
            jump,
            mouse_delta: rotation_delta,
            flying: self.fly_mode,
            sprint: self.sprint,
        };

        self.mouse_delta = (0.0, 0.0);
        player_input
    }

    pub fn get_camera_pos(&self, player: &Player) -> Vec3 {
        if self.first_person {
            // first person: Camera is at player position + eye height
            player.position + (Physics::get_up_vector(player.position) * 1.6)
        } else {
            let up = Physics::get_up_vector(player.position);
            player.position + (up * self.cam_dist)
        }
    }

    pub fn get_matrix(&self, player: &Player, width: f32, height: f32) -> Mat4 {
        // use 45 degrees in Orbit mode for less distortion.
        let fov_degrees: f32 = if self.first_person { 80.0 } else { 45.0 };

        // far plane increased to 20,000 for massive zoom out
        let proj = Mat4::perspective_rh(fov_degrees.to_radians(), width / height, 0.1, 20000.0);

        let view = if self.first_person {
            player.get_view_matrix()
        } else {
            let up = Physics::get_up_vector(player.position);
            let cam_pos = player.position + (up * self.cam_dist);
            let target = player.position;

            let player_forward = player.rotation * Vec3::NEG_Z;

            Mat4::look_at_rh(cam_pos, target, player_forward)
        };

        proj * view
    }

    pub fn view_ray(&self, player: &Player, width: f32, height: f32) -> Ray {
        let mvp = self.get_matrix(player, width, height);
        let inv = mvp.inverse();

        let (ndc_x, ndc_y) = if self.first_person {
            (0.0, 0.0)
        } else {
            (
                (2.0 * self.mouse_pos.x / width) - 1.0,
                1.0 - (2.0 * self.mouse_pos.y / height),
            )
        };

        Ray::from_clip_space(inv, ndc_x, ndc_y)
    }

    pub fn interaction_reach(&self) -> f32 {
        if self.first_person {
            8.0
        } else {
            self.cam_dist + 100.0
        }
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

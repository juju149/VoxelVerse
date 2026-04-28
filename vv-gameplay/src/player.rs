use glam::{Vec3, Quat, Mat4};
use vv_config::PlayerConfig;
use vv_physics::Physics;
use vv_world_runtime::PlanetData;

/// Runtime state of the local player.
pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    pub cam_pitch: f32,
    pub grounded:  bool,
    pub debug_mode: bool,

    // --- Tunable at runtime (e.g. via the dev console) ---
    pub move_speed: f32,
    pub jump_force: f32,
    pub mouse_sens: f32,
}

impl Player {
    pub fn new(cfg: &PlayerConfig) -> Self {
        Self {
            position:   Vec3::new(0.0, 200.0, 0.0),
            velocity:   Vec3::ZERO,
            rotation:   Quat::IDENTITY,
            cam_pitch:  0.0,
            grounded:   false,
            debug_mode: false,
            move_speed: cfg.move_speed,
            jump_force: cfg.jump_force,
            mouse_sens: cfg.mouse_sensitivity,
        }
    }

    pub fn spawn(&mut self, pos: Vec3) {
        self.position = pos;
        self.velocity = Vec3::ZERO;
        self.grounded = false;
        let up = Physics::get_up_vector(self.position);
        self.rotation = Quat::from_rotation_arc(Vec3::Y, up);
    }

    /// Advance the player for one frame. Requires the physics solver and
    /// current planet state.
    pub fn update(
        &mut self,
        dt: f32,
        planet: &PlanetData,
        physics: &Physics,
        input: Vec3,
        jump: bool,
        mouse_delta: (f32, f32),
        flying: bool,
        sprint: bool,
    ) {
        let up = Physics::get_up_vector(self.position);

        // Yaw
        if mouse_delta.0.abs() > 0.001 {
            let yaw = Quat::from_axis_angle(up, -mouse_delta.0 * self.mouse_sens);
            self.rotation = yaw * self.rotation;
        }
        // Pitch
        if mouse_delta.1.abs() > 0.001 {
            self.cam_pitch = (self.cam_pitch - mouse_delta.1 * self.mouse_sens).clamp(-1.5, 1.5);
        }

        let speed = if sprint {
            if flying { self.move_speed * 10.0 } else { self.move_speed * 2.0 }
        } else {
            self.move_speed
        };

        if flying {
            if input.length() > 0.01 {
                let pitch_rot = Quat::from_axis_angle(Vec3::X, self.cam_pitch);
                let dir = self.rotation * pitch_rot * Vec3::new(input.normalize().x, 0.0, input.normalize().z);
                self.velocity = dir * speed;
            } else {
                self.velocity = Vec3::ZERO;
            }
        } else {
            if input.length() > 0.01 {
                let move_dir = self.rotation * Vec3::new(input.normalize().x, 0.0, input.normalize().z);
                let curr_horz = self.velocity - up * self.velocity.dot(up);
                let target = move_dir * speed;
                let accel = 25.0f32;
                let new_horz = curr_horz + (target - curr_horz).clamp_length_max(accel * dt);
                self.velocity = new_horz + up * self.velocity.dot(up);
            } else {
                let horz = self.velocity - up * self.velocity.dot(up);
                let friction = if self.grounded { 15.0f32 } else { 0.5f32 };
                let reduced = horz * (1.0 - friction * dt).max(0.0);
                self.velocity = reduced + up * self.velocity.dot(up);
            }
        }

        if jump && self.grounded && !flying {
            self.velocity += up * self.jump_force;
            self.grounded = false;
        }

        if !flying {
            self.velocity -= up * physics.gravity * dt;
        }

        let (new_pos, new_vel, grounded) = physics.solve_movement(
            self.position, self.velocity, dt, planet, flying,
        );
        self.position = new_pos;
        self.velocity = new_vel;
        self.grounded = grounded;
        self.rotation = Physics::align_to_planet(self.rotation, up);
    }

    pub fn get_view_matrix(&self, physics: &Physics) -> Mat4 {
        let up = Physics::get_up_vector(self.position);
        let cam_pos = self.position + up * physics.eye_height;
        let pitch_rot = Quat::from_axis_angle(Vec3::X, self.cam_pitch);
        let forward = (self.rotation * pitch_rot) * Vec3::NEG_Z;
        Mat4::look_at_rh(cam_pos, cam_pos + forward, up)
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation)
    }
}

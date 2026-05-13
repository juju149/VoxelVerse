use glam::{Mat4, Quat, Vec3};
use vv_physics::Physics;
use vv_world::PlanetData;

pub struct Player {
    // State
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    pub cam_pitch: f32,
    pub grounded: bool,
    pub debug_mode: bool,

    // Configuration
    pub move_speed: f32,
    pub jump_force: f32,
    pub mouse_sens: f32,
}

pub struct PlayerInput {
    pub movement: Vec3,
    pub jump: bool,
    pub mouse_delta: (f32, f32),
    pub flying: bool,
    pub sprint: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 200.0, 0.0),
            velocity: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            cam_pitch: 0.0,
            grounded: false,
            debug_mode: false,
            move_speed: 5.0,
            jump_force: 8.0,
            mouse_sens: 0.002,
        }
    }

    pub fn spawn(&mut self, pos: Vec3) {
        self.position = pos;
        self.velocity = Vec3::ZERO;
        self.grounded = false;
        let up = Physics::get_up_vector(self.position);
        self.rotation = Quat::from_rotation_arc(Vec3::Y, up);
    }

    pub fn update(&mut self, dt: f32, planet: &PlanetData, input: PlayerInput) {
        let up = Physics::get_up_vector(self.position);

        // --- ROTATION (YAW) ---
        if input.mouse_delta.0.abs() > 0.001 {
            let yaw_delta = -input.mouse_delta.0 * self.mouse_sens;
            let yaw_rot = Quat::from_axis_angle(up, yaw_delta);
            self.rotation = yaw_rot * self.rotation;
        }

        // --- PITCH ---
        if input.mouse_delta.1.abs() > 0.001 {
            self.cam_pitch =
                (self.cam_pitch - input.mouse_delta.1 * self.mouse_sens).clamp(-1.5, 1.5);
        }

        let effective_speed = if input.sprint {
            if input.flying {
                self.move_speed * 100.0
            } else {
                self.move_speed * 2.0
            }
        } else {
            self.move_speed
        };

        // --- MOVEMENT INPUT ---
        if input.flying {
            if input.movement.length() > 0.01 {
                let input_normalized = input.movement.normalize();
                let pitch_rot = Quat::from_axis_angle(Vec3::X, self.cam_pitch);
                let fly_dir = self.rotation
                    * pitch_rot
                    * Vec3::new(input_normalized.x, 0.0, input_normalized.z);
                // self.velocity = fly_dir * 1.5;
                self.velocity = fly_dir * effective_speed;
            } else {
                self.velocity = Vec3::ZERO;
            }
        } else {
            // walk
            if input.movement.length() > 0.01 {
                let input_normalized = input.movement.normalize();
                let move_dir =
                    self.rotation * Vec3::new(input_normalized.x, 0.0, input_normalized.z);
                let current_horz = self.velocity - (up * self.velocity.dot(up));

                let target_horz = move_dir * effective_speed;

                // acceleration
                let accel = 25.0;
                let new_horz =
                    current_horz + (target_horz - current_horz).clamp_length_max(accel * dt);

                self.velocity = new_horz + (up * self.velocity.dot(up));
            } else {
                let horz_vel = self.velocity - (up * self.velocity.dot(up));

                let friction = if self.grounded { 15.0 } else { 0.5 };

                let reduced = horz_vel * (1.0 - friction * dt).max(0.0);
                self.velocity = reduced + (up * self.velocity.dot(up));
            }
        }

        // --- JUMP ---
        if input.jump && self.grounded && !input.flying {
            self.velocity += up * self.jump_force;
            self.grounded = false;
        }

        // --- GRAVITY ---
        if !input.flying {
            self.velocity -= up * Physics::GRAVITY * dt;
        }

        // --- PHYSICS SOLVE ---
        let (new_pos, new_vel, grounded) =
            Physics::solve_movement(self.position, self.velocity, dt, planet, input.flying);

        self.position = new_pos;
        self.velocity = new_vel;
        self.grounded = grounded;

        // --- ALIGN TO SURFACE ---
        self.rotation = Physics::align_to_planet(self.rotation, up);
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let up = Physics::get_up_vector(self.position);
        let cam_pos = self.position + (up * Physics::EYE_HEIGHT);

        let pitch_rot = Quat::from_axis_angle(Vec3::X, self.cam_pitch);
        let final_rot = self.rotation * pitch_rot;

        let forward = final_rot * Vec3::NEG_Z;

        Mat4::look_at_rh(cam_pos, cam_pos + forward, up)
    }
}

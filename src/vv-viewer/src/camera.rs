/// Orbital camera for the viewer.
use glam::{Mat4, Vec3};

pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,   // radians
    pub pitch: f32, // radians
    pub width: f32,
    pub height: f32,
}

impl OrbitCamera {
    pub fn new_for_scene(block_count: u32, width: f32, height: f32) -> Self {
        let extent = block_count as f32 * 0.5; // 0.5 m per voxel
        Self {
            target: Vec3::new(extent * 0.5 - 0.25, 0.25, extent * 0.5 - 0.25),
            distance: (extent * 3.0 + 3.0).max(3.0),
            yaw: -std::f32::consts::FRAC_PI_4,
            pitch: 0.45,
            width,
            height,
        }
    }

    pub fn reset(&mut self, block_count: u32) {
        let extent = block_count as f32 * 0.5;
        self.target = Vec3::new(extent * 0.5 - 0.25, 0.25, extent * 0.5 - 0.25);
        self.distance = (extent * 3.0 + 3.0).max(3.0);
        self.yaw = -std::f32::consts::FRAC_PI_4;
        self.pitch = 0.45;
    }

    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + Vec3::new(x, y, z)
    }

    pub fn view_proj(&self) -> Mat4 {
        let eye = self.position();
        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        let aspect = self.width / self.height;
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect, 0.1, 500.0);
        proj * view
    }

    pub fn pan_orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.008;
        self.pitch = (self.pitch + dy * 0.008).clamp(0.05, std::f32::consts::FRAC_PI_2 - 0.05);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (1.0 - delta * 0.12)).clamp(0.5, 200.0);
    }

    /// Set the horizontal angle directly (used by turntable mode).
    pub fn set_azimuth(&mut self, radians: f32) {
        self.yaw = radians;
    }
}

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use vv_config::AtmosphereConfig;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AtmosphereUniform {
    pub sun_direction: [f32; 4],
    pub sun_color: [f32; 4],
    pub sky_color: [f32; 4],
    pub ground_ambient_color: [f32; 4],
    /// Cool tint applied to sun-facing surfaces in shadow (warm/cool separation).
    /// Field order must match the WGSL Atmosphere struct exactly.
    pub shadow_tint_color: [f32; 4],
    pub fog_color_density: [f32; 4],
    pub clear_color: [f32; 4],
}

impl AtmosphereUniform {
    pub fn from_config(config: &AtmosphereConfig) -> Self {
        let sun_direction = normalized_or_up(config.sun_direction);

        Self {
            sun_direction: [sun_direction.x, sun_direction.y, sun_direction.z, 0.0],
            sun_color: [
                config.sun_color[0],
                config.sun_color[1],
                config.sun_color[2],
                0.0,
            ],
            sky_color: [
                config.sky_color[0],
                config.sky_color[1],
                config.sky_color[2],
                0.0,
            ],
            ground_ambient_color: [
                config.ground_ambient_color[0],
                config.ground_ambient_color[1],
                config.ground_ambient_color[2],
                0.0,
            ],
            shadow_tint_color: [
                config.shadow_tint_color[0],
                config.shadow_tint_color[1],
                config.shadow_tint_color[2],
                0.0,
            ],
            fog_color_density: [
                config.fog_color[0],
                config.fog_color[1],
                config.fog_color[2],
                config.fog_density,
            ],
            clear_color: [
                config.clear_color[0] as f32,
                config.clear_color[1] as f32,
                config.clear_color[2] as f32,
                config.clear_color[3] as f32,
            ],
        }
    }

    pub fn sun_direction_vec3(self) -> Vec3 {
        Vec3::new(
            self.sun_direction[0],
            self.sun_direction[1],
            self.sun_direction[2],
        )
    }

    pub fn clear_color(self) -> wgpu::Color {
        wgpu::Color {
            r: self.clear_color[0] as f64,
            g: self.clear_color[1] as f64,
            b: self.clear_color[2] as f64,
            a: self.clear_color[3] as f64,
        }
    }
}

fn normalized_or_up(direction: [f32; 3]) -> Vec3 {
    let direction = Vec3::new(direction[0], direction[1], direction[2]);
    direction.try_normalize().unwrap_or(Vec3::Y)
}

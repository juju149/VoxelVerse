use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use vv_config::AtmosphereConfig;
use vv_planet::PlanetGeometry;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct AtmosphereUniform {
    pub sun_direction: [f32; 4],
    pub sun_color: [f32; 4],
    pub sky_color: [f32; 4],
    pub ground_ambient_color: [f32; 4],
    pub shadow_tint_color: [f32; 4],
    pub fog_color_density: [f32; 4],
    pub clear_color: [f32; 4],

    pub zenith_color: [f32; 4],
    pub horizon_glow_color: [f32; 4],
    pub moon_direction: [f32; 4],
    pub moon_color: [f32; 4],

    pub grading: [f32; 4],
    pub sky_params: [f32; 4],

    pub planet_center_radius: [f32; 4],
    pub atmosphere_params: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct PlanetAtmosphereParams {
    pub center: Vec3,
    pub radius_m: f32,
    pub atmosphere_height_m: f32,
    pub fade_start_m: f32,
    pub fade_end_m: f32,
    pub terminator_softness: f32,
}

impl PlanetAtmosphereParams {
    pub fn from_geometry(geometry: PlanetGeometry) -> Self {
        let radius_m = geometry.radius_m.max(1.0);

        let atmosphere_height_m = (radius_m * 0.080).clamp(650.0, 12_000.0);
        let fade_start_m = atmosphere_height_m * 0.42;
        let fade_end_m = atmosphere_height_m * 1.25;

        Self {
            center: Vec3::ZERO,
            radius_m,
            atmosphere_height_m,
            fade_start_m,
            fade_end_m,
            terminator_softness: 0.115,
        }
    }
}

impl AtmosphereUniform {
    pub fn from_config(config: &AtmosphereConfig) -> Self {
        let sun_direction = normalized_or(config.sun_direction, Vec3::Y);
        let moon_direction = normalized_or(config.moon_direction, -sun_direction);

        Self {
            sun_direction: dir4(sun_direction),
            sun_color: rgb4(config.sun_color),
            sky_color: rgb4(config.sky_color),
            ground_ambient_color: rgb4(config.ground_ambient_color),
            shadow_tint_color: rgb4(config.shadow_tint_color),
            fog_color_density: [
                config.fog_color[0],
                config.fog_color[1],
                config.fog_color[2],
                config.fog_density.max(0.0),
            ],
            clear_color: rgba64_to_f32(config.clear_color),

            zenith_color: rgb4(config.zenith_color),
            horizon_glow_color: rgb4(config.horizon_glow_color),
            moon_direction: dir4(moon_direction),
            moon_color: rgb4(config.moon_color),

            grading: [
                default_one(config.exposure),
                default_one(config.saturation),
                default_one(config.contrast),
                0.0,
            ],

            sky_params: [
                config.fog_start_m.max(0.0),
                default_or(config.sky_horizon_power, 0.72).max(0.05),
                config.star_strength.max(0.0),
                config.night_amount.clamp(0.0, 1.0),
            ],

            planet_center_radius: [
                config.planet_center[0],
                config.planet_center[1],
                config.planet_center[2],
                1.0,
            ],

            atmosphere_params: [
                config.atmosphere_height_m.max(1.0),
                config.atmosphere_fade_start_m.max(0.0),
                config
                    .atmosphere_fade_end_m
                    .max(config.atmosphere_fade_start_m + 1.0),
                config.terminator_softness.clamp(0.01, 0.45),
            ],
        }
    }

    pub fn with_planet_geometry(mut self, geometry: PlanetGeometry) -> Self {
        self.with_planet_atmosphere(PlanetAtmosphereParams::from_geometry(geometry))
    }

    pub fn with_planet_atmosphere(mut self, params: PlanetAtmosphereParams) -> Self {
        self.planet_center_radius = [
            params.center.x,
            params.center.y,
            params.center.z,
            params.radius_m.max(1.0),
        ];

        self.atmosphere_params = [
            params.atmosphere_height_m.max(1.0),
            params.fade_start_m.max(0.0),
            params.fade_end_m.max(params.fade_start_m + 1.0),
            params.terminator_softness.clamp(0.01, 0.45),
        ];

        self
    }

    pub fn sun_direction_vec3(self) -> Vec3 {
        Vec3::new(
            self.sun_direction[0],
            self.sun_direction[1],
            self.sun_direction[2],
        )
    }

    pub fn moon_direction_vec3(self) -> Vec3 {
        Vec3::new(
            self.moon_direction[0],
            self.moon_direction[1],
            self.moon_direction[2],
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

fn normalized_or(direction: [f32; 3], fallback: Vec3) -> Vec3 {
    Vec3::new(direction[0], direction[1], direction[2])
        .try_normalize()
        .unwrap_or(fallback)
}

fn dir4(direction: Vec3) -> [f32; 4] {
    [direction.x, direction.y, direction.z, 0.0]
}

fn rgb4(color: [f32; 3]) -> [f32; 4] {
    [color[0], color[1], color[2], 0.0]
}

fn rgba64_to_f32(color: [f64; 4]) -> [f32; 4] {
    [
        color[0] as f32,
        color[1] as f32,
        color[2] as f32,
        color[3] as f32,
    ]
}

fn default_one(value: f32) -> f32 {
    if value.abs() < 0.0001 {
        1.0
    } else {
        value
    }
}

fn default_or(value: f32, fallback: f32) -> f32 {
    if value.abs() < 0.0001 {
        fallback
    } else {
        value
    }
}

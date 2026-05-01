use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use vv_config::AtmosphereConfig;

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

    /// x = exposure
    /// y = saturation
    /// z = contrast
    /// w = reserved
    pub grading: [f32; 4],

    /// x = fog_start
    /// y = sky_horizon_power
    /// z = star_strength
    /// w = night_amount
    pub sky_params: [f32; 4],
}

impl AtmosphereUniform {
    /// Backward-compatible constructor.
    ///
    /// This keeps the old config-driven atmosphere working while filling the new
    /// Phase 1 fields with a strong daytime cinematic default.
    pub fn from_config(config: &AtmosphereConfig) -> Self {
        let sun_direction = normalized_or(config.sun_direction, Vec3::Y);
        let moon_direction = -sun_direction;

        Self {
            sun_direction: direction4(sun_direction),
            sun_color: rgb4(config.sun_color),
            sky_color: rgb4(config.sky_color),
            ground_ambient_color: rgb4(config.ground_ambient_color),
            shadow_tint_color: rgb4(config.shadow_tint_color),
            fog_color_density: [
                config.fog_color[0],
                config.fog_color[1],
                config.fog_color[2],
                config.fog_density,
            ],
            clear_color: rgba_from_config(config.clear_color),

            // New fields, tuned to look good even if AtmosphereConfig has not yet
            // been expanded.
            zenith_color: [0.08, 0.18, 0.52, 1.0],
            horizon_glow_color: [0.92, 0.78, 0.58, 1.0],
            moon_direction: direction4(moon_direction),
            moon_color: [0.50, 0.58, 0.82, 1.0],

            grading: [
                1.02, // exposure
                1.08, // saturation
                1.04, // contrast
                0.0,
            ],

            sky_params: [
                24.0, // fog_start
                1.65, // horizon_power
                0.0,  // star_strength
                0.0,  // night_amount
            ],
        }
    }

    /// Cinematic day/night constructor.
    ///
    /// `day01` is normalized:
    /// - 0.00 = midnight
    /// - 0.25 = dawn
    /// - 0.50 = noon
    /// - 0.75 = sunset
    ///
    /// This should become the preferred constructor for Phase 1.
    pub fn from_config_at_day_time(config: &AtmosphereConfig, day01: f32) -> Self {
        let preset = AtmospherePreset::for_day_time(day01);

        let sun_direction = sun_direction_for_day_time(day01);
        let moon_direction = moon_direction_for_day_time(day01);

        Self {
            sun_direction: direction4(sun_direction),
            sun_color: preset.sun_color,
            sky_color: preset.sky_color,
            ground_ambient_color: preset.ground_ambient_color,
            shadow_tint_color: preset.shadow_tint_color,
            fog_color_density: preset.fog_color_density,

            // Keep config clear color as a fallback source if you want engine-level
            // control. The cinematic preset is used as the visual default.
            clear_color: blend4(
                rgba_from_config(config.clear_color),
                preset.clear_color,
                0.85,
            ),

            zenith_color: preset.zenith_color,
            horizon_glow_color: preset.horizon_glow_color,
            moon_direction: direction4(moon_direction),
            moon_color: preset.moon_color,

            grading: [
                preset.exposure,
                preset.saturation,
                preset.contrast,
                0.0,
            ],

            sky_params: [
                preset.fog_start,
                preset.horizon_power,
                preset.star_strength,
                preset.night_amount,
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

#[derive(Debug, Clone, Copy)]
pub struct AtmospherePreset {
    pub sun_color: [f32; 4],
    pub sky_color: [f32; 4],
    pub ground_ambient_color: [f32; 4],
    pub shadow_tint_color: [f32; 4],
    pub fog_color_density: [f32; 4],
    pub clear_color: [f32; 4],
    pub zenith_color: [f32; 4],
    pub horizon_glow_color: [f32; 4],
    pub moon_color: [f32; 4],

    pub exposure: f32,
    pub saturation: f32,
    pub contrast: f32,

    pub fog_start: f32,
    pub horizon_power: f32,
    pub star_strength: f32,
    pub night_amount: f32,
}

impl AtmospherePreset {
    pub const DAWN: Self = Self {
        sun_color: [1.95, 0.86, 0.36, 1.0],
        sky_color: [0.38, 0.48, 0.98, 1.0],
        ground_ambient_color: [0.38, 0.22, 0.20, 1.0],
        shadow_tint_color: [0.22, 0.15, 0.42, 1.0],
        fog_color_density: [0.82, 0.48, 0.58, 0.0022],
        clear_color: [0.36, 0.44, 0.88, 1.0],
        zenith_color: [0.08, 0.10, 0.48, 1.0],
        horizon_glow_color: [1.00, 0.36, 0.16, 1.0],
        moon_color: [0.42, 0.48, 0.72, 1.0],

        exposure: 1.04,
        saturation: 1.32,
        contrast: 1.12,

        fog_start: 24.0,
        horizon_power: 1.05,
        star_strength: 0.10,
        night_amount: 0.10,
    };

    pub const NOON: Self = Self {
        sun_color: [1.45, 1.18, 0.78, 1.0],
        sky_color: [0.20, 0.52, 1.05, 1.0],
        ground_ambient_color: [0.32, 0.24, 0.16, 1.0],
        shadow_tint_color: [0.12, 0.18, 0.38, 1.0],
        fog_color_density: [0.46, 0.68, 0.98, 0.0018],
        clear_color: [0.18, 0.48, 0.95, 1.0],
        zenith_color: [0.025, 0.12, 0.68, 1.0],
        horizon_glow_color: [0.95, 0.55, 0.26, 1.0],
            moon_color: [0.0, 0.0, 0.0, 1.0],

        exposure: 1.00,
        saturation: 1.28,
        contrast: 1.10,

        fog_start: 36.0,
        horizon_power: 1.25,
        star_strength: 0.0,
        night_amount: 0.0,
    };

    pub const SUNSET: Self = Self {
        sun_color: [2.20, 0.72, 0.26, 1.0],
        sky_color: [0.34, 0.30, 0.88, 1.0],
        ground_ambient_color: [0.42, 0.22, 0.16, 1.0],
        shadow_tint_color: [0.20, 0.12, 0.42, 1.0],
        fog_color_density: [0.88, 0.40, 0.38, 0.0028],
        clear_color: [0.30, 0.25, 0.72, 1.0],
        zenith_color: [0.045, 0.055, 0.32, 1.0],
        horizon_glow_color: [1.00, 0.26, 0.06, 1.0],
        moon_color: [0.38, 0.44, 0.68, 1.0],

        exposure: 1.08,
        saturation: 1.38,
        contrast: 1.14,

        fog_start: 20.0,
        horizon_power: 0.95,
        star_strength: 0.18,
        night_amount: 0.18,
    };

    pub const NIGHT: Self = Self {
        sun_color: [0.0, 0.0, 0.0, 1.0],
        sky_color: [0.025, 0.055, 0.20, 1.0],
        ground_ambient_color: [0.025, 0.035, 0.075, 1.0],
        shadow_tint_color: [0.010, 0.018, 0.055, 1.0],
        fog_color_density: [0.020, 0.035, 0.095, 0.0018],
        clear_color: [0.006, 0.012, 0.040, 1.0],
        zenith_color: [0.002, 0.006, 0.030, 1.0],
        horizon_glow_color: [0.035, 0.060, 0.160, 1.0],
        moon_color: [0.52, 0.62, 0.95, 1.0],

        exposure: 0.92,
        saturation: 1.10,
        contrast: 1.18,

        fog_start: 18.0,
        horizon_power: 1.55,
        star_strength: 1.0,
        night_amount: 1.0,
    };

    pub fn for_day_time(day01: f32) -> Self {
        let t = day01.rem_euclid(1.0);

        // 0.00 = midnight
        // 0.25 = dawn
        // 0.50 = noon
        // 0.75 = sunset
        if t < 0.25 {
            Self::mix(Self::NIGHT, Self::DAWN, t / 0.25)
        } else if t < 0.50 {
            Self::mix(Self::DAWN, Self::NOON, (t - 0.25) / 0.25)
        } else if t < 0.75 {
            Self::mix(Self::NOON, Self::SUNSET, (t - 0.50) / 0.25)
        } else {
            Self::mix(Self::SUNSET, Self::NIGHT, (t - 0.75) / 0.25)
        }
    }

    pub fn mix(a: Self, b: Self, t: f32) -> Self {
        let t = smoothstep01(t);

        Self {
            sun_color: blend4(a.sun_color, b.sun_color, t),
            sky_color: blend4(a.sky_color, b.sky_color, t),
            ground_ambient_color: blend4(a.ground_ambient_color, b.ground_ambient_color, t),
            shadow_tint_color: blend4(a.shadow_tint_color, b.shadow_tint_color, t),
            fog_color_density: blend4(a.fog_color_density, b.fog_color_density, t),
            clear_color: blend4(a.clear_color, b.clear_color, t),
            zenith_color: blend4(a.zenith_color, b.zenith_color, t),
            horizon_glow_color: blend4(a.horizon_glow_color, b.horizon_glow_color, t),
            moon_color: blend4(a.moon_color, b.moon_color, t),

            exposure: lerp(a.exposure, b.exposure, t),
            saturation: lerp(a.saturation, b.saturation, t),
            contrast: lerp(a.contrast, b.contrast, t),

            fog_start: lerp(a.fog_start, b.fog_start, t),
            horizon_power: lerp(a.horizon_power, b.horizon_power, t),
            star_strength: lerp(a.star_strength, b.star_strength, t),
            night_amount: lerp(a.night_amount, b.night_amount, t),
        }
    }
}

fn sun_direction_for_day_time(day01: f32) -> Vec3 {
    let angle = day01.rem_euclid(1.0) * std::f32::consts::TAU;

    // Slight Z offset avoids perfectly flat light and gives prettier highlights.
    Vec3::new(angle.cos(), angle.sin(), 0.22)
        .try_normalize()
        .unwrap_or(Vec3::Y)
}

fn moon_direction_for_day_time(day01: f32) -> Vec3 {
    let angle = day01.rem_euclid(1.0) * std::f32::consts::TAU;

    Vec3::new(-angle.cos(), -angle.sin(), -0.18)
        .try_normalize()
        .unwrap_or(-Vec3::Y)
}

fn normalized_or(direction: [f32; 3], fallback: Vec3) -> Vec3 {
    Vec3::new(direction[0], direction[1], direction[2])
        .try_normalize()
        .unwrap_or(fallback)
}

fn direction4(direction: Vec3) -> [f32; 4] {
    let direction = direction.try_normalize().unwrap_or(Vec3::Y);
    [direction.x, direction.y, direction.z, 0.0]
}

fn rgb4(color: [f32; 3]) -> [f32; 4] {
    [color[0], color[1], color[2], 1.0]
}

fn rgba_from_config(color: [f64; 4]) -> [f32; 4] {
    [
        color[0] as f32,
        color[1] as f32,
        color[2] as f32,
        color[3] as f32,
    ]
}

fn blend4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
        lerp(a[3], b[3], t),
    ]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
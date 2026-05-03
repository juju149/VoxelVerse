use std::f32::consts::TAU;

use glam::Vec3;
use vv_config::{AtmosphereConfig, DayCycleConfig};

#[derive(Clone, Copy)]
struct SkyKeyframe {
    time: f32,

    sun_color: [f32; 3],
    sky_color: [f32; 3],
    zenith_color: [f32; 3],
    horizon_glow_color: [f32; 3],
    ground_ambient_color: [f32; 3],
    shadow_tint_color: [f32; 3],
    fog_color: [f32; 3],
    moon_color: [f32; 3],

    fog_density: f32,
    exposure: f32,
    saturation: f32,
    contrast: f32,
    fog_start_m: f32,
    star_strength: f32,
    night_amount: f32,
}

const KEYFRAMES: &[SkyKeyframe] = &[
    SkyKeyframe {
        time: 0.000,
        sun_color: [0.010, 0.016, 0.055],
        sky_color: [0.010, 0.018, 0.070],
        zenith_color: [0.004, 0.008, 0.035],
        horizon_glow_color: [0.020, 0.026, 0.080],
        ground_ambient_color: [0.004, 0.006, 0.020],
        shadow_tint_color: [0.006, 0.010, 0.040],
        fog_color: [0.006, 0.010, 0.040],
        moon_color: [0.240, 0.315, 0.610],
        fog_density: 0.00042,
        exposure: 0.92,
        saturation: 1.10,
        contrast: 1.28,
        fog_start_m: 90.0,
        star_strength: 1.35,
        night_amount: 1.0,
    },
    SkyKeyframe {
        time: 0.215,
        sun_color: [0.260, 0.110, 0.075],
        sky_color: [0.040, 0.075, 0.210],
        zenith_color: [0.016, 0.034, 0.120],
        horizon_glow_color: [0.700, 0.250, 0.150],
        ground_ambient_color: [0.018, 0.018, 0.040],
        shadow_tint_color: [0.020, 0.035, 0.105],
        fog_color: [0.140, 0.090, 0.150],
        moon_color: [0.170, 0.220, 0.430],
        fog_density: 0.00055,
        exposure: 0.90,
        saturation: 1.18,
        contrast: 1.30,
        fog_start_m: 75.0,
        star_strength: 0.65,
        night_amount: 0.72,
    },
    SkyKeyframe {
        time: 0.250,
        sun_color: [2.75, 1.28, 0.48],
        sky_color: [0.185, 0.330, 0.780],
        zenith_color: [0.050, 0.125, 0.480],
        horizon_glow_color: [1.45, 0.520, 0.230],
        ground_ambient_color: [0.060, 0.046, 0.038],
        shadow_tint_color: [0.030, 0.052, 0.170],
        fog_color: [0.760, 0.420, 0.240],
        moon_color: [0.020, 0.025, 0.040],
        fog_density: 0.00048,
        exposure: 0.88,
        saturation: 1.24,
        contrast: 1.32,
        fog_start_m: 90.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.365,
        sun_color: [2.30, 1.70, 0.95],
        sky_color: [0.120, 0.330, 0.820],
        zenith_color: [0.038, 0.150, 0.560],
        horizon_glow_color: [0.900, 0.650, 0.380],
        ground_ambient_color: [0.050, 0.052, 0.066],
        shadow_tint_color: [0.032, 0.066, 0.190],
        fog_color: [0.420, 0.560, 0.760],
        moon_color: [0.000, 0.000, 0.000],
        fog_density: 0.00030,
        exposure: 0.86,
        saturation: 1.18,
        contrast: 1.34,
        fog_start_m: 140.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.500,
        sun_color: [2.10, 1.78, 1.10],
        sky_color: [0.105, 0.305, 0.760],
        zenith_color: [0.030, 0.125, 0.520],
        horizon_glow_color: [0.620, 0.780, 1.020],
        ground_ambient_color: [0.042, 0.050, 0.070],
        shadow_tint_color: [0.030, 0.060, 0.190],
        fog_color: [0.380, 0.520, 0.760],
        moon_color: [0.000, 0.000, 0.000],
        fog_density: 0.00028,
        exposure: 0.86,
        saturation: 1.16,
        contrast: 1.36,
        fog_start_m: 155.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.665,
        sun_color: [2.45, 1.50, 0.72],
        sky_color: [0.120, 0.260, 0.650],
        zenith_color: [0.038, 0.115, 0.440],
        horizon_glow_color: [1.050, 0.610, 0.310],
        ground_ambient_color: [0.052, 0.046, 0.055],
        shadow_tint_color: [0.030, 0.052, 0.160],
        fog_color: [0.550, 0.500, 0.650],
        moon_color: [0.010, 0.014, 0.030],
        fog_density: 0.00034,
        exposure: 0.88,
        saturation: 1.22,
        contrast: 1.34,
        fog_start_m: 120.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.750,
        sun_color: [3.10, 1.04, 0.34],
        sky_color: [0.135, 0.170, 0.520],
        zenith_color: [0.035, 0.055, 0.260],
        horizon_glow_color: [1.65, 0.410, 0.180],
        ground_ambient_color: [0.060, 0.038, 0.040],
        shadow_tint_color: [0.025, 0.035, 0.120],
        fog_color: [0.780, 0.300, 0.180],
        moon_color: [0.060, 0.080, 0.160],
        fog_density: 0.00058,
        exposure: 0.90,
        saturation: 1.28,
        contrast: 1.32,
        fog_start_m: 82.0,
        star_strength: 0.10,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.835,
        sun_color: [0.220, 0.090, 0.100],
        sky_color: [0.040, 0.050, 0.190],
        zenith_color: [0.012, 0.022, 0.095],
        horizon_glow_color: [0.560, 0.150, 0.210],
        ground_ambient_color: [0.020, 0.018, 0.040],
        shadow_tint_color: [0.014, 0.018, 0.070],
        fog_color: [0.100, 0.070, 0.160],
        moon_color: [0.190, 0.250, 0.500],
        fog_density: 0.00050,
        exposure: 0.91,
        saturation: 1.16,
        contrast: 1.30,
        fog_start_m: 80.0,
        star_strength: 0.75,
        night_amount: 0.68,
    },
    SkyKeyframe {
        time: 1.000,
        sun_color: [0.010, 0.016, 0.055],
        sky_color: [0.010, 0.018, 0.070],
        zenith_color: [0.004, 0.008, 0.035],
        horizon_glow_color: [0.020, 0.026, 0.080],
        ground_ambient_color: [0.004, 0.006, 0.020],
        shadow_tint_color: [0.006, 0.010, 0.040],
        fog_color: [0.006, 0.010, 0.040],
        moon_color: [0.240, 0.315, 0.610],
        fog_density: 0.00042,
        exposure: 0.92,
        saturation: 1.10,
        contrast: 1.28,
        fog_start_m: 90.0,
        star_strength: 1.35,
        night_amount: 1.0,
    },
];

pub struct SkyState {
    pub time: f32,
    config: DayCycleConfig,
}

impl SkyState {
    pub fn new(config: DayCycleConfig) -> Self {
        Self {
            time: config.initial_time.rem_euclid(1.0),
            config,
        }
    }

    pub fn advance(&mut self, dt: f32) {
        if self.config.freeze_time {
            return;
        }

        let tick = dt * self.config.time_scale / self.config.day_duration_secs.max(1.0);
        self.time = (self.time + tick).rem_euclid(1.0);
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time.rem_euclid(1.0);
    }

    pub fn to_atmosphere(&self) -> AtmosphereConfig {
        let kf = interpolate_keyframes(self.time);
        let sun_dir = sun_direction(self.time);
        let moon_dir = [-sun_dir[0], -sun_dir[1], -sun_dir[2]];

        let clear = [
            (kf.zenith_color[0] * 0.75).min(1.0) as f64,
            (kf.zenith_color[1] * 0.82).min(1.0) as f64,
            (kf.zenith_color[2] * 1.00).min(1.0) as f64,
            1.0,
        ];

        AtmosphereConfig {
            sun_direction: sun_dir,
            sun_color: kf.sun_color,
            sky_color: kf.sky_color,
            ground_ambient_color: kf.ground_ambient_color,
            shadow_tint_color: kf.shadow_tint_color,
            fog_color: kf.fog_color,
            fog_density: kf.fog_density,
            clear_color: clear,

            zenith_color: kf.zenith_color,
            horizon_glow_color: kf.horizon_glow_color,
            moon_direction: moon_dir,
            moon_color: kf.moon_color,

            exposure: kf.exposure,
            saturation: kf.saturation,
            contrast: kf.contrast,

            fog_start_m: kf.fog_start_m,
            sky_horizon_power: 0.72,
            star_strength: kf.star_strength,
            night_amount: kf.night_amount,

            planet_center: [0.0, 0.0, 0.0],
            atmosphere_height_m: 90_000.0,
            atmosphere_fade_start_m: 55_000.0,
            atmosphere_fade_end_m: 120_000.0,
            terminator_softness: 0.095,
        }
    }
}

fn interpolate_keyframes(time: f32) -> SkyKeyframe {
    let mut lo = KEYFRAMES.len() - 2;

    for i in 0..KEYFRAMES.len() - 1 {
        if KEYFRAMES[i + 1].time >= time {
            lo = i;
            break;
        }
    }

    let hi = lo + 1;
    let a = KEYFRAMES[lo];
    let b = KEYFRAMES[hi];

    let range = (b.time - a.time).max(0.00001);
    let t = smootherstep(((time - a.time) / range).clamp(0.0, 1.0));

    SkyKeyframe {
        time,
        sun_color: lerp3(a.sun_color, b.sun_color, t),
        sky_color: lerp3(a.sky_color, b.sky_color, t),
        zenith_color: lerp3(a.zenith_color, b.zenith_color, t),
        horizon_glow_color: lerp3(a.horizon_glow_color, b.horizon_glow_color, t),
        ground_ambient_color: lerp3(a.ground_ambient_color, b.ground_ambient_color, t),
        shadow_tint_color: lerp3(a.shadow_tint_color, b.shadow_tint_color, t),
        fog_color: lerp3(a.fog_color, b.fog_color, t),
        moon_color: lerp3(a.moon_color, b.moon_color, t),
        fog_density: lerp(a.fog_density, b.fog_density, t),
        exposure: lerp(a.exposure, b.exposure, t),
        saturation: lerp(a.saturation, b.saturation, t),
        contrast: lerp(a.contrast, b.contrast, t),
        fog_start_m: lerp(a.fog_start_m, b.fog_start_m, t),
        star_strength: lerp(a.star_strength, b.star_strength, t),
        night_amount: lerp(a.night_amount, b.night_amount, t),
    }
}

fn sun_direction(time: f32) -> [f32; 3] {
    let phase = time * TAU;
    let elevation = -phase.cos();
    let east_west = phase.sin() * 0.92;
    let orbital_tilt = 0.28;

    let direction = Vec3::new(east_west, elevation, orbital_tilt)
        .try_normalize()
        .unwrap_or(Vec3::Y);

    [direction.x, direction.y, direction.z]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ]
}

fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

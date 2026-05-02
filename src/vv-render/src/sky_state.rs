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
        sun_color: [0.020, 0.030, 0.090],
        sky_color: [0.018, 0.030, 0.120],
        zenith_color: [0.006, 0.012, 0.055],
        horizon_glow_color: [0.040, 0.050, 0.140],
        ground_ambient_color: [0.008, 0.010, 0.035],
        shadow_tint_color: [0.010, 0.014, 0.055],
        fog_color: [0.012, 0.018, 0.070],
        moon_color: [0.34, 0.42, 0.72],
        fog_density: 0.00085,
        exposure: 1.04,
        saturation: 1.18,
        contrast: 1.12,
        fog_start_m: 48.0,
        star_strength: 1.25,
        night_amount: 1.0,
    },
    SkyKeyframe {
        time: 0.205,
        sun_color: [0.180, 0.110, 0.090],
        sky_color: [0.070, 0.100, 0.260],
        zenith_color: [0.025, 0.045, 0.155],
        horizon_glow_color: [0.520, 0.245, 0.190],
        ground_ambient_color: [0.030, 0.028, 0.060],
        shadow_tint_color: [0.030, 0.045, 0.125],
        fog_color: [0.170, 0.120, 0.180],
        moon_color: [0.24, 0.30, 0.52],
        fog_density: 0.00110,
        exposure: 1.08,
        saturation: 1.22,
        contrast: 1.10,
        fog_start_m: 38.0,
        star_strength: 0.75,
        night_amount: 0.80,
    },
    SkyKeyframe {
        time: 0.250,
        sun_color: [2.90, 1.34, 0.48],
        sky_color: [0.310, 0.470, 0.980],
        zenith_color: [0.090, 0.170, 0.620],
        horizon_glow_color: [1.55, 0.55, 0.24],
        ground_ambient_color: [0.115, 0.076, 0.055],
        shadow_tint_color: [0.050, 0.075, 0.220],
        fog_color: [1.05, 0.62, 0.32],
        moon_color: [0.05, 0.06, 0.10],
        fog_density: 0.00175,
        exposure: 1.14,
        saturation: 1.28,
        contrast: 1.08,
        fog_start_m: 28.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.340,
        sun_color: [2.35, 1.85, 1.05],
        sky_color: [0.250, 0.520, 1.12],
        zenith_color: [0.085, 0.250, 0.820],
        horizon_glow_color: [1.02, 0.72, 0.42],
        ground_ambient_color: [0.105, 0.092, 0.075],
        shadow_tint_color: [0.055, 0.095, 0.245],
        fog_color: [0.690, 0.760, 0.920],
        moon_color: [0.02, 0.025, 0.04],
        fog_density: 0.00105,
        exposure: 1.08,
        saturation: 1.24,
        contrast: 1.11,
        fog_start_m: 42.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.500,
        sun_color: [2.10, 1.82, 1.24],
        sky_color: [0.180, 0.500, 1.18],
        zenith_color: [0.050, 0.205, 0.900],
        horizon_glow_color: [0.760, 0.910, 1.12],
        ground_ambient_color: [0.090, 0.095, 0.125],
        shadow_tint_color: [0.052, 0.105, 0.285],
        fog_color: [0.560, 0.760, 1.02],
        moon_color: [0.0, 0.0, 0.0],
        fog_density: 0.00090,
        exposure: 1.04,
        saturation: 1.26,
        contrast: 1.16,
        fog_start_m: 56.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.665,
        sun_color: [2.28, 1.62, 0.88],
        sky_color: [0.210, 0.430, 1.02],
        zenith_color: [0.065, 0.185, 0.780],
        horizon_glow_color: [1.02, 0.62, 0.34],
        ground_ambient_color: [0.105, 0.085, 0.092],
        shadow_tint_color: [0.055, 0.090, 0.245],
        fog_color: [0.690, 0.700, 0.900],
        moon_color: [0.015, 0.020, 0.040],
        fog_density: 0.00105,
        exposure: 1.08,
        saturation: 1.28,
        contrast: 1.14,
        fog_start_m: 44.0,
        star_strength: 0.0,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.750,
        sun_color: [3.15, 1.16, 0.40],
        sky_color: [0.245, 0.295, 0.760],
        zenith_color: [0.080, 0.105, 0.420],
        horizon_glow_color: [1.75, 0.46, 0.22],
        ground_ambient_color: [0.118, 0.065, 0.052],
        shadow_tint_color: [0.042, 0.055, 0.175],
        fog_color: [1.08, 0.44, 0.24],
        moon_color: [0.10, 0.13, 0.24],
        fog_density: 0.00195,
        exposure: 1.15,
        saturation: 1.34,
        contrast: 1.10,
        fog_start_m: 28.0,
        star_strength: 0.12,
        night_amount: 0.0,
    },
    SkyKeyframe {
        time: 0.830,
        sun_color: [0.220, 0.115, 0.125],
        sky_color: [0.085, 0.090, 0.300],
        zenith_color: [0.022, 0.035, 0.145],
        horizon_glow_color: [0.620, 0.190, 0.240],
        ground_ambient_color: [0.035, 0.028, 0.060],
        shadow_tint_color: [0.020, 0.025, 0.095],
        fog_color: [0.185, 0.115, 0.245],
        moon_color: [0.26, 0.33, 0.62],
        fog_density: 0.00125,
        exposure: 1.06,
        saturation: 1.25,
        contrast: 1.12,
        fog_start_m: 40.0,
        star_strength: 0.78,
        night_amount: 0.68,
    },
    SkyKeyframe {
        time: 1.000,
        sun_color: [0.020, 0.030, 0.090],
        sky_color: [0.018, 0.030, 0.120],
        zenith_color: [0.006, 0.012, 0.055],
        horizon_glow_color: [0.040, 0.050, 0.140],
        ground_ambient_color: [0.008, 0.010, 0.035],
        shadow_tint_color: [0.010, 0.014, 0.055],
        fog_color: [0.012, 0.018, 0.070],
        moon_color: [0.34, 0.42, 0.72],
        fog_density: 0.00085,
        exposure: 1.04,
        saturation: 1.18,
        contrast: 1.12,
        fog_start_m: 48.0,
        star_strength: 1.25,
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

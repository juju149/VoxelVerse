use std::f32::consts::TAU;

use glam::Vec3;
use vv_config::{AtmosphereConfig, DayCycleConfig};

#[derive(Clone, Copy)]
struct DayKeyframe {
    time: f32,
    sun_color: [f32; 3],
    sky_color: [f32; 3],
    ground_ambient_color: [f32; 3],
    shadow_tint_color: [f32; 3],
    fog_color: [f32; 3],
    fog_density: f32,
}

const KEYFRAMES: &[DayKeyframe] = &[
    DayKeyframe {
        time: 0.000,
        sun_color: [0.000, 0.000, 0.000],
        sky_color: [0.010, 0.020, 0.095],
        ground_ambient_color: [0.018, 0.025, 0.070],
        shadow_tint_color: [0.006, 0.010, 0.040],
        fog_color: [0.016, 0.030, 0.100],
        fog_density: 0.0014,
    },
    DayKeyframe {
        time: 0.180,
        sun_color: [0.050, 0.040, 0.090],
        sky_color: [0.030, 0.045, 0.180],
        ground_ambient_color: [0.025, 0.028, 0.085],
        shadow_tint_color: [0.015, 0.020, 0.075],
        fog_color: [0.040, 0.050, 0.150],
        fog_density: 0.0017,
    },
    DayKeyframe {
        time: 0.250,
        sun_color: [2.150, 0.900, 0.320],
        sky_color: [0.300, 0.430, 1.000],
        ground_ambient_color: [0.340, 0.190, 0.170],
        shadow_tint_color: [0.180, 0.120, 0.380],
        fog_color: [0.880, 0.420, 0.260],
        fog_density: 0.0022,
    },
    DayKeyframe {
        time: 0.330,
        sun_color: [1.850, 1.240, 0.620],
        sky_color: [0.240, 0.500, 1.050],
        ground_ambient_color: [0.290, 0.220, 0.170],
        shadow_tint_color: [0.120, 0.160, 0.360],
        fog_color: [0.540, 0.660, 0.960],
        fog_density: 0.0017,
    },
    DayKeyframe {
        time: 0.500,
        sun_color: [1.450, 1.180, 0.780],
        sky_color: [0.200, 0.520, 1.050],
        ground_ambient_color: [0.320, 0.240, 0.160],
        shadow_tint_color: [0.120, 0.180, 0.380],
        fog_color: [0.460, 0.680, 0.980],
        fog_density: 0.0016,
    },
    DayKeyframe {
        time: 0.670,
        sun_color: [1.700, 1.180, 0.650],
        sky_color: [0.230, 0.470, 1.000],
        ground_ambient_color: [0.340, 0.230, 0.150],
        shadow_tint_color: [0.140, 0.160, 0.360],
        fog_color: [0.560, 0.620, 0.900],
        fog_density: 0.0018,
    },
    DayKeyframe {
        time: 0.750,
        sun_color: [2.200, 0.720, 0.260],
        sky_color: [0.340, 0.300, 0.880],
        ground_ambient_color: [0.420, 0.220, 0.160],
        shadow_tint_color: [0.200, 0.120, 0.420],
        fog_color: [0.880, 0.400, 0.380],
        fog_density: 0.0028,
    },
    DayKeyframe {
        time: 0.820,
        sun_color: [0.280, 0.140, 0.120],
        sky_color: [0.090, 0.090, 0.330],
        ground_ambient_color: [0.060, 0.045, 0.090],
        shadow_tint_color: [0.035, 0.030, 0.120],
        fog_color: [0.180, 0.120, 0.280],
        fog_density: 0.0020,
    },
    DayKeyframe {
        time: 0.900,
        sun_color: [0.000, 0.000, 0.000],
        sky_color: [0.020, 0.035, 0.150],
        ground_ambient_color: [0.020, 0.025, 0.075],
        shadow_tint_color: [0.010, 0.014, 0.055],
        fog_color: [0.030, 0.040, 0.120],
        fog_density: 0.0015,
    },
    DayKeyframe {
        time: 1.000,
        sun_color: [0.000, 0.000, 0.000],
        sky_color: [0.010, 0.020, 0.095],
        ground_ambient_color: [0.018, 0.025, 0.070],
        shadow_tint_color: [0.006, 0.010, 0.040],
        fog_color: [0.016, 0.030, 0.100],
        fog_density: 0.0014,
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

        let duration = self.config.day_duration_secs.max(0.001);
        let tick = dt * self.config.time_scale / duration;

        self.time = (self.time + tick).rem_euclid(1.0);
    }

    pub fn day01(&self) -> f32 {
        self.time.rem_euclid(1.0)
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time.rem_euclid(1.0);
    }

    pub fn to_atmosphere(&self) -> AtmosphereConfig {
        let keyframe = interpolate_keyframes(self.day01());
        let sun_direction = sun_direction(self.day01());
        let clear_color = clear_color_from_sky(keyframe.sky_color);

        AtmosphereConfig {
            sun_direction,
            sun_color: keyframe.sun_color,
            sky_color: keyframe.sky_color,
            ground_ambient_color: keyframe.ground_ambient_color,
            shadow_tint_color: keyframe.shadow_tint_color,
            fog_color: keyframe.fog_color,
            fog_density: keyframe.fog_density,
            clear_color,
        }
    }
}

fn interpolate_keyframes(time: f32) -> DayKeyframe {
    let time = time.rem_euclid(1.0);

    let mut lo = KEYFRAMES.len() - 2;

    for index in 0..KEYFRAMES.len() - 1 {
        if KEYFRAMES[index + 1].time >= time {
            lo = index;
            break;
        }
    }

    let hi = lo + 1;
    let a = KEYFRAMES[lo];
    let b = KEYFRAMES[hi];

    let range = (b.time - a.time).max(0.00001);
    let t = ease(((time - a.time) / range).clamp(0.0, 1.0));

    DayKeyframe {
        time,
        sun_color: lerp3(a.sun_color, b.sun_color, t),
        sky_color: lerp3(a.sky_color, b.sky_color, t),
        ground_ambient_color: lerp3(a.ground_ambient_color, b.ground_ambient_color, t),
        shadow_tint_color: lerp3(a.shadow_tint_color, b.shadow_tint_color, t),
        fog_color: lerp3(a.fog_color, b.fog_color, t),
        fog_density: lerp_f(a.fog_density, b.fog_density, t),
    }
}

fn sun_direction(time: f32) -> [f32; 3] {
    let phase = time.rem_euclid(1.0) * TAU;

    let elevation = -phase.cos();
    let azimuth = phase.sin() * 0.88;
    let tilt = 0.28;

    let direction = Vec3::new(azimuth, elevation, tilt)
        .try_normalize()
        .unwrap_or(Vec3::Y);

    [direction.x, direction.y, direction.z]
}

fn clear_color_from_sky(sky: [f32; 3]) -> [f64; 4] {
    [
        (sky[0] * 0.70).clamp(0.0, 1.0) as f64,
        (sky[1] * 0.82).clamp(0.0, 1.0) as f64,
        (sky[2] * 1.00).clamp(0.0, 1.0) as f64,
        1.0,
    ]
}

fn lerp_f(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        lerp_f(a[0], b[0], t),
        lerp_f(a[1], b[1], t),
        lerp_f(a[2], b[2], t),
    ]
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
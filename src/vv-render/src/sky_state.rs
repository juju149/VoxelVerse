/// Sky state: owns the world clock and computes per-frame atmosphere parameters.
///
/// This is the single source of truth for all time-of-day driven visual parameters.
/// It interpolates between authored keyframes and produces an `AtmosphereConfig` that
/// the renderer injects into the GPU uniform every frame.
///
/// Responsibility boundary:
/// - `DayCycleConfig` (in vv-config) owns tunable parameters.
/// - `SkyState` (here) owns the runtime clock and interpolation.
/// - `AtmosphereConfig` (in vv-config) owns the GPU-ready parameter set.
/// - The renderer consumes `SkyState::to_atmosphere()` and uploads it.
use std::f32::consts::TAU;

use glam::Vec3;
use vv_config::{AtmosphereConfig, DayCycleConfig};

// --- Keyframe data ----------------------------------------------------------

/// A single authored keyframe in the 24-hour color cycle.
/// All color values are in linear HDR space.
#[derive(Clone, Copy)]
struct DayKeyframe {
    /// Moment in the cycle: 0.0 = midnight, 0.25 = sunrise, 0.5 = noon, 0.75 = sunset.
    time: f32,
    /// Sun radiance (RGB). Multiplied by NdotL for direct lighting.
    sun_color: [f32; 3],
    /// Upper-hemisphere ambient color (sky dome).
    sky_color: [f32; 3],
    /// Lower-hemisphere ambient bounce (ground reflected light).
    ground_ambient_color: [f32; 3],
    /// Cool fill applied to shadowed sun-facing surfaces.
    shadow_tint_color: [f32; 3],
    /// Atmospheric fog / aerial perspective color.
    fog_color: [f32; 3],
    /// Exponential-squared fog density.
    fog_density: f32,
}

/// Full cycle keyframes.
/// The first and last entry share the same values so the interpolation wraps seamlessly.
const KEYFRAMES: &[DayKeyframe] = &[
    // 00:00 — Midnight
    DayKeyframe {
        time: 0.000,
        sun_color:            [0.040, 0.050, 0.160], // Moonlight equivalent — very dim
        sky_color:            [0.018, 0.025, 0.075],
        ground_ambient_color: [0.010, 0.012, 0.038],
        shadow_tint_color:    [0.008, 0.010, 0.040],
        fog_color:            [0.020, 0.025, 0.065],
        fog_density:          0.0010,
    },
    // 04:48 — Pre-dawn
    DayKeyframe {
        time: 0.200,
        sun_color:            [0.120, 0.080, 0.060],
        sky_color:            [0.040, 0.050, 0.160],
        ground_ambient_color: [0.020, 0.022, 0.065],
        shadow_tint_color:    [0.018, 0.022, 0.075],
        fog_color:            [0.055, 0.058, 0.140],
        fog_density:          0.0018,
    },
    // 06:00 — Sunrise
    DayKeyframe {
        time: 0.250,
        sun_color:            [2.200, 1.050, 0.400], // Intense warm orange
        sky_color:            [0.200, 0.260, 0.680],
        ground_ambient_color: [0.075, 0.055, 0.038],
        shadow_tint_color:    [0.035, 0.055, 0.175],
        fog_color:            [0.880, 0.500, 0.200], // Warm orange haze
        fog_density:          0.0022,
    },
    // 07:55 — Morning
    DayKeyframe {
        time: 0.330,
        sun_color:            [2.050, 1.580, 0.880],
        sky_color:            [0.160, 0.280, 0.700],
        ground_ambient_color: [0.072, 0.068, 0.055],
        shadow_tint_color:    [0.042, 0.072, 0.200],
        fog_color:            [0.700, 0.620, 0.520],
        fog_density:          0.0015,
    },
    // 12:00 — Noon
    DayKeyframe {
        time: 0.500,
        sun_color:            [1.850, 1.550, 1.050], // Neutral warm
        sky_color:            [0.180, 0.320, 0.740], // Rich saturated blue
        ground_ambient_color: [0.080, 0.070, 0.130], // Cool violet bounce
        shadow_tint_color:    [0.060, 0.100, 0.240],
        fog_color:            [0.580, 0.700, 0.860],
        fog_density:          0.0012,
    },
    // 16:05 — Afternoon
    DayKeyframe {
        time: 0.670,
        sun_color:            [1.950, 1.500, 0.920], // Getting warmer
        sky_color:            [0.170, 0.300, 0.700],
        ground_ambient_color: [0.078, 0.068, 0.112],
        shadow_tint_color:    [0.055, 0.092, 0.220],
        fog_color:            [0.620, 0.660, 0.820],
        fog_density:          0.0013,
    },
    // 18:00 — Sunset
    DayKeyframe {
        time: 0.750,
        sun_color:            [2.200, 1.000, 0.380], // Symmetric with sunrise
        sky_color:            [0.200, 0.260, 0.660],
        ground_ambient_color: [0.076, 0.052, 0.035],
        shadow_tint_color:    [0.035, 0.050, 0.175],
        fog_color:            [0.920, 0.480, 0.180], // Deep warm orange
        fog_density:          0.0022,
    },
    // 19:41 — Dusk
    DayKeyframe {
        time: 0.820,
        sun_color:            [0.280, 0.180, 0.140],
        sky_color:            [0.075, 0.092, 0.300], // Blue-violet
        ground_ambient_color: [0.038, 0.032, 0.058],
        shadow_tint_color:    [0.028, 0.035, 0.115],
        fog_color:            [0.200, 0.160, 0.280], // Purple-grey
        fog_density:          0.0016,
    },
    // 21:07 — Twilight
    DayKeyframe {
        time: 0.880,
        sun_color:            [0.050, 0.040, 0.080],
        sky_color:            [0.030, 0.042, 0.165],
        ground_ambient_color: [0.016, 0.018, 0.055],
        shadow_tint_color:    [0.012, 0.016, 0.060],
        fog_color:            [0.042, 0.045, 0.115],
        fog_density:          0.0011,
    },
    // 00:00 — Midnight (closing frame, identical to t=0.000 for seamless loop)
    DayKeyframe {
        time: 1.000,
        sun_color:            [0.040, 0.050, 0.160],
        sky_color:            [0.018, 0.025, 0.075],
        ground_ambient_color: [0.010, 0.012, 0.038],
        shadow_tint_color:    [0.008, 0.010, 0.040],
        fog_color:            [0.020, 0.025, 0.065],
        fog_density:          0.0010,
    },
];

// --- Interpolation helpers --------------------------------------------------

#[inline]
fn lerp_f(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp_f(a[0], b[0], t), lerp_f(a[1], b[1], t), lerp_f(a[2], b[2], t)]
}

/// Smoothstep easing for silky transitions between keyframes.
#[inline]
fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn interpolate_keyframes(time: f32) -> DayKeyframe {
    // Find the two surrounding keyframes.
    let mut lo = KEYFRAMES.len() - 2;
    for i in 0..KEYFRAMES.len() - 1 {
        if KEYFRAMES[i + 1].time >= time {
            lo = i;
            break;
        }
    }
    let hi = lo + 1;
    let a = &KEYFRAMES[lo];
    let b = &KEYFRAMES[hi];
    let range = b.time - a.time;
    let raw_t = if range > 1e-5 { (time - a.time) / range } else { 0.0 };
    let t = ease(raw_t.clamp(0.0, 1.0));

    DayKeyframe {
        time,
        sun_color:            lerp3(a.sun_color, b.sun_color, t),
        sky_color:            lerp3(a.sky_color, b.sky_color, t),
        ground_ambient_color: lerp3(a.ground_ambient_color, b.ground_ambient_color, t),
        shadow_tint_color:    lerp3(a.shadow_tint_color, b.shadow_tint_color, t),
        fog_color:            lerp3(a.fog_color, b.fog_color, t),
        fog_density:          lerp_f(a.fog_density, b.fog_density, t),
    }
}

// --- Sun direction ----------------------------------------------------------

/// Computes the world-space direction FROM the planet TO the sun for a given time.
///
/// The sun orbits in the XY plane (Y = planet north-pole direction at spawn).
/// - t = 0.00 → midnight  → sun directly below  [0, -1, …]
/// - t = 0.25 → sunrise   → sun on east horizon [+X, 0, …]
/// - t = 0.50 → noon      → sun overhead        [0, +1, …]
/// - t = 0.75 → sunset    → sun on west horizon [-X, 0, …]
fn sun_direction(time: f32) -> [f32; 3] {
    let phase = time * TAU;
    // Y component: -cos maps 0→midnight(-1), 0.5→noon(+1) correctly.
    let elev = -phase.cos();
    // X component: east at sunrise, west at sunset.
    let azimuth = phase.sin() * 0.88;
    // Z tilt: slight persistent offset so midday light is never perfectly flat.
    let tilt = 0.28_f32;
    let v = Vec3::new(azimuth, elev, tilt);
    let v = v.try_normalize().unwrap_or(Vec3::Y);
    [v.x, v.y, v.z]
}

// --- SkyState ---------------------------------------------------------------

/// Runtime day/night cycle clock.
///
/// Advances the world time and produces a fully interpolated `AtmosphereConfig`
/// ready to upload to the GPU each frame.
pub struct SkyState {
    /// Current time of day: 0.0 = midnight, 0.5 = noon. Wraps at 1.0.
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

    /// Advances the world clock by `dt` real-world seconds.
    /// No-op when `freeze_time` is set.
    pub fn advance(&mut self, dt: f32) {
        if self.config.freeze_time {
            return;
        }
        let tick = dt * self.config.time_scale / self.config.day_duration_secs;
        self.time = (self.time + tick).rem_euclid(1.0);
    }

    /// Computes the current `AtmosphereConfig` for this moment in the day.
    ///
    /// Called once per frame; all operations are cheap arithmetic.
    pub fn to_atmosphere(&self) -> AtmosphereConfig {
        let kf = interpolate_keyframes(self.time);
        let sun_dir = sun_direction(self.time);

        // Approximate the sky zenith color for the wgpu clear-color op.
        // Uses the same multiplier the sky shader applies for the zenith (sky * [0.62, 0.72, 1.22]).
        // This avoids a visible flash of the wrong color before the sky draw runs.
        let clear_r = (kf.sky_color[0] * 0.62).min(1.0) as f64;
        let clear_g = (kf.sky_color[1] * 0.72).min(1.0) as f64;
        let clear_b = (kf.sky_color[2] * 1.22).min(1.0) as f64;

        AtmosphereConfig {
            sun_direction: sun_dir,
            sun_color: kf.sun_color,
            sky_color: kf.sky_color,
            ground_ambient_color: kf.ground_ambient_color,
            shadow_tint_color: kf.shadow_tint_color,
            fog_color: kf.fog_color,
            fog_density: kf.fog_density,
            clear_color: [clear_r, clear_g, clear_b, 1.0],
        }
    }
}

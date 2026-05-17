//! Wind solver — base wind, gust process, slow direction drift.
//!
//! Stays scalar + 2D in the XZ plane: the renderer only needs the horizontal
//! drag. Lift gets recomputed per-particle if/when we add advection.

use crate::profile::ResolvedProfile;
use crate::rng::PcgRng;
use crate::snapshot::WindVector;

/// Persistent wind state, advanced once per [`crate::WeatherSimState::tick`].
#[derive(Clone, Debug)]
pub struct WindState {
    /// Direction angle in radians (yaw around +Y). Drifts slowly.
    pub heading_rad: f32,
    /// Seconds remaining before the next gust peak fires.
    pub time_to_next_gust_s: f32,
    /// Current gust contribution in m/s (decays linearly between peaks).
    pub gust_speed_m_s: f32,
    /// Time since the most recent gust peak — drives the linear decay.
    pub time_since_gust_s: f32,
}

impl WindState {
    pub fn new(initial_heading_rad: f32) -> Self {
        Self {
            heading_rad: initial_heading_rad,
            time_to_next_gust_s: 0.0,
            gust_speed_m_s: 0.0,
            time_since_gust_s: 0.0,
        }
    }

    /// Advance the gust process and drift the heading using the supplied
    /// profile's `wind` config.
    pub fn tick(&mut self, dt: f32, profile: &ResolvedProfile, rng: &mut PcgRng) {
        let wind = &profile.wind;

        // Heading drift: deterministic random walk, ±drift_per_s rad/s.
        let drift = wind.direction_drift_per_s.max(0.0);
        if drift > 0.0 {
            let step = (rng.next_unit() * 2.0 - 1.0) * drift * dt;
            self.heading_rad = (self.heading_rad + step).rem_euclid(std::f32::consts::TAU);
        }

        // Gust process: every `gust_interval_s` (jittered ±30 %), trigger a
        // peak. Between peaks, the gust contribution decays linearly to zero
        // over the same interval.
        let interval = wind.gust_interval_s.max(0.1);
        if self.time_to_next_gust_s <= 0.0 {
            // Trigger a gust.
            let jitter = rng.next_range(0.7, 1.3);
            self.time_to_next_gust_s = interval * jitter;
            self.gust_speed_m_s = (wind.gust_speed - wind.base_speed).max(0.0);
            self.time_since_gust_s = 0.0;
        } else {
            self.time_to_next_gust_s -= dt;
            self.time_since_gust_s += dt;
        }

        // Linear decay between peaks.
        let decay = (self.time_since_gust_s / interval).clamp(0.0, 1.0);
        let active_gust = (wind.gust_speed - wind.base_speed).max(0.0) * (1.0 - decay);
        self.gust_speed_m_s = active_gust;
    }

    /// Sample the current wind vector. `profile_blend` lets the caller smooth
    /// the base speed across a `current → next` transition without owning the
    /// blend state here.
    pub fn sample(
        &self,
        current: &ResolvedProfile,
        next: Option<&ResolvedProfile>,
        blend: f32,
    ) -> WindVector {
        let base = lerp(
            current.wind.base_speed,
            next.map(|p| p.wind.base_speed)
                .unwrap_or(current.wind.base_speed),
            blend,
        );
        let speed = (base + self.gust_speed_m_s).max(0.0);
        let dir = glam::Vec3::new(self.heading_rad.cos(), 0.0, self.heading_rad.sin());
        WindVector {
            direction: dir,
            speed,
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

//! Per-frame snapshot consumed by the renderer, audio, and HUD.
//!
//! Produced by [`crate::WeatherSimState::snapshot`] each frame, this struct is
//! the *only* surface the rest of the engine reads. The solver state itself
//! never leaks.

use crate::profile::WeatherProfileId;

/// Visual + audio precipitation contribution for the current frame.
///
/// During a transition between two profiles, the sim blends the `current` and
/// `next` precipitation linearly so streaks fade in/out smoothly.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrecipitationSample {
    pub kind: PrecipitationKindSample,
    /// Visual intensity in `[0, 1]`.
    pub intensity: f32,
    /// Wind drag factor in `[0, 1]`.
    pub wind_drift: f32,
    /// Decal density on solids/water in `[0, 1]`.
    pub splash_density: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrecipitationKindSample {
    #[default]
    None,
    Rain,
    Snow,
    Sleet,
    Sand,
    Ash,
    ToxicMist,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WindVector {
    /// Horizontal direction (unit, XZ plane). Y is unused — gusts don't lift.
    pub direction: glam::Vec3,
    /// Effective speed in m/s (base + active gust contribution).
    pub speed: f32,
}

/// Lightning strike emitted by the sim. The renderer draws the bolt and the
/// audio layer schedules a delayed thunder clap based on `distance_m`.
#[derive(Clone, Copy, Debug)]
pub struct LightningStrike {
    /// World-space ground impact (Y on terrain top in caller's frame).
    pub position: glam::Vec3,
    /// Distance from the observer in metres — drives thunder delay.
    pub distance_m: f32,
    /// Frame-additive ambient brightness boost.
    pub flash_intensity: f32,
    /// Audio delay in seconds (precomputed from `distance_m`).
    pub thunder_delay_s: f32,
}

/// Public snapshot. Cheap to copy.
#[derive(Clone, Debug)]
pub struct WeatherState {
    pub current: WeatherProfileId,
    pub next: Option<WeatherProfileId>,
    /// Linear blend `0..1` from `current` toward `next`. `0` when not
    /// transitioning, `1` the frame the swap completes.
    pub blend: f32,
    /// Effective cloud coverage after blending (`0..1`).
    pub cloud_coverage: f32,
    /// Effective fog multiplier after blending (`> 0`).
    pub fog_multiplier: f32,
    /// Effective cloud density multiplier after blending.
    pub cloud_density_mul: f32,
    /// Effective cloud speed multiplier after blending.
    pub cloud_speed_mul: f32,
    pub wind: WindVector,
    pub precipitation: PrecipitationSample,
    /// Lightning strikes produced this frame. Drained next frame.
    pub lightning_events: Vec<LightningStrike>,
}

impl WeatherState {
    /// Returns `true` while a `current → next` transition is in flight.
    pub fn is_transitioning(&self) -> bool {
        self.next.is_some()
    }
}

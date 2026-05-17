//! Weather profile schema — `defs/world/weather/*.weather.ron`.
//!
//! A `WeatherProfile` describes **one kind** of weather condition (clear,
//! storm, blizzard, ...), not an instantaneous state. The runtime solver
//! (`vv-weather`, Phase 2) draws transitions between profiles weighted by
//! biome and climate.
//!
//! See `docs/v1/13_WEATHER_AND_COSMOS.md` §3.1.

use crate::ContentRef;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawPrecipitationKind {
    None,
    Rain,
    Snow,
    Sleet,
    Sand,
    Ash,
    ToxicMist,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawWeatherPrecipitationDef {
    pub kind: RawPrecipitationKind,
    /// Visual intensity in `[0, 1]`. Drives streak/flake density.
    pub intensity: f32,
    /// 0..1 — how strongly horizontal wind drags streaks.
    #[serde(default)]
    pub wind_drift: f32,
    /// 0..1 — density of splash decals on solids and water.
    #[serde(default)]
    pub splash_density: f32,
    /// Sound event reference (resolved by the audio crate).
    #[serde(default)]
    pub sound: Option<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawWeatherWindDef {
    /// Base wind speed in m/s.
    pub base_speed: f32,
    /// Gust speed in m/s, sampled periodically.
    pub gust_speed: f32,
    /// Average seconds between gust peaks.
    pub gust_interval_s: f32,
    /// Radians per second of slow direction drift.
    #[serde(default)]
    pub direction_drift_per_s: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawWeatherLightningDef {
    pub strikes_per_minute: f32,
    /// Multiplier added to ambient on the flash frame.
    pub flash_intensity: f32,
    /// Audio delay per km of distance to the strike (seconds).
    #[serde(default = "default_thunder_delay")]
    pub thunder_delay_per_km: f32,
    #[serde(default)]
    pub sound: Option<ContentRef>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RawWeatherPostFxDef {
    #[serde(default = "one")]
    pub exposure_mul: f32,
    #[serde(default = "one")]
    pub saturation_mul: f32,
    #[serde(default)]
    pub contrast_add: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RawWeatherTransitionsDef {
    #[serde(default = "default_fade")]
    pub fade_in_s: f32,
    #[serde(default = "default_fade")]
    pub fade_out_s: f32,
    #[serde(default = "default_min_duration")]
    pub min_duration_s: f32,
    #[serde(default = "default_max_duration")]
    pub max_duration_s: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawWeatherProfileDef {
    #[serde(alias = "name")]
    pub display_name: String,
    /// Base spawn weight in `[0, 1]`. Combined with `biome_bias` to pick
    /// the next condition during a transition tick.
    #[serde(default = "default_rarity")]
    pub rarity: f32,
    /// Optional per-biome weight multipliers keyed by short biome id.
    #[serde(default)]
    pub biome_bias: BTreeMap<String, f32>,
    /// Overrides for `AtmosphereConfig::weather.cloud_coverage`.
    #[serde(default = "default_coverage")]
    pub cloud_coverage: f32,
    /// Multiplier on the resolved cloud density.
    #[serde(default = "one")]
    pub cloud_density_mul: f32,
    /// Multiplier on the resolved cloud scrolling speed.
    #[serde(default = "one")]
    pub cloud_speed_mul: f32,
    /// Optional cloud tint mixed into the sky colour.
    #[serde(default)]
    pub cloud_tint: Option<(f32, f32, f32)>,
    #[serde(default = "one")]
    pub fog_multiplier: f32,
    #[serde(default)]
    pub fog_tint: Option<(f32, f32, f32)>,
    #[serde(default)]
    pub precipitation: Option<RawWeatherPrecipitationDef>,
    pub wind: RawWeatherWindDef,
    #[serde(default)]
    pub lightning: Option<RawWeatherLightningDef>,
    #[serde(default = "default_post_fx")]
    pub post_fx: RawWeatherPostFxDef,
    #[serde(default = "default_transitions")]
    pub transitions: RawWeatherTransitionsDef,
}

fn one() -> f32 {
    1.0
}

fn default_rarity() -> f32 {
    0.1
}

fn default_coverage() -> f32 {
    0.5
}

fn default_thunder_delay() -> f32 {
    3.0
}

fn default_fade() -> f32 {
    8.0
}

fn default_min_duration() -> f32 {
    60.0
}

fn default_max_duration() -> f32 {
    240.0
}

fn default_post_fx() -> RawWeatherPostFxDef {
    RawWeatherPostFxDef {
        exposure_mul: 1.0,
        saturation_mul: 1.0,
        contrast_add: 0.0,
    }
}

fn default_transitions() -> RawWeatherTransitionsDef {
    RawWeatherTransitionsDef {
        fade_in_s: default_fade(),
        fade_out_s: default_fade(),
        min_duration_s: default_min_duration(),
        max_duration_s: default_max_duration(),
    }
}

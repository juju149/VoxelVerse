//! Biome ambience schema — `defs/world/biome_ambience/*.biome_ambience.ron`.
//!
//! Layer L1 of the weather/cosmos stack: what the *biome* contributes to the
//! sky, fog, particles and post-FX **on top** of the weather profile.
//!
//! See `docs/v1/13_WEATHER_AND_COSMOS.md` §3.2.

use crate::ContentRef;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RawAmbiencePostFxDef {
    #[serde(default = "one")]
    pub exposure_mul: f32,
    #[serde(default = "one")]
    pub saturation_mul: f32,
    #[serde(default)]
    pub contrast_add: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAmbienceParticleDef {
    /// Particle system / scatter id, resolved at compile time.
    pub kind: ContentRef,
    /// Spawn intensity in `[0, 1]`.
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RawAuroraConfigDef {
    /// Absolute latitude fraction `[0, 1]` above which the aurora can ignite.
    pub latitude_threshold: f32,
    /// Inner ribbon colour.
    pub color_a: (f32, f32, f32),
    /// Outer ribbon colour.
    pub color_b: (f32, f32, f32),
    /// 0..1 — base intensity multiplier when above `latitude_threshold` and
    /// fully dark. The renderer further modulates it by night factor and
    /// cloud coverage.
    #[serde(default = "default_aurora_intensity")]
    pub intensity: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBiomeAmbienceDef {
    #[serde(alias = "name")]
    pub display_name: String,
    /// Multiplier applied to the resolved fog colour.
    #[serde(default = "default_white")]
    pub fog_tint_mul: (f32, f32, f32),
    /// Optional tint pushed into the horizon colour.
    #[serde(default)]
    pub sky_horizon_tint: Option<(f32, f32, f32)>,
    /// Ambient dust density in `[0, 1]` (e.g. desert haze).
    #[serde(default)]
    pub ambient_dust_density: f32,
    /// Optional ambient particle layer (snow drift, pollen, spores, ...).
    #[serde(default)]
    pub ambient_particles: Option<RawAmbienceParticleDef>,
    #[serde(default = "default_post_fx")]
    pub post_fx: RawAmbiencePostFxDef,
    /// Weather profile ids allowed in this biome. Empty = all allowed.
    #[serde(default)]
    pub allowed_weather: Vec<ContentRef>,
    /// Per-weather-id multiplier on top of the profile's base `rarity`.
    #[serde(default)]
    pub weather_weights: BTreeMap<String, f32>,
    /// Optional aurora overlay, used in polar biomes.
    #[serde(default)]
    pub aurora: Option<RawAuroraConfigDef>,
}

fn one() -> f32 {
    1.0
}

fn default_white() -> (f32, f32, f32) {
    (1.0, 1.0, 1.0)
}

fn default_aurora_intensity() -> f32 {
    1.0
}

fn default_post_fx() -> RawAmbiencePostFxDef {
    RawAmbiencePostFxDef {
        exposure_mul: 1.0,
        saturation_mul: 1.0,
        contrast_add: 0.0,
    }
}

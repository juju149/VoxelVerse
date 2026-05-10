//! Sound event definitions.
//!
//! Sprint 0 scope: typed registry only. The runtime audio engine is **not yet
//! implemented** — these defs exist so `ContentRef`s like `core:sound/step/grass`
//! resolve to a real, addressable asset rather than dangling.
//!
//! A sound event represents a logical sound (e.g. "footstep on grass"). At play
//! time, the runtime will pick one variant and apply random pitch/volume in the
//! configured ranges. For now, `variants` may be empty; clips are optional.

use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawSoundEventDef {
    pub format_version: u32,
    pub display_name: String,
    /// Default playback gain applied when no per-variant override is present.
    /// Range: [0.0, 1.0]. Defaults to 1.0.
    #[serde(default = "default_one")]
    pub default_volume: f32,
    /// Default playback pitch multiplier. 1.0 = nominal.
    #[serde(default = "default_one")]
    pub default_pitch: f32,
    /// Possible audio variants picked at play time. May be empty during
    /// Sprint 0 (no runtime audio yet).
    #[serde(default)]
    pub variants: Vec<RawSoundVariant>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSoundVariant {
    /// Reference to an audio clip asset. Optional so stub events with no
    /// recorded audio yet remain valid.
    #[serde(default)]
    pub clip: Option<ContentRef>,
    /// Per-variant volume override; falls back to event default.
    #[serde(default)]
    pub volume: Option<f32>,
    /// Per-variant pitch override; falls back to event default.
    #[serde(default)]
    pub pitch: Option<f32>,
    /// Relative pick weight when sampling a variant. Defaults to 1.0.
    #[serde(default = "default_one")]
    pub weight: f32,
}

fn default_one() -> f32 {
    1.0
}

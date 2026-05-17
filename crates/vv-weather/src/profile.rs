//! Runtime weather profile registry.
//!
//! The pack loader hands the runtime a list of `(key, RawWeatherProfileDef)`.
//! We resolve each entry into a [`ResolvedProfile`] (validated, indexed by a
//! compact [`WeatherProfileId`]) and keep a short-id lookup for biome bias
//! resolution (`biome_bias` keys reference biome short ids).

use std::collections::BTreeMap;

use vv_content_schema::{
    RawPrecipitationKind, RawWeatherLightningDef, RawWeatherPostFxDef, RawWeatherPrecipitationDef,
    RawWeatherProfileDef, RawWeatherTransitionsDef, RawWeatherWindDef,
};

use crate::snapshot::PrecipitationKindSample;

/// Compact index into the [`WeatherRegistry`]. Stable across a save: indices
/// are assigned in the order the registry receives them, which the loader
/// guarantees is alphabetical (`load_typed_tree` sorts by path).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct WeatherProfileId(pub u16);

#[derive(Clone, Debug)]
pub struct ResolvedProfile {
    pub id: WeatherProfileId,
    /// Full content key, e.g. `"core:weather/thunderstorm"`.
    pub key: String,
    /// Short id, e.g. `"thunderstorm"`. Used by `biome_bias` lookups.
    pub short_id: String,
    pub display_name: String,
    pub rarity: f32,
    pub biome_bias: BTreeMap<String, f32>,
    pub cloud_coverage: f32,
    pub cloud_density_mul: f32,
    pub cloud_speed_mul: f32,
    pub fog_multiplier: f32,
    pub precipitation: Option<ResolvedPrecipitation>,
    pub wind: RawWeatherWindDef,
    pub lightning: Option<RawWeatherLightningDef>,
    pub post_fx: RawWeatherPostFxDef,
    pub transitions: RawWeatherTransitionsDef,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedPrecipitation {
    pub kind: PrecipitationKindSample,
    pub intensity: f32,
    pub wind_drift: f32,
    pub splash_density: f32,
}

#[derive(Default)]
pub struct WeatherRegistry {
    profiles: Vec<ResolvedProfile>,
    by_short_id: BTreeMap<String, WeatherProfileId>,
}

impl WeatherRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a registry from the loader's raw output. Duplicate short ids
    /// shadow earlier entries; the loader guarantees keys are unique so this
    /// only happens if the caller has constructed a list manually.
    pub fn from_raw(items: &[(String, RawWeatherProfileDef)]) -> Self {
        let mut reg = Self::default();
        for (key, raw) in items {
            reg.insert(key.clone(), raw);
        }
        reg
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResolvedProfile> {
        self.profiles.iter()
    }

    pub fn get(&self, id: WeatherProfileId) -> &ResolvedProfile {
        &self.profiles[id.0 as usize]
    }

    pub fn id_of(&self, short_id: &str) -> Option<WeatherProfileId> {
        self.by_short_id.get(short_id).copied()
    }

    fn insert(&mut self, key: String, raw: &RawWeatherProfileDef) {
        let id = WeatherProfileId(self.profiles.len() as u16);
        let short_id = short_id_from_key(&key);
        let resolved = ResolvedProfile {
            id,
            key: key.clone(),
            short_id: short_id.clone(),
            display_name: raw.display_name.clone(),
            rarity: raw.rarity.max(0.0),
            biome_bias: raw.biome_bias.clone(),
            cloud_coverage: raw.cloud_coverage.clamp(0.0, 1.0),
            cloud_density_mul: raw.cloud_density_mul.max(0.0),
            cloud_speed_mul: raw.cloud_speed_mul.max(0.0),
            fog_multiplier: raw.fog_multiplier.max(0.0),
            precipitation: raw.precipitation.as_ref().map(resolve_precipitation),
            wind: raw.wind.clone(),
            lightning: raw.lightning.clone(),
            post_fx: raw.post_fx,
            transitions: raw.transitions,
        };
        self.by_short_id.insert(short_id, id);
        self.profiles.push(resolved);
    }
}

fn resolve_precipitation(raw: &RawWeatherPrecipitationDef) -> ResolvedPrecipitation {
    ResolvedPrecipitation {
        kind: map_precipitation_kind(raw.kind),
        intensity: raw.intensity.clamp(0.0, 1.0),
        wind_drift: raw.wind_drift.clamp(0.0, 1.0),
        splash_density: raw.splash_density.clamp(0.0, 1.0),
    }
}

fn map_precipitation_kind(kind: RawPrecipitationKind) -> PrecipitationKindSample {
    match kind {
        RawPrecipitationKind::None => PrecipitationKindSample::None,
        RawPrecipitationKind::Rain => PrecipitationKindSample::Rain,
        RawPrecipitationKind::Snow => PrecipitationKindSample::Snow,
        RawPrecipitationKind::Sleet => PrecipitationKindSample::Sleet,
        RawPrecipitationKind::Sand => PrecipitationKindSample::Sand,
        RawPrecipitationKind::Ash => PrecipitationKindSample::Ash,
        RawPrecipitationKind::ToxicMist => PrecipitationKindSample::ToxicMist,
    }
}

/// `"core:weather/thunderstorm"` → `"thunderstorm"`. Falls back to the full
/// key when the conventional shape is absent.
fn short_id_from_key(key: &str) -> String {
    key.rsplit('/').next().unwrap_or(key).to_string()
}

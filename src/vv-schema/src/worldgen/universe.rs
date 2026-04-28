use crate::worldgen::planet::PlanetTypeRef;
use serde::{Deserialize, Serialize};

/// Universe singleton. Deserialized from defs/worldgen/universe.ron.
/// One file per pack (at most).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UniverseDef {
    pub default_planet_type: PlanetTypeRef,
    #[serde(default)]
    pub default_seed: Option<u64>,
    #[serde(default)]
    pub starting_planet_count: u32,
}

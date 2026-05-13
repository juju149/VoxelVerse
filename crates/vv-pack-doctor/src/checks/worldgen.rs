//! World-files cross-checks.
//!
//! References inside world files are largely handled by `references.rs`
//! (because the world schema is in flux). This module adds higher-level
//! sanity checks that don't need typed access:
//!
//!   - the pack must have at least one planet profile
//!   - each planet profile must list at least one biome via `biome_set`
//!   - every biome must declare a `surface.top` value

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};
use crate::scan::WorldCategory;

const CHECK: &str = "worldgen";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    let planets: Vec<_> = index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::Planets)
        .collect();
    if planets.is_empty() {
        report.error(
            Diagnostic::new(CHECK, "pack contains no planet profiles")
                .with_path("defs/world/planets/".to_string())
                .with_suggestion(
                    "add at least one `defs/world/planets/*.profile.ron` so the world can spawn"
                        .to_string(),
                ),
        );
    }
    for planet in planets {
        if value_field(&planet.value, "biome_set").is_none() {
            report.error(
                Diagnostic::new(
                    CHECK,
                    "planet profile has no `biome_set` field",
                )
                .with_path(planet.rel_path.clone())
                .with_id(planet.id.clone())
                .with_field("biome_set"),
            );
        }
        if value_field(&planet.value, "terrain_layers").is_none() {
            report.error(
                Diagnostic::new(
                    CHECK,
                    "planet profile has no `terrain_layers` field",
                )
                .with_path(planet.rel_path.clone())
                .with_id(planet.id.clone())
                .with_field("terrain_layers"),
            );
        }
    }

    for biome in index
        .scan
        .world_files
        .iter()
        .filter(|f| f.category == WorldCategory::Biome)
    {
        let surface = value_field(&biome.value, "surface");
        let top_ok = surface
            .as_ref()
            .map(|v| value_field(v, "top").is_some())
            .unwrap_or(false);
        if !top_ok {
            report.error(
                Diagnostic::new(CHECK, "biome has no `surface.top` block")
                    .with_path(biome.rel_path.clone())
                    .with_id(biome.id.clone())
                    .with_field("surface.top"),
            );
        }
    }
}

fn value_field<'a>(value: &'a ron::Value, key: &str) -> Option<&'a ron::Value> {
    let ron::Value::Map(map) = value else { return None };
    for (k, v) in map.iter() {
        if let ron::Value::String(s) = k {
            if s == key {
                return Some(v);
            }
        }
    }
    None
}

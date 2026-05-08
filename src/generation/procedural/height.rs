//! Per-biome height composition.
//!
//! For each candidate biome at a given direction, evaluates its hill +
//! optional ridge fields, applies amplitude / flatness / terrace, and sums
//! the result weighted by the biome blend.  A small macro shape then
//! biases the global relief based on continentality (continents push up),
//! erosion (older terrain sits lower), and weirdness (a small jitter).
//!
//! Output is a clamped layer index in `[core_layers + 2, resolution - 3]`
//! so that callers never have to worry about over- or under-shooting the
//! voxel grid.

use super::biome_select::resolve_biome_weights;
use super::climate::SurfaceFields;
use super::noise_sampler::sample_noise_field;
use crate::content::{CompiledProceduralPlanet, ProceduralRegistry};
use crate::generation::noise::NoiseGenerator;
use crate::world::PlanetProfile;
use glam::Vec3;

pub(super) fn resolve_height(
    registry: &ProceduralRegistry,
    generators: &[NoiseGenerator],
    planet: &CompiledProceduralPlanet,
    profile: PlanetProfile,
    dir: Vec3,
    fields: &SurfaceFields,
) -> (u32, usize) {
    let (weights, primary) = resolve_biome_weights(registry, planet, *fields);
    let mut height_offset = 0.0;
    for weight in &weights {
        let biome = registry.biome(weight.biome);
        let hill =
            sample_noise_field(registry, generators, biome.terrain.hill_field, dir, 0) * 2.0 - 1.0;
        let ridge = biome
            .terrain
            .ridge_field
            .map(|field| sample_noise_field(registry, generators, field, dir, 0) * 2.0 - 1.0)
            .unwrap_or(0.0);
        let flat = 1.0 - biome.terrain.flatness;
        let mut local = biome.terrain.base_height
            + hill * biome.terrain.amplitude * flat
            + ridge * biome.terrain.amplitude * 0.35;
        if biome.terrain.terrace_strength > 0.0 {
            let steps = 12.0;
            let terraced = (local * steps).round() / steps;
            local = local + (terraced - local) * biome.terrain.terrace_strength;
        }
        height_offset += local * weight.weight;
    }
    let macro_shape =
        (fields.continentality - 0.5) * 0.40 - fields.erosion * 0.18 + fields.weirdness * 0.08;
    height_offset += macro_shape;
    // `profile.max_terrain_offset` is already scaled by the profile builder
    // so mountains keep their authored physical relief regardless of voxel
    // size.
    let layer = profile.surface_layer as i32
        + (height_offset * profile.max_terrain_offset as f32).round() as i32;
    let min_layer = profile.core_layers.saturating_add(2) as i32;
    let max_layer = profile.resolution.saturating_sub(3) as i32;
    (layer.clamp(min_layer, max_layer) as u32, primary)
}

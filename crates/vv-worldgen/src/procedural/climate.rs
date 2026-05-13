//! Climate-axis sampling.
//!
//! `SurfaceFields` is the bag of normalized 0..1 values that drive every
//! downstream decision (biome selection, height composition).  Each axis is
//! a latitude bias plus a weighted sum of named noise fields, exactly as
//! authored in the climate RON.  `roughness` is derived from `erosion` and
//! `weirdness` so packs that don't define a dedicated roughness axis still
//! get the values the biome selector expects.

use super::noise_sampler::sample_noise_field;
use vv_pack_compiler::{CompiledClimateAxis, CompiledProceduralPlanet, ProceduralRegistry};
use crate::noise::NoiseGenerator;
use glam::Vec3;

#[derive(Clone, Copy)]
pub(super) struct SurfaceFields {
    pub temperature: f32,
    pub humidity: f32,
    pub roughness: f32,
    pub continentality: f32,
    pub erosion: f32,
    pub weirdness: f32,
}

pub(super) fn sample_surface_fields(
    registry: &ProceduralRegistry,
    generators: &[NoiseGenerator],
    planet: &CompiledProceduralPlanet,
    dir: Vec3,
) -> SurfaceFields {
    let climate = &registry.climates[planet.climate];
    let latitude = dir.y.abs();
    let temperature = sample_axis(registry, generators, &climate.temperature, dir, latitude);
    let humidity = sample_axis(registry, generators, &climate.humidity, dir, latitude);
    let continentality = sample_axis(registry, generators, &climate.continentality, dir, latitude);
    let erosion = sample_axis(registry, generators, &climate.erosion, dir, latitude);
    let weirdness = sample_axis(registry, generators, &climate.weirdness, dir, latitude);
    let roughness = ((1.0 - erosion) * 0.65 + weirdness * 0.35).clamp(0.0, 1.0);
    SurfaceFields {
        temperature,
        humidity,
        roughness,
        continentality,
        erosion,
        weirdness,
    }
}

fn sample_axis(
    registry: &ProceduralRegistry,
    generators: &[NoiseGenerator],
    axis: &CompiledClimateAxis,
    dir: Vec3,
    latitude: f32,
) -> f32 {
    let mut value = 0.5 + (1.0 - latitude - 0.5) * axis.latitude_bias + axis.ocean_bias;
    for (field, weight) in &axis.fields {
        value += (sample_noise_field(registry, generators, *field, dir, 0) - 0.5) * *weight;
    }
    value.clamp(0.0, 1.0)
}



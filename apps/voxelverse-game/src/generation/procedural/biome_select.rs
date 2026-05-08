//! Climate-space biome resolution.
//!
//! Every selector defines a hyper-rectangle in normalized 6-D climate space
//! (T/H/R + optional C/E/W).  A position's distance to that rectangle is the
//! Euclidean distance in unit space; biomes whose distance is smaller than
//! `blend_radius` contribute a weight, and the top
//! [`MAX_BIOME_WEIGHTS`](super::MAX_BIOME_WEIGHTS) winners are normalized to
//! sum to 1.  `None` axes contribute zero distance (the biome accepts any
//! value on that axis), keeping older 3-axis packs valid.

use super::climate::SurfaceFields;
use super::{BiomeWeight, MAX_BIOME_WEIGHTS};
use crate::content::{CompiledBiomeSelector, CompiledProceduralPlanet, ProceduralRegistry};

pub(super) fn resolve_biome_weights(
    registry: &ProceduralRegistry,
    planet: &CompiledProceduralPlanet,
    fields: SurfaceFields,
) -> (Vec<BiomeWeight>, usize) {
    let set = &registry.biome_sets[planet.biome_set];

    let mut weights: Vec<BiomeWeight> = set
        .selectors
        .iter()
        .filter_map(|selector| {
            let d = selector_distance(selector, &fields);
            let w = ((set.blend_radius - d) / set.blend_radius).clamp(0.0, 1.0) * selector.weight;
            (w > 0.0).then_some(BiomeWeight {
                biome: selector.biome,
                weight: w,
            })
        })
        .collect();

    if weights.is_empty() {
        let fallback = set
            .selectors
            .iter()
            .min_by(|a, b| {
                selector_total_dist(a, &fields)
                    .partial_cmp(&selector_total_dist(b, &fields))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.biome)
            .unwrap_or(0);
        weights.push(BiomeWeight {
            biome: fallback,
            weight: 1.0,
        });
    }

    weights.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    weights.truncate(MAX_BIOME_WEIGHTS);
    let total = weights.iter().map(|w| w.weight).sum::<f32>().max(0.001);
    for weight in &mut weights {
        weight.weight /= total;
    }
    let primary = weights.first().map(|w| w.biome).unwrap_or(0);
    (weights, primary)
}

/// Euclidean distance from `fields` to the selector's accept window in
/// normalized climate space.  Optional axes (C/E/W) skipped when `None`.
fn selector_distance(selector: &CompiledBiomeSelector, fields: &SurfaceFields) -> f32 {
    let dt = range_distance(fields.temperature, selector.temperature);
    let dh = range_distance(fields.humidity, selector.humidity);
    let dr = range_distance(fields.roughness, selector.roughness);
    let dc = selector
        .continentality
        .map(|r| range_distance(fields.continentality, r))
        .unwrap_or(0.0);
    let de = selector
        .erosion
        .map(|r| range_distance(fields.erosion, r))
        .unwrap_or(0.0);
    let dw = selector
        .weirdness
        .map(|r| range_distance(fields.weirdness, r))
        .unwrap_or(0.0);
    (dt * dt + dh * dh + dr * dr + dc * dc + de * de + dw * dw).sqrt()
}

/// L1 distance — used only as the tie-breaker for the empty-weights
/// fallback path (picks the closest selector regardless of blend radius).
fn selector_total_dist(s: &CompiledBiomeSelector, fields: &SurfaceFields) -> f32 {
    range_distance(fields.temperature, s.temperature)
        + range_distance(fields.humidity, s.humidity)
        + range_distance(fields.roughness, s.roughness)
        + s.continentality
            .map(|r| range_distance(fields.continentality, r))
            .unwrap_or(0.0)
        + s.erosion
            .map(|r| range_distance(fields.erosion, r))
            .unwrap_or(0.0)
        + s.weirdness
            .map(|r| range_distance(fields.weirdness, r))
            .unwrap_or(0.0)
}

fn range_distance(value: f32, range: (f32, f32)) -> f32 {
    if value < range.0 {
        range.0 - value
    } else if value > range.1 {
        value - range.1
    } else {
        0.0
    }
}

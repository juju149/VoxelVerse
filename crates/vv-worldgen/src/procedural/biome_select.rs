//! Climate-space biome resolution.
//!
//! Every selector defines a hyper-rectangle in normalized 6-D climate space
//! (T/H/R + optional C/E/W).  A position's distance to that rectangle is the
//! Euclidean distance in unit space; biomes whose distance is smaller than
//! `blend_radius` contribute a weight, and the top
//! [`MAX_BIOME_WEIGHTS`](super::MAX_BIOME_WEIGHTS) winners are normalized to
//! sum to 1.  `None` axes contribute zero distance (the biome accepts any
//! value on that axis), keeping older 3-axis packs valid.
//!
//! The resolver writes into a caller-provided fixed-size buffer so the hot
//! path never allocates.  Adjacent biomes' contributions feed continuous
//! transitions in [`super::height`] and the upcoming feature-density
//! feathering — eliminating the visible "forest cut in a straight line" at
//! biome boundaries.

use super::climate::SurfaceFields;
use super::{BiomeWeight, MAX_BIOME_WEIGHTS};
use vv_pack_compiler::{CompiledBiomeSelector, CompiledProceduralPlanet, ProceduralRegistry};

/// Resolve weighted biome contributions into a caller-provided buffer.
/// Returns `(count, primary_biome_index)` where `count` is the number of
/// valid entries written from index 0.  Output entries are normalized to
/// sum to 1 and sorted descending by weight.
pub(super) fn resolve_biome_weights_into(
    registry: &ProceduralRegistry,
    planet: &CompiledProceduralPlanet,
    fields: SurfaceFields,
    out: &mut [BiomeWeight; MAX_BIOME_WEIGHTS],
) -> (u8, usize) {
    let set = &registry.biome_sets[planet.biome_set];

    // Insert-sorted top-N reservoir keyed on weight (descending).  Avoids a
    // full Vec + sort when only the strongest 4 contributors matter.
    let mut count: usize = 0;
    for selector in &set.selectors {
        let d = selector_distance(selector, &fields);
        if d >= set.blend_radius {
            continue;
        }
        // Smoothstep falloff inside the blend radius — biomes contribute
        // more strongly near their centre and feather out at the edge.
        let raw = ((set.blend_radius - d) / set.blend_radius).clamp(0.0, 1.0);
        let weight = (raw * raw * (3.0 - 2.0 * raw)) * selector.weight;
        if weight <= 0.0 {
            continue;
        }
        let new_entry = BiomeWeight {
            biome: selector.biome as u16,
            weight,
        };
        insert_sorted(out, &mut count, new_entry);
    }

    if count == 0 {
        // No selector matched within the blend radius — pick the closest one
        // overall and emit a single full-weight entry.  This keeps the
        // pipeline deterministic even for "alien" climate combinations.
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
        out[0] = BiomeWeight {
            biome: fallback as u16,
            weight: 1.0,
        };
        return (1, fallback);
    }

    // Normalize to sum to 1.0.
    let total = out[..count]
        .iter()
        .map(|w| w.weight)
        .sum::<f32>()
        .max(0.001);
    for entry in &mut out[..count] {
        entry.weight /= total;
    }
    let primary = out[0].biome as usize;
    (count as u8, primary)
}

/// Insert `entry` into the descending-by-weight reservoir.  Drops the
/// smallest contributor if the buffer is already full.
fn insert_sorted(
    out: &mut [BiomeWeight; MAX_BIOME_WEIGHTS],
    count: &mut usize,
    entry: BiomeWeight,
) {
    let mut insert_at = *count;
    for (i, existing) in out.iter().take(*count).enumerate() {
        if entry.weight > existing.weight {
            insert_at = i;
            break;
        }
    }
    if insert_at == MAX_BIOME_WEIGHTS {
        return; // weaker than every existing contributor, drop.
    }
    let end = (*count).min(MAX_BIOME_WEIGHTS - 1);
    for i in (insert_at..end).rev() {
        out[i + 1] = out[i];
    }
    out[insert_at] = entry;
    if *count < MAX_BIOME_WEIGHTS {
        *count += 1;
    }
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



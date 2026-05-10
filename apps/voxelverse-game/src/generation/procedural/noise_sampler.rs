//! Noise-field evaluator with domain-warp recursion and post-remap.
//!
//! Reads a [`CompiledNoiseField`] description and runs the underlying
//! `NoiseGenerator`, then optionally warps the input position by another
//! field (capped at depth 4 so chains of warps can't recurse forever) and
//! applies a curve-aware remap to the output.
//!
//! # Domain Warp
//! Three independent noise samples (at offset positions in world space) are
//! used for the X, Y, Z displacement axes so there is no directional
//! correlation between axes.  One-sample warp (the old approach, using
//! `warp * [1, 0.73, 1.37]`) creates banding because all three axes share
//! the same underlying warp pattern.

use crate::content::{CompiledCurve, CompiledNoiseField, CompiledNoiseKind, ProceduralRegistry};
use crate::generation::noise::{NoiseGenerator, NoiseSettings, NoiseType};
use glam::Vec3;

pub(super) fn sample_noise_field(
    registry: &ProceduralRegistry,
    generators: &[NoiseGenerator],
    field_idx: usize,
    pos: Vec3,
    depth: u32,
) -> f32 {
    let field: &CompiledNoiseField = &registry.fields[field_idx];
    if matches!(field.kind, CompiledNoiseKind::Constant) {
        return field.amplitude.clamp(0.0, 1.0);
    }

    let mut sample_pos = pos;
    if let Some((warp_idx, strength)) = field.domain_warp {
        if depth < 4 {
            // Three independent noise samples — different world-space offsets for
            // each axis so there is no directional correlation between X, Y and Z.
            // The offsets are large prime-ish values to avoid any periodicity.
            let wx = sample_noise_field(
                registry, generators, warp_idx,
                pos + Vec3::new(0.0, 71.3, 13.7), depth + 1,
            ) * 2.0 - 1.0;
            let wy = sample_noise_field(
                registry, generators, warp_idx,
                pos + Vec3::new(83.7, 0.0, 31.9), depth + 1,
            ) * 2.0 - 1.0;
            let wz = sample_noise_field(
                registry, generators, warp_idx,
                pos + Vec3::new(29.4, 47.6, 0.0), depth + 1,
            ) * 2.0 - 1.0;
            sample_pos += Vec3::new(wx, wy, wz) * strength;
        }
    }

    let noise_type = match field.kind {
        // Both Simplex and OpenSimplex2S use the 3-D simplex kernel.
        CompiledNoiseKind::Simplex | CompiledNoiseKind::OpenSimplex2S => NoiseType::OpenSimplex2S,
        // Ridged and Cellular use the spectral ridged multifractal.
        CompiledNoiseKind::Ridged | CompiledNoiseKind::Cellular => NoiseType::RidgedMulti,
        // Perlin kept for explicit opt-in (legacy packs).
        _ => NoiseType::Perlin,
    };
    let settings = NoiseSettings {
        noise_type,
        frequency: field.frequency,
        amplitude: field.amplitude,
        octaves: field.octaves,
        persistence: field.persistence,
        lacunarity: field.lacunarity,
        offset: Vec3::ZERO,
    };
    let mut value = generators[field_idx].compute(sample_pos, &settings) * field.amplitude;
    if let Some(remap) = &field.remap {
        let denom = (remap.in_max - remap.in_min).abs().max(0.0001);
        let mut t = ((value - remap.in_min) / denom).clamp(0.0, 1.0);
        if matches!(remap.curve, CompiledCurve::Smoothstep) {
            t = t * t * (3.0 - 2.0 * t);
        }
        value = remap.out_min + (remap.out_max - remap.out_min) * t;
    }
    value.clamp(0.0, 1.0)
}

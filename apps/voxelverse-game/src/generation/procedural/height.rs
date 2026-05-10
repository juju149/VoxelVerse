//! Per-biome height composition — geological layering approach.
//!
//! The height at any surface point is composed as a geological "lasagna":
//!
//! ```text
//! final_height =
//!     continent_curve(continentality)     // ocean / coast / land separation
//!   + biome_blend(hills_fbm, ridge_fbm)   // local terrain character per biome
//!   + mountain_boost(roughness, erosion)  // mountain chains where terrain is active
//!   + erosion_modifier                    // older terrain sits lower and flatter
//!   + weirdness_jitter                    // small local variation
//! ```
//!
//! This replaces the old linear `macro_shape` formula which produced
//! ocean/land bands at constant latitude rings.  The non-linear continent
//! curve creates distinct ocean / coastal / continental zones whose
//! boundaries are controlled by the continent noise field (already
//! domain-warped independently from temperature/humidity).

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

    // ── Layer 1: Biome blend ─────────────────────────────────────────────────
    // Each biome contributes hill noise + optional ridge noise scaled by its
    // amplitude and flatness.  The weighted sum drives local terrain character.
    let mut height_offset = 0.0_f32;
    for weight in &weights {
        let biome = registry.biome(weight.biome);
        let hill =
            sample_noise_field(registry, generators, biome.terrain.hill_field, dir, 0) * 2.0 - 1.0;
        let ridge = biome
            .terrain
            .ridge_field
            .map(|f| sample_noise_field(registry, generators, f, dir, 0) * 2.0 - 1.0)
            .unwrap_or(0.0);
        let flat = 1.0 - biome.terrain.flatness;
        let mut local = biome.terrain.base_height
            + hill * biome.terrain.amplitude * flat
            + ridge * biome.terrain.amplitude * 0.35;

        // Terracing with spatial phase so steps tilt rather than being horizontal.
        if biome.terrain.terrace_strength > 0.0 {
            let steps = 32.0_f32;
            let phase = (dir.x * 4.73 + dir.z * 3.17 + dir.y * 2.57).fract() * (1.0 / steps);
            let shifted = local + phase;
            let terraced = (shifted * steps).round() / steps - phase;
            local += (terraced - local) * biome.terrain.terrace_strength;
        }
        height_offset += local * weight.weight;
    }

    // ── Layer 2: Continental baseline ───────────────────────────────────────
    // Non-linear S-curve on continentality creates distinct zones:
    //   < 0.38 → ocean (strongly negative, deep water baseline)
    //   0.38–0.52 → coastal transition (smooth crossing of sea level)
    //   > 0.52 → continental land (gently rising with continentality)
    //
    // The boundaries of these zones are curved in world space because the
    // continentality field uses a different domain warp from temperature and
    // humidity — so ocean/land borders are never parallel to climate bands.
    let c = fields.continentality;
    let continent_baseline = if c < 0.38 {
        // Deep ocean: steeply negative so terrain stays submerged.
        (c / 0.38 - 1.0) * 0.38
    } else if c < 0.52 {
        // Coastal S-curve: smoothly crosses zero near the shoreline.
        let t = (c - 0.38) / 0.14;
        let t = t * t * (3.0 - 2.0 * t); // smoothstep
        -0.08 + t * 0.22
    } else {
        // Continental interior: gently rising, giving space for valleys/plains.
        0.14 + (c - 0.52) * 0.72
    };

    // ── Layer 3: Mountain boost ──────────────────────────────────────────────
    // Mountains appear where terrain is both rough AND young (low erosion)
    // AND inland (high continentality).  All three conditions must be met.
    //
    // This avoids mountains on coasts, on flat old peneplains, and in deserts
    // — matching real-world tectonic geography.
    let inland = (c - 0.42).clamp(0.0, 0.58) / 0.58; // 0 at coast, 1 inland
    let active = (1.0 - fields.erosion).max(0.0); // 1 = fresh terrain
    let rough = fields.roughness; // 0..1
    let mountain_potential = inland * active * rough; // 0..1
                                                      // Cubic sharpen so the boost only kicks in where all three are high.
    let mountain_boost = mountain_potential * mountain_potential * mountain_potential * 0.52;

    // ── Layer 4: Erosion modifier ────────────────────────────────────────────
    // Heavily eroded terrain is lower and flatter — ancient cratons and
    // peneplains.  Slightly reduces the biome height as well.
    let erosion_mod = -fields.erosion * 0.16;

    // ── Layer 5: Weirdness jitter ────────────────────────────────────────────
    // Small random variation that breaks any remaining regularity.
    let weirdness_mod = (fields.weirdness - 0.5) * 0.07;

    height_offset += continent_baseline + mountain_boost + erosion_mod + weirdness_mod;

    // ── Final layer index ────────────────────────────────────────────────────
    let layer = profile.surface_layer as i32
        + (height_offset * profile.max_terrain_offset as f32).round() as i32;
    let min_layer = profile.core_layers.saturating_add(2) as i32;
    let max_layer = profile.resolution.saturating_sub(3) as i32;
    (layer.clamp(min_layer, max_layer) as u32, primary)
}

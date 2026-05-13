//! Per-biome height composition — geological layering with data-driven
//! curves and biome-weighted mountain boost.
//!
//! The height at any surface point is composed as a geological "lasagna":
//!
//! ```text
//! final_height =
//!     continent_curve(continentality)            // ocean / coast / land
//!   + Σ_biome weight × curve_b(hills_fbm + ridge_fbm) × amplitude_b
//!   + Σ_biome weight × mountain_intensity_b × mountain_boost
//!   + erosion_modifier                            // older terrain sits lower
//!   + weirdness_jitter                            // small local variation
//! ```
//!
//! Mountains are no longer a single hard cubic of `(inland × active × rough)`
//! — that produced abrupt walls when any of the three fields jumped.  The
//! boost now uses a smoothstep with a soft floor so the contribution rises
//! smoothly from inland edges, and biomes opt into it via
//! `mountain_intensity` (0 = flat biomes never get mountainsides, 1 =
//! standard, >1 = exaggerated).  Each biome's *shape* (peak, plateau,
//! plains) is driven by its `height_curve`.

use super::biome_select::resolve_biome_weights_into;
use super::climate::SurfaceFields;
use super::noise_sampler::sample_noise_field;
use super::{BiomeWeight, MAX_BIOME_WEIGHTS};
use crate::noise::NoiseGenerator;
use glam::Vec3;
use vv_pack_compiler::{CompiledProceduralPlanet, ProceduralRegistry};
use vv_voxel::PlanetProfile;

pub(super) fn resolve_height(
    registry: &ProceduralRegistry,
    generators: &[NoiseGenerator],
    planet: &CompiledProceduralPlanet,
    profile: PlanetProfile,
    dir: Vec3,
    fields: &SurfaceFields,
) -> (u32, usize) {
    let mut weights = [BiomeWeight::default(); MAX_BIOME_WEIGHTS];
    let (count, primary) = resolve_biome_weights_into(registry, planet, *fields, &mut weights);
    let active_weights = &weights[..count as usize];

    // ── Layer 1: Biome blend ─────────────────────────────────────────────────
    // Each biome contributes hill+ridge noise reshaped by its height curve.
    // The weighted sum drives local terrain character, with curves controlling
    // silhouette per biome (plains stay flat, alpine spikes, badlands plateau).
    let mut height_offset = 0.0_f32;
    let mut mountain_intensity = 0.0_f32;
    for weight in active_weights {
        let biome = registry.biome(weight.biome as usize);
        let hill =
            sample_noise_field(registry, generators, biome.terrain.hill_field, dir, 0) * 2.0 - 1.0;
        let ridge = biome
            .terrain
            .ridge_field
            .map(|f| sample_noise_field(registry, generators, f, dir, 0) * 2.0 - 1.0)
            .unwrap_or(0.0);
        let flat = 1.0 - biome.terrain.flatness;

        // Apply the biome's height curve to the combined hill/ridge signal
        // before scaling.  The curve operates on the [-1,1] unit-range
        // pre-amplitude value so authoring stays predictable across biomes.
        let raw_signal = (hill * flat + ridge * 0.35).clamp(-1.5, 1.5);
        let shaped = biome.terrain.height_curve.evaluate(raw_signal);

        let mut local = biome.terrain.base_height + shaped * biome.terrain.amplitude;

        // Terracing with spatial phase so steps tilt rather than being horizontal.
        if biome.terrain.terrace_strength > 0.0 {
            let steps = 32.0_f32;
            let phase = (dir.x * 4.73 + dir.z * 3.17 + dir.y * 2.57).fract() * (1.0 / steps);
            let shifted = local + phase;
            let terraced = (shifted * steps).round() / steps - phase;
            local += (terraced - local) * biome.terrain.terrace_strength;
        }
        height_offset += local * weight.weight;
        mountain_intensity += biome.terrain.mountain_intensity * weight.weight;
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
    // Mountains appear where:
    //   - terrain is inland (continentality high)
    //   - terrain is active (low erosion)
    //   - terrain is rough (low erosion + high weirdness)
    //   - the contributing biome opts in via mountain_intensity
    //
    // The boost is a smoothstep on the combined potential rather than a hard
    // cubic — that kills the abrupt walls the cubic produced when any single
    // field jumped.  A small floor (0.04) keeps subtle relief on inland
    // plateaus without breaking the "mountain biome only" silhouette.
    let inland = ((c - 0.42) / 0.58).clamp(0.0, 1.0);
    let active = (1.0 - fields.erosion).clamp(0.0, 1.0);
    let rough = fields.roughness.clamp(0.0, 1.0);
    let potential = inland * active * rough;
    // Smoothstep keeps low-potential areas flat without snapping high-
    // potential ones into vertical cliffs.
    let shaped_potential = potential * potential * (3.0 - 2.0 * potential);
    let mountain_boost = (shaped_potential * 0.42 + potential * 0.04) * mountain_intensity;

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

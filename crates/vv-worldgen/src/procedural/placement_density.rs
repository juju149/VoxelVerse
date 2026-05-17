//! Placement density gating: scatter-field × clump × authored density × roll.
//!
//! Called by every per-chunk scatter (vegetation, props, cave decoration) so
//! the same density math is shared. Voxel-size compensation (`density_scale`,
//! `voxel_scale`) lives here so authors only ever think in physical units.

use super::{noise_sampler, ProceduralPlanetTerrain, WORLD_SCALE_BASELINE_METERS};
use vv_math::CoordSystem;

impl ProceduralPlanetTerrain {
    /// Density gate evaluated at a placement candidate's sub-voxel position.
    /// Scatter-field × optional clump-field × authored density, gated by a
    /// candidate-specific RNG roll.  Keeps the scatter pattern continuous
    /// (no chunk-aligned hash grid) and adds biome-scale clump/clearing
    /// modulation when `clump_field` is set.
    pub fn placement_density_hit(
        &self,
        placement: &vv_pack_compiler::CompiledFeaturePlacement,
        face: u8,
        candidate: &crate::placement::PlacementCandidate,
    ) -> bool {
        let res = self.voxel_res;
        // Direction at the jittered sub-voxel position so the scatter
        // field is sampled continuously, not on integer cells.
        let dir = CoordSystem::get_direction(
            face,
            (candidate.pu_f.round() as u32).min(res.saturating_sub(1)),
            (candidate.pv_f.round() as u32).min(res.saturating_sub(1)),
            res,
        );
        let cluster = noise_sampler::sample_noise_field(
            &self.registry,
            &self.noise_generators,
            placement.field,
            dir,
            0,
        );
        let clump = match placement.clump_field {
            Some(idx) => noise_sampler::sample_noise_field(
                &self.registry,
                &self.noise_generators,
                idx,
                dir,
                0,
            ),
            None => 0.5,
        };
        let strength = placement.clump_strength.clamp(0.0, 1.0);
        // lerp(1, clump×2, strength) — clump >0.5 boosts density, <0.5 cuts it.
        let modulator = (1.0 - strength) + strength * (clump * 2.0);
        // Density compensation: candidate iteration runs once per placement
        // cell (≥ 1 voxel), so authored "per 1 m² cell" densities must shrink
        // with cell area.  density_scale handles the voxel-size axis; the
        // cell side cancels here because both numerator and denominator
        // scale with it.
        let density = placement.density.clamp(0.0, 1.0) * self.density_scale();
        let effective = (density * cluster * modulator * 2.0).clamp(0.0, 1.0);
        let roll = (candidate.seed as f32) / (u32::MAX as f32);
        roll < effective
    }

    /// Multiplier applied to every voxel-counted quantity authored in the
    /// procedural pack so that physical world size stays constant when
    /// `voxel_size_meters` shrinks.  RON values are written assuming a 1 m
    /// baseline; at 0.5 m voxels this returns 2.0 → trees, soil layers, ore
    /// veins all double in voxel count but keep their physical dimensions.
    pub fn voxel_scale(&self) -> f32 {
        WORLD_SCALE_BASELINE_METERS / self.planet().base.voxel_size_meters.max(0.0001)
    }

    /// Density compensation: `feature_hit` runs once per `(u, v)` cell.  When
    /// voxels shrink the cell grid densifies quadratically, so authored
    /// per-cell densities must shrink by the same factor to preserve the
    /// physical "trees per m²" the pack writer intended.
    pub fn density_scale(&self) -> f32 {
        let voxel_m = self.planet().base.voxel_size_meters.max(0.0001);
        (voxel_m / WORLD_SCALE_BASELINE_METERS).powi(2)
    }
}

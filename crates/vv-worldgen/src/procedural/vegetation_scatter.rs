//! Per-chunk vegetation feature stamping (trees, large flora).
//!
//! Iterates the planet's vegetation sets and emits `FeatureStamp::Tree`
//! entries for every placement candidate that passes biome / surface /
//! density gating. The result is consumed by the chunk feature bakery.

use super::{range_pick, FeatureStamp, ProceduralPlanetTerrain};
use vv_voxel::{SurfaceChunkKey, VoxelCoord, CHUNK_SIZE};

impl ProceduralPlanetTerrain {
    pub fn features_for_chunk(&self, key: SurfaceChunkKey) -> Vec<FeatureStamp> {
        let planet = self.planet();
        let mut stamps = Vec::new();
        let u0 = key.u_idx * CHUNK_SIZE;
        let v0 = key.v_idx * CHUNK_SIZE;
        let u1 = (u0 + CHUNK_SIZE).min(self.voxel_res);
        let v1 = (v0 + CHUNK_SIZE).min(self.voxel_res);
        let voxel_scale = self.voxel_scale();
        let budget = planet.streaming.feature_budget_per_chunk as usize;

        for veg_idx in &planet.vegetation_sets {
            if budget > 0 && stamps.len() >= budget {
                break;
            }
            let veg = &self.registry.vegetation[*veg_idx];
            crate::placement::for_each_candidate(
                &veg.placement,
                crate::placement::PlacementArea {
                    face: key.face,
                    u_lo: u0,
                    u_hi: u1,
                    v_lo: v0,
                    v_hi: v1,
                    voxel_scale,
                },
                |candidate| {
                    if budget > 0 && stamps.len() >= budget {
                        return;
                    }
                    let surface = self.surface_sample(key.face, candidate.pu, candidate.pv);
                    let biome = self.registry.biome(surface.primary_biome);
                    if !veg.placement.allowed_in_biome(biome) {
                        self.stats.record_reject();
                        return;
                    }
                    if !veg.placement.surface_blocks.contains(&biome.surface.top) {
                        self.stats.record_reject();
                        return;
                    }
                    if !self.placement_density_hit(&veg.placement, key.face, &candidate) {
                        return;
                    }
                    let coord = VoxelCoord {
                        face: key.face,
                        layer: surface.height.saturating_add(1),
                        u: candidate.pu,
                        v: candidate.pv,
                    };
                    let h = range_pick(veg.height, candidate.seed);
                    let r = range_pick(veg.canopy_radius, candidate.seed.rotate_left(11));
                    stamps.push(FeatureStamp::Tree {
                        coord,
                        trunk: veg.trunk,
                        leaves: veg.leaves,
                        height: h,
                        canopy_radius: r,
                        priority: 30,
                    });
                    self.stats.record_feature();
                },
            );
        }

        stamps
    }
}

//! Per-chunk feature baker.
//!
//! The slow path in [`super::procedural::ProceduralPlanetTerrain`] resolves a
//! single voxel by scanning the neighbourhood for any tree whose canopy might
//! reach it.  That is fine for occasional collision / raycast lookups but
//! becomes a hot loop when the mesher asks the same question for thousands of
//! voxels per chunk.
//!
//! This module inverts the loop: for one chunk extent we iterate every
//! candidate planting column once, stamp each tree (trunk + branches +
//! canopy) and each visual-detail block into a sparse map, then hand the map
//! back to the mesher.  The mesher does map look-ups in O(1) instead of an
//! O(R²) neighbourhood scan per voxel.
//!
//! Single-responsibility: this file owns "given a chunk, produce its feature
//! voxels". Trunk / canopy geometry rules live here too — they used to be
//! duplicated inside [`ProceduralPlanetTerrain::evaluate_tree_at`].  The slow
//! path now delegates to these helpers as well.

use crate::content::{
    CompiledProceduralPlanet, CompiledVegetation, ProceduralRegistry,
};
use crate::generation::procedural::ProceduralPlanetTerrain;
use crate::voxel::{VoxelCoord, VoxelId, CHUNK_SIZE};
use std::collections::HashMap;

#[path = "tree_shape.rs"]
mod tree_shape;
pub use tree_shape::TreeShape;

/// Sparse map of feature-driven voxels covering one chunk + a small margin
/// for face-culling neighbour lookups.
#[derive(Clone, Debug, Default)]
pub struct ChunkFeatureMap {
    pub blocks: HashMap<VoxelCoord, VoxelId>,
    /// Highest layer (exclusive) covered by any feature in this map. Lets the
    /// mesher know how far up to walk when collecting candidates.
    pub max_layer_exclusive: u32,
}

impl ChunkFeatureMap {
    pub fn get(&self, coord: VoxelCoord) -> Option<VoxelId> {
        self.blocks.get(&coord).copied()
    }
}

/// Produces a [`ChunkFeatureMap`] for a given chunk extent.
pub struct FeatureBakery<'a> {
    terrain: &'a ProceduralPlanetTerrain,
    registry: &'a ProceduralRegistry,
    planet: &'a CompiledProceduralPlanet,
}

impl<'a> FeatureBakery<'a> {
    pub fn new(terrain: &'a ProceduralPlanetTerrain) -> Self {
        let registry = terrain.registry();
        let planet = &registry.planets[terrain.planet_index()];
        Self {
            terrain,
            registry,
            planet,
        }
    }

    fn voxel_scale(&self) -> f32 {
        self.terrain.voxel_scale()
    }

    fn density_scale(&self) -> f32 {
        self.terrain.density_scale()
    }

    /// Bake every tree voxel that overlaps the rectangle
    /// `[u_lo, u_hi) × [v_lo, v_hi)` (plus `margin` for face culling).
    /// Props (vox props) are NOT part of the voxel feature map — query
    /// `ProceduralPlanetTerrain::props_for_chunk()` separately.
    pub fn bake_chunk(
        &self,
        face: u8,
        u_lo: u32,
        u_hi: u32,
        v_lo: u32,
        v_hi: u32,
        margin: u32,
    ) -> ChunkFeatureMap {
        let res = self.terrain.voxel_res();
        let mut map = ChunkFeatureMap::default();

        let scan_u_lo = u_lo.saturating_sub(self.max_veg_radius() + margin);
        let scan_v_lo = v_lo.saturating_sub(self.max_veg_radius() + margin);
        let scan_u_hi = (u_hi + self.max_veg_radius() + margin).min(res);
        let scan_v_hi = (v_hi + self.max_veg_radius() + margin).min(res);

        let chunk_u_lo = u_lo.saturating_sub(margin);
        let chunk_v_lo = v_lo.saturating_sub(margin);
        let chunk_u_hi = (u_hi + margin).min(res);
        let chunk_v_hi = (v_hi + margin).min(res);

        for veg_idx in &self.planet.vegetation_sets {
            let veg = &self.registry.vegetation[*veg_idx];
            for pu in scan_u_lo..scan_u_hi {
                for pv in scan_v_lo..scan_v_hi {
                    if !self.is_planted(veg, face, pu, pv) {
                        continue;
                    }
                    self.stamp_tree(
                        veg, face, pu, pv, chunk_u_lo, chunk_v_lo, chunk_u_hi, chunk_v_hi, &mut map,
                    );
                }
            }
        }

        map
    }

    fn max_veg_radius(&self) -> u32 {
        // Trees can reach beyond their plant column via canopy lobes (offset
        // up to canopy_r * 0.55 + lobe_r up to canopy_r * 1.0 ≈ 1.55× canopy_r),
        // branches with arbitrary angles, and a small horizontal lean.  Keep a
        // conservative bound so the bakery never misses an overhanging voxel.
        let scale = self.voxel_scale();
        self.planet
            .vegetation_sets
            .iter()
            .map(|i| {
                let v = &self.registry.vegetation[*i];
                let canopy = scale_range(v.canopy_radius, scale).1;
                let thickness = scale_range(v.trunk_thickness, scale).1.max(1);
                let branch = scale_range(v.branch_length, scale).1;
                let canopy_reach = ((canopy as f32) * 1.6).ceil() as u32;
                let branch_reach = branch + 2;
                canopy_reach.max(branch_reach) + thickness + 2
            })
            .max()
            .unwrap_or(0)
    }

    fn is_planted(&self, veg: &CompiledVegetation, face: u8, pu: u32, pv: u32) -> bool {
        let biome = self
            .registry
            .biome(self.terrain.get_biome_id(face, pu, pv) as usize);
        if !veg.placement.allowed_in_biome(biome) {
            return false;
        }
        let top = biome.surface.top;
        if !veg.placement.surface_blocks.contains(&top) {
            return false;
        }
        // Slope filter: skip steep terrain if slope_max is set.
        if veg.placement.slope_max > 0.0 {
            let res = self.terrain.voxel_res();
            let h0 = self.terrain.get_height(face, pu, pv) as i32;
            let h1 = self.terrain.get_height(face, (pu + 1).min(res - 1), pv) as i32;
            let h2 = self.terrain.get_height(face, pu, (pv + 1).min(res - 1)) as i32;
            let slope = (((h1 - h0).pow(2) + (h2 - h0).pow(2)) as f32).sqrt();
            if slope > veg.placement.slope_max {
                return false;
            }
        }
        // density is authored "per 1 m² cell"; rescale for the active grid.
        let density = veg.placement.density * self.density_scale();
        self.terrain
            .feature_hit_pub(veg.placement.field, face, pu, pv, density)
    }

    /// Stamp every voxel of one planted tree into `map`, but only the ones
    /// that fall inside the requested chunk rectangle.
    #[allow(clippy::too_many_arguments)]
    fn stamp_tree(
        &self,
        veg: &CompiledVegetation,
        face: u8,
        pu: u32,
        pv: u32,
        chunk_u_lo: u32,
        chunk_v_lo: u32,
        chunk_u_hi: u32,
        chunk_v_hi: u32,
        map: &mut ChunkFeatureMap,
    ) {
        let scale = self.voxel_scale();
        let plant_height = self.terrain.get_height(face, pu, pv);
        let shape = TreeShape::compute(veg, face, pu, pv, plant_height, scale);
        let res = self.terrain.voxel_res();

        let mut emit = |u: i32, layer: i32, v: i32, block: VoxelId| {
            if u < 0 || layer < 0 || v < 0 {
                return;
            }
            let (u, layer, v) = (u as u32, layer as u32, v as u32);
            if u < chunk_u_lo
                || u >= chunk_u_hi
                || v < chunk_v_lo
                || v >= chunk_v_hi
                || u >= res
                || v >= res
                || layer >= res
            {
                return;
            }
            let coord = VoxelCoord { face, layer, u, v };
            map.blocks.entry(coord).or_insert(block);
            if layer + 1 > map.max_layer_exclusive {
                map.max_layer_exclusive = layer + 1;
            }
        };

        shape.stamp(&mut emit);
    }
}

// ---- Hash + range primitives shared with procedural.rs --------------------

pub fn hash4(face: u8, u: u32, v: u32, salt: u32) -> u32 {
    let mut x = salt ^ (face as u32).wrapping_mul(0x9E37_79B9);
    x ^= u.wrapping_mul(0x85EB_CA6B).rotate_left(13);
    x ^= v.wrapping_mul(0xC2B2_AE35).rotate_right(7);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^ (x >> 16)
}

pub fn range_pick(range: (u32, u32), roll: u32) -> u32 {
    if range.0 == range.1 {
        range.0
    } else {
        range.0 + roll % (range.1 - range.0 + 1)
    }
}

/// Multiply both range endpoints by a floating-point scale and clamp to a sane
/// minimum.  Used to translate authored "voxels at 1 m baseline" into the
/// active planet's voxel grid.
pub fn scale_range(range: (u32, u32), scale: f32) -> (u32, u32) {
    let lo = ((range.0 as f32) * scale).round().max(0.0) as u32;
    let hi = ((range.1 as f32) * scale).round().max(lo as f32) as u32;
    (lo, hi)
}

/// Same idea for single counts (e.g. `core_layers`, `surface_layer`).
pub fn scale_count(value: u32, scale: f32) -> u32 {
    ((value as f32) * scale).round().max(0.0) as u32
}

/// Convenience helper used by the mesher when it needs the entire
/// `[u_idx*CHUNK_SIZE, (u_idx+1)*CHUNK_SIZE)` extent of a chunk plus a 1-voxel
/// margin so face culling can peek across chunk boundaries.
pub fn bake_for_chunk(
    terrain: &ProceduralPlanetTerrain,
    face: u8,
    u_idx: u32,
    v_idx: u32,
    margin: u32,
) -> ChunkFeatureMap {
    let u_lo = u_idx * CHUNK_SIZE;
    let v_lo = v_idx * CHUNK_SIZE;
    let res = terrain.voxel_res();
    let u_hi = (u_lo + CHUNK_SIZE).min(res);
    let v_hi = (v_lo + CHUNK_SIZE).min(res);
    FeatureBakery::new(terrain).bake_chunk(face, u_lo, u_hi, v_lo, v_hi, margin)
}

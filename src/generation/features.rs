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
    CompiledProceduralPlanet, CompiledVegetation, CompiledVisualDetail, ProceduralRegistry,
};
use crate::generation::procedural::ProceduralPlanetTerrain;
use crate::voxel::{VoxelCoord, VoxelId, CHUNK_SIZE};
use std::collections::HashMap;

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

    /// Bake every tree + visual-detail voxel that overlaps the rectangle
    /// `[u_lo, u_hi) × [v_lo, v_hi)` (plus `margin` for face culling).
    pub fn bake_chunk(&self, face: u8, u_lo: u32, u_hi: u32, v_lo: u32, v_hi: u32, margin: u32)
        -> ChunkFeatureMap
    {
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
                        veg,
                        face,
                        pu,
                        pv,
                        chunk_u_lo,
                        chunk_v_lo,
                        chunk_u_hi,
                        chunk_v_hi,
                        &mut map,
                    );
                }
            }
        }

        for detail_idx in &self.planet.visual_detail_sets {
            let detail = &self.registry.visual_details[*detail_idx];
            for u in chunk_u_lo..chunk_u_hi {
                for v in chunk_v_lo..chunk_v_hi {
                    if let Some(block) = self.detail_block_at(detail, face, u, v) {
                        let coord = VoxelCoord {
                            face,
                            layer: self.terrain.get_height(face, u, v).saturating_add(1),
                            u,
                            v,
                        };
                        // Trees take precedence over flowers.
                        map.blocks.entry(coord).or_insert(block);
                        if coord.layer + 1 > map.max_layer_exclusive {
                            map.max_layer_exclusive = coord.layer + 1;
                        }
                    }
                }
            }
        }

        map
    }

    fn max_veg_radius(&self) -> u32 {
        self.planet
            .vegetation_sets
            .iter()
            .map(|i| {
                let v = &self.registry.vegetation[*i];
                v.canopy_radius.1 + v.trunk_thickness.1.max(1) + v.branch_length.1
            })
            .max()
            .unwrap_or(0)
    }

    fn is_planted(&self, veg: &CompiledVegetation, face: u8, pu: u32, pv: u32) -> bool {
        let top = self
            .registry
            .biome(self.terrain.get_biome_id(face, pu, pv) as usize)
            .surface
            .top;
        if !veg.placement.surface_blocks.contains(&top) {
            return false;
        }
        self.terrain
            .feature_hit_pub(veg.placement.field, face, pu, pv, veg.placement.density)
    }

    fn detail_block_at(
        &self,
        detail: &CompiledVisualDetail,
        face: u8,
        u: u32,
        v: u32,
    ) -> Option<VoxelId> {
        let top = self
            .registry
            .biome(self.terrain.get_biome_id(face, u, v) as usize)
            .surface
            .top;
        if !detail.placement.surface_blocks.contains(&top) {
            return None;
        }
        if !self.terrain.feature_hit_pub(
            detail.placement.field,
            face,
            u,
            v,
            detail.placement.density,
        ) {
            return None;
        }
        weighted_detail(&detail.details, hash4(face, u, v, 17))
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
        let geom = TreeGeometry::compute(veg, face, pu, pv, self.terrain.get_height(face, pu, pv));

        let res = self.terrain.voxel_res();
        let half = geom.thickness as i32 / 2;

        let mut emit = |u: u32, v: u32, layer: u32, block: VoxelId| {
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
            let coord = VoxelCoord {
                face,
                layer,
                u,
                v,
            };
            map.blocks.entry(coord).or_insert(block);
            if layer + 1 > map.max_layer_exclusive {
                map.max_layer_exclusive = layer + 1;
            }
        };

        // Trunk
        let footprint_u_lo = (pu as i32) - half;
        let footprint_v_lo = (pv as i32) - half;
        let footprint_u_hi = (pu as i32) - half + geom.thickness as i32;
        let footprint_v_hi = (pv as i32) - half + geom.thickness as i32;
        for tu in footprint_u_lo..footprint_u_hi {
            for tv in footprint_v_lo..footprint_v_hi {
                if tu < 0 || tv < 0 {
                    continue;
                }
                for layer in (geom.plant_height + 1)..=geom.trunk_top_layer {
                    emit(tu as u32, tv as u32, layer, veg.trunk);
                }
            }
        }

        // Branches
        if geom.branch_count > 0 && geom.height >= 3 {
            let branch_band_start = geom.plant_height + (geom.height * 2 / 3).max(1);
            let branch_band_end = geom.trunk_top_layer.saturating_sub(1);
            if branch_band_end >= branch_band_start {
                let max_len = range_pick(veg.branch_length, hash4(face, pu, pv, 37)).max(1);
                let band_span = branch_band_end - branch_band_start + 1;
                for i in 0..geom.branch_count {
                    let h = hash4(face, pu, pv, 41 + i);
                    let dir = h & 0b11;
                    let len = 1 + (h >> 2) % max_len;
                    let layer_off = (h >> 6) % band_span;
                    let branch_layer = branch_band_start + layer_off;
                    let (axis_du, axis_dv): (i32, i32) = match dir {
                        0 => (1, 0),
                        1 => (-1, 0),
                        2 => (0, 1),
                        _ => (0, -1),
                    };
                    for step in 1..=(len as i32) {
                        let bu = pu as i32 + axis_du * step;
                        let bv = pv as i32 + axis_dv * step;
                        if bu < 0 || bv < 0 {
                            continue;
                        }
                        emit(bu as u32, bv as u32, branch_layer, veg.trunk);
                    }
                }
            }
        }

        // Canopy ellipsoid
        if geom.canopy_radius > 0 {
            let r = geom.canopy_radius as i32;
            let squash = veg.canopy_vertical_squash.max(0.1);
            let cx = pu as f32 + 0.5;
            let cy = pv as f32 + 0.5;
            let cz = geom.trunk_top_layer as f32 - 0.5;
            for du in -r..=r {
                for dv in -r..=r {
                    let cu = pu as i32 + du;
                    let cv = pv as i32 + dv;
                    if cu < 0 || cv < 0 {
                        continue;
                    }
                    let cu_u = cu as u32;
                    let cv_u = cv as u32;
                    if cu_u >= res || cv_u >= res {
                        continue;
                    }
                    let layer_lo = (geom.plant_height + 1).max(geom.trunk_top_layer.saturating_sub(r as u32));
                    let layer_hi = (geom.trunk_top_layer + r as u32).min(res.saturating_sub(1));
                    let voxel_jitter =
                        (hash4(face, cu_u, cv_u, 51) as f32 / u32::MAX as f32) * 0.45;
                    for layer in layer_lo..=layer_hi {
                        let dx = cu_u as f32 + 0.5 - cx;
                        let dy = cv_u as f32 + 0.5 - cy;
                        let dz = (layer as f32 + 0.5 - cz) / squash;
                        let dist_sq = dx * dx + dy * dy + dz * dz;
                        if dist_sq <= (geom.canopy_radius as f32) * (geom.canopy_radius as f32)
                            + voxel_jitter
                        {
                            emit(cu_u, cv_u, layer, veg.leaves);
                        }
                    }
                }
            }
        }
    }
}

/// Cached deterministic geometry for one planted tree.
#[derive(Clone, Copy)]
pub struct TreeGeometry {
    pub plant_height: u32,
    pub height: u32,
    pub thickness: u32,
    pub canopy_radius: u32,
    pub branch_count: u32,
    pub trunk_top_layer: u32,
}

impl TreeGeometry {
    pub fn compute(veg: &CompiledVegetation, face: u8, pu: u32, pv: u32, plant_height: u32) -> Self {
        let height = range_pick(veg.height, hash4(face, pu, pv, 33));
        let thickness = range_pick(veg.trunk_thickness, hash4(face, pu, pv, 35)).max(1);
        let canopy_radius = range_pick(veg.canopy_radius, hash4(face, pu, pv, 34));
        let branch_count = range_pick(veg.branch_count, hash4(face, pu, pv, 36));
        Self {
            plant_height,
            height,
            thickness,
            canopy_radius,
            branch_count,
            trunk_top_layer: plant_height + height,
        }
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

pub fn weighted_detail(
    items: &[crate::content::CompiledVisualDetailItem],
    roll: u32,
) -> Option<VoxelId> {
    let total = items.iter().map(|i| i.weight).sum::<u32>();
    if total == 0 {
        return None;
    }
    let mut pick = roll % total;
    for item in items {
        if pick < item.weight {
            return Some(item.block);
        }
        pick -= item.weight;
    }
    None
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

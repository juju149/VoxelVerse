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
use glam::Vec3;
use std::collections::HashMap;
use std::f32::consts::TAU;

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
        let top = self
            .registry
            .biome(self.terrain.get_biome_id(face, pu, pv) as usize)
            .surface
            .top;
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
        let density = detail.placement.density * self.density_scale();
        if !self
            .terrain
            .feature_hit_pub(detail.placement.field, face, u, v, density)
        {
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

// ---------------------------------------------------------------------------
// Tree shape — natural-looking generation with full per-tree randomness.
// ---------------------------------------------------------------------------

/// One blob in the canopy.  A canopy is a union of overlapping lobes whose
/// boundary is jittered per-voxel — never a pure sphere.
#[derive(Clone, Copy, Debug)]
pub struct CanopyLobe {
    pub center: Vec3, // x = u + 0.5, y = layer + 0.5, z = v + 0.5
    pub radius: f32,
    pub jitter_seed: u32,
}

/// One branch growing out of the trunk with an arbitrary horizontal angle and
/// a slight upward slope.  May carry a small canopy lobe at its tip.
#[derive(Clone, Copy, Debug)]
pub struct Branch {
    pub start: Vec3,
    pub direction: Vec3, // unit-ish: (cos θ, slope_y, sin θ)
    pub length: f32,
    pub thickness: f32,
    pub tip: Option<CanopyLobe>,
}

/// Full per-tree shape.  Computed once (deterministically from `(face, pu, pv)`
/// + a per-tree seed) so the bakery and the slow path always agree.
#[derive(Clone, Debug)]
pub struct TreeShape {
    pub plant_height: u32,
    pub trunk_top_layer: u32,
    pub height: u32,

    pub trunk_block: VoxelId,
    pub leaves_block: VoxelId,

    /// Pivot at the base of the trunk (centre of the planted column).
    pub trunk_pivot: Vec3,
    /// Horizontal lean at the very top (curves quadratically along height).
    pub trunk_lean: (f32, f32),
    pub trunk_base_radius: f32,
    pub trunk_top_radius: f32,
    /// Per-voxel boundary jitter scaler for the trunk (0..=1 fraction of r²).
    pub trunk_seed: u32,

    pub branches: Vec<Branch>,
    pub lobes: Vec<CanopyLobe>,
}

impl TreeShape {
    pub fn compute(
        veg: &CompiledVegetation,
        face: u8,
        pu: u32,
        pv: u32,
        plant_height: u32,
        scale: f32,
    ) -> Self {
        // -- per-tree seeds & dimensions ------------------------------------
        let height =
            range_pick(scale_range(veg.height, scale), hash4(face, pu, pv, 33)).max(2);
        let thickness = range_pick(
            scale_range(veg.trunk_thickness, scale),
            hash4(face, pu, pv, 35),
        )
        .max(1) as f32;
        let canopy_radius = range_pick(
            scale_range(veg.canopy_radius, scale),
            hash4(face, pu, pv, 34),
        ) as f32;
        let branch_count = range_pick(veg.branch_count, hash4(face, pu, pv, 36));
        let branch_len_max =
            range_pick(scale_range(veg.branch_length, scale), hash4(face, pu, pv, 37))
                .max(1) as f32;
        let trunk_seed = hash4(face, pu, pv, 0xA17EE5);

        let trunk_top_layer = plant_height + height;
        let pivot = Vec3::new(pu as f32 + 0.5, plant_height as f32 + 0.5, pv as f32 + 0.5);

        // -- trunk lean: data-driven max lean fraction of tree height -------
        let lean_max = (height as f32) * veg.trunk_lean_max;
        let lean_theta = hash01(face, pu, pv, 71) * TAU;
        let lean_amount = hash01(face, pu, pv, 72) * lean_max;
        let trunk_lean = (lean_theta.cos() * lean_amount, lean_theta.sin() * lean_amount);

        // Tapering: the top is narrower than the base. Always at least 0.5 v.
        let trunk_base_radius = (thickness * 0.5).max(0.5);
        let trunk_top_radius = (trunk_base_radius * 0.6).max(0.5);

        // -- branches at arbitrary angles -----------------------------------
        let mut branches = Vec::with_capacity(branch_count as usize);
        if branch_count > 0 && height >= 3 {
            let band_start = plant_height as f32 + (height as f32) * 0.55;
            let band_end = trunk_top_layer as f32 - 0.5;
            for i in 0..branch_count {
                let h = hash4(face, pu, pv, 401 + i);
                let theta = hash01_u32(h, 0) * TAU;
                let raw_len = (1.5 + hash01_u32(h, 1) * (branch_len_max + 1.0)).min(branch_len_max + 2.0);
                let layer_t = hash01_u32(h, 2);
                let start_layer = band_start + layer_t * (band_end - band_start).max(0.0);
                // Data-driven branch slope range (rise per horizontal voxel).
                let slope_lo = veg.branch_slope.0;
                let slope = slope_lo + hash01_u32(h, 3) * (veg.branch_slope.1 - slope_lo).max(0.0);
                // Branch starts at the trunk surface, not at the centerline.
                let dir_xz = Vec3::new(theta.cos(), 0.0, theta.sin());
                let trunk_r_at_start = trunk_radius_at(
                    start_layer,
                    plant_height as f32,
                    height as f32,
                    trunk_base_radius,
                    trunk_top_radius,
                );
                let start = Vec3::new(
                    pivot.x + dir_xz.x * trunk_r_at_start * 0.6,
                    start_layer,
                    pivot.z + dir_xz.z * trunk_r_at_start * 0.6,
                );
                let direction = Vec3::new(dir_xz.x, slope, dir_xz.z).normalize_or_zero();
                let thickness = (trunk_top_radius * 0.55).max(0.5);

                // Tip lobe: a soft leaf cluster at the end of the branch.
                let tip = Some(CanopyLobe {
                    center: start + direction * raw_len,
                    radius: (canopy_radius * 0.35 + hash01_u32(h, 4) * canopy_radius * 0.35).max(1.2),
                    jitter_seed: h ^ 0xBEEF,
                });
                branches.push(Branch {
                    start,
                    direction,
                    length: raw_len,
                    thickness,
                    tip,
                });
            }
        }

        // -- canopy lobes: data-driven count around the trunk top ----------
        let mut lobes = Vec::new();
        if canopy_radius > 0.0 {
            let n_lobes_seed = hash4(face, pu, pv, 0x10BE);
            let (lobe_min, lobe_max) = veg.canopy_lobe_count;
            let lobe_range = (lobe_max.saturating_sub(lobe_min) + 1).max(1);
            let n_lobes = lobe_min + n_lobes_seed % lobe_range;
            let canopy_anchor = Vec3::new(
                pivot.x + trunk_lean.0,
                trunk_top_layer as f32 - 0.25,
                pivot.z + trunk_lean.1,
            );
            for i in 0..n_lobes {
                let h = hash4(face, pu, pv, 0x501E + i);
                let theta = hash01_u32(h, 0) * TAU;
                let r_off = hash01_u32(h, 1) * canopy_radius * 0.55;
                let y_off = (hash01_u32(h, 2) - 0.35) * canopy_radius * veg.canopy_vertical_squash.max(0.3);
                let lobe_r = canopy_radius * (0.45 + hash01_u32(h, 3) * 0.55);
                let center = canopy_anchor
                    + Vec3::new(
                        theta.cos() * r_off,
                        y_off,
                        theta.sin() * r_off,
                    );
                lobes.push(CanopyLobe {
                    center,
                    radius: lobe_r.max(1.5),
                    jitter_seed: h ^ 0xCAFE,
                });
            }
        }

        Self {
            plant_height,
            trunk_top_layer,
            height,
            trunk_block: veg.trunk,
            leaves_block: veg.leaves,
            trunk_pivot: pivot,
            trunk_lean,
            trunk_base_radius,
            trunk_top_radius,
            trunk_seed,
            branches,
            lobes,
        }
    }

    /// Furthest a single tree can reach horizontally from its plant column.
    /// Used by the bakery to size its neighbourhood scan.
    #[allow(dead_code)]
    pub fn horizontal_reach(&self) -> f32 {
        let lean = (self.trunk_lean.0.powi(2) + self.trunk_lean.1.powi(2)).sqrt();
        let canopy = self
            .lobes
            .iter()
            .map(|l| {
                let dx = l.center.x - self.trunk_pivot.x;
                let dz = l.center.z - self.trunk_pivot.z;
                (dx * dx + dz * dz).sqrt() + l.radius
            })
            .fold(0.0_f32, f32::max);
        let branch = self
            .branches
            .iter()
            .map(|b| b.length + b.tip.map(|t| t.radius).unwrap_or(0.0))
            .fold(0.0_f32, f32::max);
        lean + canopy.max(branch).max(self.trunk_base_radius)
    }

    #[allow(dead_code)]
    pub fn vertical_reach_above(&self) -> f32 {
        let canopy = self
            .lobes
            .iter()
            .map(|l| (l.center.y + l.radius) - self.plant_height as f32)
            .fold(self.height as f32, f32::max);
        let branch = self
            .branches
            .iter()
            .map(|b| b.start.y + b.direction.y * b.length + 1.5)
            .map(|y| y - self.plant_height as f32)
            .fold(canopy, f32::max);
        branch
    }

    /// Walk every voxel that belongs to this tree and forward it to `emit`.
    /// The bakery uses this to populate the chunk feature map.
    pub fn stamp(&self, emit: &mut dyn FnMut(i32, i32, i32, VoxelId)) {
        // Trunk: layer-by-layer disc with lean, taper, and per-voxel jitter.
        for layer in (self.plant_height + 1)..=self.trunk_top_layer {
            let t = (layer - self.plant_height) as f32 / self.height.max(1) as f32;
            let cx = self.trunk_pivot.x + self.trunk_lean.0 * t * t;
            let cz = self.trunk_pivot.z + self.trunk_lean.1 * t * t;
            let r = trunk_radius_at(
                layer as f32,
                self.plant_height as f32,
                self.height as f32,
                self.trunk_base_radius,
                self.trunk_top_radius,
            );
            stamp_disc(cx, cz, layer as i32, r, self.trunk_seed ^ layer, self.trunk_block, emit);
        }

        // Branches: trunk-block voxels stepped along the branch direction.
        for (bi, branch) in self.branches.iter().enumerate() {
            let steps = (branch.length.max(1.0).ceil()) as i32;
            for s in 1..=steps {
                let p = branch.start + branch.direction * (s as f32);
                let r = branch.thickness * (1.0 - 0.4 * (s as f32 / steps as f32));
                stamp_disc(
                    p.x,
                    p.z,
                    p.y.round() as i32,
                    r.max(0.55),
                    (self.trunk_seed ^ 0x1234).wrapping_add((bi as u32).wrapping_mul(7919) + s as u32),
                    self.trunk_block,
                    emit,
                );
            }
            if let Some(tip) = branch.tip {
                stamp_lobe(tip, self.leaves_block, emit);
            }
        }

        // Canopy: union of jittered lobes.
        for lobe in &self.lobes {
            stamp_lobe(*lobe, self.leaves_block, emit);
        }
    }

    /// Test whether the voxel at `(u, layer, v)` belongs to this tree.
    /// Mirrors `stamp` exactly so the slow path agrees with the baked map.
    pub fn voxel_at(&self, u: i32, layer: i32, v: i32) -> Option<VoxelId> {
        // Trunk
        if (layer as u32) > self.plant_height && (layer as u32) <= self.trunk_top_layer {
            let l = layer as u32;
            let t = (l - self.plant_height) as f32 / self.height.max(1) as f32;
            let cx = self.trunk_pivot.x + self.trunk_lean.0 * t * t;
            let cz = self.trunk_pivot.z + self.trunk_lean.1 * t * t;
            let r = trunk_radius_at(
                layer as f32,
                self.plant_height as f32,
                self.height as f32,
                self.trunk_base_radius,
                self.trunk_top_radius,
            );
            if disc_contains(u, v, cx, cz, r, self.trunk_seed ^ l) {
                return Some(self.trunk_block);
            }
        }
        // Branches
        for (bi, branch) in self.branches.iter().enumerate() {
            let steps = (branch.length.max(1.0).ceil()) as i32;
            for s in 1..=steps {
                let p = branch.start + branch.direction * (s as f32);
                let r = (branch.thickness * (1.0 - 0.4 * (s as f32 / steps as f32))).max(0.55);
                if (p.y.round() as i32) == layer
                    && disc_contains(
                        u,
                        v,
                        p.x,
                        p.z,
                        r,
                        (self.trunk_seed ^ 0x1234)
                            .wrapping_add((bi as u32).wrapping_mul(7919) + s as u32),
                    )
                {
                    return Some(self.trunk_block);
                }
            }
            if let Some(tip) = branch.tip {
                if lobe_contains(tip, u, layer, v) {
                    return Some(self.leaves_block);
                }
            }
        }
        // Canopy
        for lobe in &self.lobes {
            if lobe_contains(*lobe, u, layer, v) {
                return Some(self.leaves_block);
            }
        }
        None
    }
}

// ---- shape primitives -----------------------------------------------------

fn trunk_radius_at(
    layer: f32,
    plant_height: f32,
    total_height: f32,
    base_r: f32,
    top_r: f32,
) -> f32 {
    let h = total_height.max(0.001);
    let t = ((layer - plant_height) / h).clamp(0.0, 1.0);
    // Slight bulge near the base for natural taper (square-root falloff).
    let bulge = (1.0 - t).sqrt();
    top_r + (base_r - top_r) * bulge
}

fn stamp_disc(
    cx: f32,
    cz: f32,
    layer: i32,
    r: f32,
    seed: u32,
    block: VoxelId,
    emit: &mut dyn FnMut(i32, i32, i32, VoxelId),
) {
    let r2 = r * r;
    let bound = r.ceil() as i32 + 1;
    let bx = cx.floor() as i32;
    let bz = cz.floor() as i32;
    for du in -bound..=bound {
        for dv in -bound..=bound {
            let u = bx + du;
            let v = bz + dv;
            let dx = u as f32 + 0.5 - cx;
            let dz = v as f32 + 0.5 - cz;
            let d2 = dx * dx + dz * dz;
            // Boundary jitter: ±0.45 v² perturbation breaks the perfect circle.
            let j = (hash01_u32(seed, ((du as u32) << 16) ^ (dv as u32 & 0xFFFF)) - 0.5) * 0.45;
            if d2 <= r2 + j {
                emit(u, layer, v, block);
            }
        }
    }
}

fn disc_contains(u: i32, v: i32, cx: f32, cz: f32, r: f32, seed: u32) -> bool {
    let dx = u as f32 + 0.5 - cx;
    let dz = v as f32 + 0.5 - cz;
    let d2 = dx * dx + dz * dz;
    let bx = cx.floor() as i32;
    let bz = cz.floor() as i32;
    let du = u - bx;
    let dv = v - bz;
    let j = (hash01_u32(seed, ((du as u32) << 16) ^ (dv as u32 & 0xFFFF)) - 0.5) * 0.45;
    d2 <= r * r + j
}

fn stamp_lobe(
    lobe: CanopyLobe,
    block: VoxelId,
    emit: &mut dyn FnMut(i32, i32, i32, VoxelId),
) {
    let r = lobe.radius;
    let r2 = r * r;
    let bound = r.ceil() as i32 + 1;
    let bx = lobe.center.x.floor() as i32;
    let by = lobe.center.y.floor() as i32;
    let bz = lobe.center.z.floor() as i32;
    for du in -bound..=bound {
        for dl in -bound..=bound {
            for dv in -bound..=bound {
                let u = bx + du;
                let l = by + dl;
                let v = bz + dv;
                let dx = u as f32 + 0.5 - lobe.center.x;
                let dy = l as f32 + 0.5 - lobe.center.y;
                let dz = v as f32 + 0.5 - lobe.center.z;
                let d2 = dx * dx + dy * dy + dz * dz;
                // Boundary jitter scales with r² so big lobes get bigger nibbles.
                let j = (hash01_u32(
                    lobe.jitter_seed,
                    ((du as u32) << 20) ^ ((dl as u32) << 10) ^ (dv as u32 & 0x3FF),
                ) - 0.5)
                    * r2
                    * 0.40;
                if d2 <= r2 + j {
                    emit(u, l, v, block);
                }
            }
        }
    }
}

fn lobe_contains(lobe: CanopyLobe, u: i32, layer: i32, v: i32) -> bool {
    let dx = u as f32 + 0.5 - lobe.center.x;
    let dy = layer as f32 + 0.5 - lobe.center.y;
    let dz = v as f32 + 0.5 - lobe.center.z;
    let d2 = dx * dx + dy * dy + dz * dz;
    let bx = lobe.center.x.floor() as i32;
    let by = lobe.center.y.floor() as i32;
    let bz = lobe.center.z.floor() as i32;
    let du = u - bx;
    let dl = layer - by;
    let dv = v - bz;
    let r2 = lobe.radius * lobe.radius;
    let j = (hash01_u32(
        lobe.jitter_seed,
        ((du as u32) << 20) ^ ((dl as u32) << 10) ^ (dv as u32 & 0x3FF),
    ) - 0.5)
        * r2
        * 0.40;
    d2 <= r2 + j
}

fn hash01(face: u8, u: u32, v: u32, salt: u32) -> f32 {
    hash4(face, u, v, salt) as f32 / u32::MAX as f32
}

fn hash01_u32(base: u32, salt: u32) -> f32 {
    let mut x = base ^ salt.wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
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

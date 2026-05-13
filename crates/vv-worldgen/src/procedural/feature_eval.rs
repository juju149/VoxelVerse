//! Slow-path tree query.
//!
//! When the bakery has not yet stamped a chunk's feature map (typical for
//! collision / raycast / single-voxel queries that bypass meshing), we still
//! need to know whether a voxel above the surface is part of a tree.  This
//! submodule iterates the same world-aligned placement cells the bakery
//! uses, applies the same biome / surface / slope / density filters, and
//! asks [`TreeShape`] whether the voxel belongs to that tree.
//!
//! Both code paths (bake and query) ultimately call `TreeShape::voxel_at`,
//! so they're guaranteed to agree on the tree's exact silhouette.

use super::{GeneratedVoxelContext, ProceduralPlanetTerrain};
use crate::placement::{candidate_for_cell, placement_cell_size, PlacementCandidate};
use vv_pack_compiler::CompiledVegetation;
use vv_voxel::VoxelId;

impl ProceduralPlanetTerrain {
    /// Return the tree voxel (trunk / branch / leaf) occupying this cell,
    /// if any tree planted within the neighborhood reaches it.  Iterates
    /// the placement cells around the query column.
    pub(super) fn tree_voxel_at(
        &self,
        ctx: &GeneratedVoxelContext,
        veg: &CompiledVegetation,
    ) -> Option<VoxelId> {
        let res = self.voxel_res as i32;
        let scale = self.voxel_scale();
        // The natural-shaped tree can reach beyond the trunk thickness via
        // canopy lobes and branches.  Use a conservative bound based on
        // the authored ranges + a margin for branch length / lean.
        let max_thickness = crate::features::scale_range(veg.trunk_thickness, scale)
            .1
            .max(1) as i32;
        let max_canopy = crate::features::scale_range(veg.canopy_radius, scale).1 as i32;
        let max_branch = crate::features::scale_range(veg.branch_length, scale).1 as i32;
        let scan_radius = max_thickness + max_canopy.max(max_branch) + 2;

        let cell = placement_cell_size(&veg.placement, scale).max(1);
        let u_lo = (ctx.u as i32 - scan_radius).max(0) as u32;
        let v_lo = (ctx.v as i32 - scan_radius).max(0) as u32;
        let u_hi = ((ctx.u as i32 + scan_radius + 1).min(res)) as u32;
        let v_hi = ((ctx.v as i32 + scan_radius + 1).min(res)) as u32;

        // Iterate every placement cell that could plant a tree reaching
        // `(ctx.u, ctx.v)`.  Cells are world-aligned (multiples of `cell`).
        let start_u = (u_lo / cell) * cell;
        let start_v = (v_lo / cell) * cell;
        let mut cu = start_u;
        while cu < u_hi {
            let mut cv = start_v;
            while cv < v_hi {
                if let Some(candidate) = candidate_for_cell(&veg.placement, ctx.face, cu, cv, cell)
                {
                    if let Some(block) = self.try_tree_at(ctx, veg, &candidate) {
                        return Some(block);
                    }
                }
                cv = cv.saturating_add(cell);
                if cv == 0 {
                    break;
                }
            }
            cu = cu.saturating_add(cell);
            if cu == 0 {
                break;
            }
        }
        None
    }

    fn try_tree_at(
        &self,
        ctx: &GeneratedVoxelContext,
        veg: &CompiledVegetation,
        candidate: &PlacementCandidate,
    ) -> Option<VoxelId> {
        let pu = candidate.pu;
        let pv = candidate.pv;
        let res_u = self.voxel_res;
        if pu >= res_u || pv >= res_u {
            return None;
        }

        let plant_biome = self
            .registry
            .biome(self.get_biome_id(ctx.face, pu, pv) as usize);
        if !veg.placement.allowed_in_biome(plant_biome) {
            return None;
        }
        if !veg
            .placement
            .surface_blocks
            .contains(&plant_biome.surface.top)
        {
            return None;
        }
        if veg.placement.slope_max > 0.0 || veg.placement.slope_min > 0.0 {
            let h0 = self.get_height(ctx.face, pu, pv) as i32;
            let h1 = self.get_height(ctx.face, (pu + 1).min(res_u - 1), pv) as i32;
            let h2 = self.get_height(ctx.face, pu, (pv + 1).min(res_u - 1)) as i32;
            let slope = (((h1 - h0).pow(2) + (h2 - h0).pow(2)) as f32).sqrt();
            let slope01 = (slope / (1.0 + slope)).clamp(0.0, 1.0);
            if veg.placement.slope_max > 0.0 && slope01 > veg.placement.slope_max {
                return None;
            }
            if slope01 < veg.placement.slope_min {
                return None;
            }
        }
        if let Some((lo, hi)) = veg.placement.altitude_range {
            let altitude = self.get_height(ctx.face, pu, pv) as i32 - self.surface_layer as i32;
            let alt_f = altitude as f32 * self.voxel_scale();
            if alt_f < lo || alt_f > hi {
                return None;
            }
        }
        if !self.placement_density_hit(&veg.placement, ctx.face, candidate) {
            return None;
        }
        self.evaluate_tree_at(ctx, veg, pu, pv)
    }

    /// Evaluate whether the current voxel belongs to a tree planted at
    /// column `(pu, pv)`.  Delegates to [`TreeShape`] so this slow path
    /// agrees exactly with what the chunk bakery would write into the
    /// feature map.
    fn evaluate_tree_at(
        &self,
        ctx: &GeneratedVoxelContext,
        veg: &CompiledVegetation,
        pu: u32,
        pv: u32,
    ) -> Option<VoxelId> {
        let plant_height = self.get_height(ctx.face, pu, pv);
        let scale = self.voxel_scale();
        let shape = crate::features::TreeShape::compute(veg, ctx.face, pu, pv, plant_height, scale);
        shape.voxel_at(ctx.u as i32, ctx.layer as i32, ctx.v as i32)
    }
}

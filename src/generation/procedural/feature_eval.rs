//! Slow-path tree query.
//!
//! When the bakery has not yet stamped a chunk's feature map (typical for
//! collision / raycast / single-voxel queries that bypass meshing), we still
//! need to know whether a voxel above the surface is part of a tree.  This
//! submodule scans the plant-column neighborhood, applies the same biome /
//! surface / slope / density filters as
//! [`crate::generation::features::FeatureBakery::is_planted`], and asks
//! [`TreeShape`] whether the voxel belongs to that tree.
//!
//! Both code paths (bake and query) ultimately call `TreeShape::voxel_at`,
//! so they're guaranteed to agree on the tree's exact silhouette.

use super::{GeneratedVoxelContext, ProceduralPlanetTerrain};
use crate::content::CompiledVegetation;
use crate::voxel::VoxelId;

impl ProceduralPlanetTerrain {
    /// Return the tree voxel (trunk / branch / leaf) occupying this cell,
    /// if any tree planted within the neighborhood reaches it.  Each
    /// candidate planting column `(pu, pv)` within
    /// `(trunk_thickness + canopy_radius)` of the current column is tested
    /// independently — first match wins, so density and tree size dictate
    /// whether two crowns can fight for the same voxel.
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
        let max_thickness = crate::generation::features::scale_range(veg.trunk_thickness, scale)
            .1
            .max(1) as i32;
        let max_canopy =
            crate::generation::features::scale_range(veg.canopy_radius, scale).1 as i32;
        let max_branch =
            crate::generation::features::scale_range(veg.branch_length, scale).1 as i32;
        let scan_radius = max_thickness + max_canopy.max(max_branch) + 2;

        for du in -scan_radius..=scan_radius {
            for dv in -scan_radius..=scan_radius {
                let pu_i = ctx.u as i32 + du;
                let pv_i = ctx.v as i32 + dv;
                if pu_i < 0 || pv_i < 0 || pu_i >= res || pv_i >= res {
                    continue;
                }
                let pu = pu_i as u32;
                let pv = pv_i as u32;

                let plant_biome =
                    self.registry.biome(self.get_biome_id(ctx.face, pu, pv) as usize);
                if !veg.placement.allowed_in_biome(plant_biome) {
                    continue;
                }
                let plant_top = plant_biome.surface.top;
                if !veg.placement.surface_blocks.contains(&plant_top) {
                    continue;
                }
                // Slope filter: mirrors `FeatureBakery::is_planted` in the
                // baked path.
                if veg.placement.slope_max > 0.0 {
                    let res_u = self.voxel_res;
                    let h0 = self.get_height(ctx.face, pu, pv) as i32;
                    let h1 = self.get_height(ctx.face, (pu + 1).min(res_u - 1), pv) as i32;
                    let h2 = self.get_height(ctx.face, pu, (pv + 1).min(res_u - 1)) as i32;
                    let slope = (((h1 - h0).pow(2) + (h2 - h0).pow(2)) as f32).sqrt();
                    if slope > veg.placement.slope_max {
                        continue;
                    }
                }
                let density = veg.placement.density * self.density_scale();
                if !self.feature_hit(veg.placement.field, ctx.face, pu, pv, density) {
                    continue;
                }

                if let Some(block) = self.evaluate_tree_at(ctx, veg, pu, pv) {
                    return Some(block);
                }
            }
        }
        None
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
        let shape = crate::generation::features::TreeShape::compute(
            veg, ctx.face, pu, pv, plant_height, scale,
        );
        shape.voxel_at(ctx.u as i32, ctx.layer as i32, ctx.v as i32)
    }
}

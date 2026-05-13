//! Single-voxel resolution: surface, terrain layers, ores, and caves.
//!
//! Splits the per-voxel pipeline into a handful of small methods on
//! [`ProceduralPlanetTerrain`].  The orchestration (decide which path to
//! take) lives in `voxel_at` upstream; this submodule owns the four leaf
//! questions:
//!
//! * Is this voxel inside a cave carver?
//! * What block does the biome's surface / terrain layer set choose at
//!   this depth?
//! * Should an ore replace the surface block here?
//! * If we're above the surface, is it a tree, a flower, or just air?

use super::{GeneratedVoxelContext, ProceduralPlanetTerrain};
use glam::Vec3;
use vv_voxel::PlanetProfile;
use vv_voxel::VoxelId;

impl ProceduralPlanetTerrain {
    pub(super) fn resolve_voxel(
        &self,
        ctx: &GeneratedVoxelContext,
        profile: PlanetProfile,
    ) -> VoxelId {
        if self.is_cave(ctx) {
            return VoxelId::AIR;
        }

        let scale = self.voxel_scale();
        let biome = self.registry.biome(ctx.surface.primary_biome);
        let surface_depth = crate::features::scale_range(biome.surface.depth, scale);
        let mut block = if ctx.depth_from_surface == 0 {
            biome.surface.top
        } else if ctx.depth_from_surface as u32 <= surface_depth.1 {
            biome.surface.under
        } else {
            self.layer_block(ctx, profile)
                .unwrap_or(biome.surface.under)
        };

        if let Some(ore) = self.ore_block(ctx, block) {
            block = ore;
        }

        block
    }

    pub(super) fn resolve_above_surface_voxel(&self, ctx: &GeneratedVoxelContext) -> VoxelId {
        // Walk every vegetation entry; the first tree that claims this voxel wins.
        for veg_idx in &self.planet().vegetation_sets {
            let veg = &self.registry.vegetation[*veg_idx];
            if let Some(block) = self.tree_voxel_at(ctx, veg) {
                return block;
            }
        }
        // Props are not in the voxel grid — query props_for_chunk() separately.
        VoxelId::AIR
    }

    fn layer_block(&self, ctx: &GeneratedVoxelContext, profile: PlanetProfile) -> Option<VoxelId> {
        let layers = &self.registry.terrain_layers[self.planet().terrain_layers];
        for layer in &layers.layers {
            let biome_ok = layer.all_biomes || layer.biomes.contains(&ctx.surface.primary_biome);
            if !biome_ok {
                continue;
            }
            let scale = self.voxel_scale();
            let depth_ok = layer.depth.is_some_and(|range| {
                let (min, max) = crate::features::scale_range(range, scale);
                (ctx.depth_from_surface as u32) >= min && (ctx.depth_from_surface as u32) <= max
            });
            let center_depth = profile.core_layers.saturating_sub(ctx.layer);
            let center_ok = layer.depth_from_center.is_some_and(|range| {
                let (min, max) = crate::features::scale_range(range, scale);
                center_depth >= min && center_depth <= max
            });
            if depth_ok || center_ok {
                return Some(layer.block);
            }
        }
        None
    }

    fn ore_block(&self, ctx: &GeneratedVoxelContext, current: VoxelId) -> Option<VoxelId> {
        let planet = self.planet();
        let biome = self.registry.biome(ctx.surface.primary_biome);
        let depth = ctx.depth_from_surface.max(0) as u32;
        let scale = self.voxel_scale();
        for ore_idx in &planet.ore_sets {
            let ore = &self.registry.ores[*ore_idx];
            let ore_depth = crate::features::scale_range(ore.depth, scale);
            if depth < ore_depth.0 || depth > ore_depth.1 || !ore.replace.contains(&current) {
                continue;
            }
            let tag_ok = ore.biome_tags.iter().any(|t| t == "*")
                || ore
                    .biome_tags
                    .iter()
                    .any(|t| biome.vegetation_tags.contains(t) || biome.fauna_tags.contains(t));
            if !tag_ok {
                continue;
            }
            let n = self.sample_field(ore.field, ctx.dir + Vec3::splat(depth as f32 * 0.013));
            let threshold = 1.0 - ore.density.clamp(0.0, 0.95);
            if n >= threshold {
                return Some(ore.block);
            }
        }
        None
    }

    fn is_cave(&self, ctx: &GeneratedVoxelContext) -> bool {
        let scale = self.voxel_scale();
        // 4-voxel cap-thickness scales with the grid so the surface skin
        // keeps the same physical thickness regardless of voxel resolution.
        let cap_thickness = crate::features::scale_count(4, scale).max(1) as i32;
        if ctx.depth_from_surface <= cap_thickness {
            return false;
        }
        for cave_idx in &self.planet().caves {
            let cave = &self.registry.caves[*cave_idx];
            for carver in &cave.carvers {
                let depth = ctx.depth_from_surface as u32;
                let cd = crate::features::scale_range(carver.depth, scale);
                if depth < cd.0 || depth > cd.1 {
                    continue;
                }
                let n = self.sample_field(
                    carver.field,
                    ctx.dir + Vec3::new(ctx.layer as f32 * 0.017, depth as f32 * 0.011, 0.0),
                );
                if n >= carver.threshold {
                    return true;
                }
            }
        }
        false
    }
}

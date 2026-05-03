use std::collections::HashMap;

use vv_registry::{BlockId as ContentBlockId, CompiledFloraFeature};
use vv_voxel::BlockId;

use crate::{
    climate::TerrainBiome,
    hash01, tags_match,
    tree::{TreeShape, TreeShapeConfig},
};

use super::{types::TerrainFlora, PlanetTerrain};

impl PlanetTerrain {
    pub(super) fn add_flora_feature_blocks(
        &self,
        face: u8,
        u: u32,
        v: u32,
        surface: u32,
        flora: &TerrainFlora,
        target: (u32, u32, u32, u32),
        blocks: &mut HashMap<BlockId, ContentBlockId>,
    ) {
        match flora.data.feature {
            CompiledFloraFeature::Plant {
                block,
                height_max_m,
                ..
            } => self.add_plant_blocks(face, u, v, surface, block, height_max_m, target, blocks),

            CompiledFloraFeature::Tree {
                log_block,
                leaf_block,
                trunk_height_min_m,
                trunk_height_max_m,
                canopy_radius_m,
                canopy_height_m,
                canopy_start_t,
                trunk_girth,
                crown_bias,
            } => {
                let shape = TreeShape::new(TreeShapeConfig {
                    face,
                    u,
                    v,
                    flora_index: flora.index,
                    world_seed: self.world_seed,
                    voxel_size_m: self.geometry.voxel_size_m,
                    trunk_height_min_m,
                    trunk_height_max_m,
                    canopy_radius_m,
                    canopy_height_m,
                    canopy_start_t,
                    trunk_girth,
                    crown_bias,
                });

                self.add_tree_blocks(
                    face, u, v, surface, log_block, leaf_block, &shape, target, blocks,
                );
            }

            CompiledFloraFeature::Cluster {
                block,
                radius_max_m,
                ..
            } => self.add_cluster_blocks(face, u, v, block, radius_max_m, target, blocks),
        }
    }

    pub(super) fn flora_block_at(
        &self,
        face: u8,
        u: u32,
        v: u32,
        layer: u32,
    ) -> Option<ContentBlockId> {
        if u >= self.geometry.resolution || v >= self.geometry.resolution {
            return None;
        }

        let target_column = self.column(face, u, v);

        if layer <= target_column.height as u32 {
            return None;
        }

        for flora in self.flora.iter() {
            match flora.data.feature {
                CompiledFloraFeature::Plant {
                    block,
                    height_max_m,
                    ..
                } => {
                    let biome = &self.biomes[target_column.biome_index as usize];

                    if !tags_match(
                        &flora.data.required_tags,
                        &flora.data.forbidden_tags,
                        &biome.data.provided_tags,
                    ) {
                        continue;
                    }

                    if !self.flora_origin_for_column(face, u, v, target_column, flora) {
                        continue;
                    }

                    let height_layers = self.geometry.meters_to_voxels_ceil(height_max_m);
                    let surface = target_column.height as u32;

                    if layer > surface && layer <= surface.saturating_add(height_layers) {
                        return Some(block);
                    }
                }

                CompiledFloraFeature::Tree {
                    log_block,
                    leaf_block,
                    trunk_height_min_m,
                    trunk_height_max_m,
                    canopy_radius_m,
                    canopy_height_m,
                    canopy_start_t,
                    trunk_girth,
                    crown_bias,
                } => {
                    let scan_radius = TreeShape::expanded_scan_radius_layers(
                        self.geometry.voxel_size_m,
                        canopy_radius_m,
                        trunk_height_max_m,
                    );

                    for origin_du in -scan_radius..=scan_radius {
                        for origin_dv in -scan_radius..=scan_radius {
                            let Some((origin_u, origin_v)) =
                                self.offset_column(u, v, origin_du, origin_dv)
                            else {
                                continue;
                            };

                            let origin_column = self.column(face, origin_u, origin_v);
                            let origin_biome = &self.biomes[origin_column.biome_index as usize];

                            if !tags_match(
                                &flora.data.required_tags,
                                &flora.data.forbidden_tags,
                                &origin_biome.data.provided_tags,
                            ) {
                                continue;
                            }

                            if !self.flora_origin_for_column(
                                face,
                                origin_u,
                                origin_v,
                                origin_column,
                                flora,
                            ) {
                                continue;
                            }

                            let origin_surface = origin_column.height as u32;

                            if layer <= origin_surface {
                                continue;
                            }

                            let rel_layer = layer - origin_surface;

                            let shape = TreeShape::new(TreeShapeConfig {
                                face,
                                u: origin_u,
                                v: origin_v,
                                flora_index: flora.index,
                                world_seed: self.world_seed,
                                voxel_size_m: self.geometry.voxel_size_m,
                                trunk_height_min_m,
                                trunk_height_max_m,
                                canopy_radius_m,
                                canopy_height_m,
                                canopy_start_t,
                                trunk_girth,
                                crown_bias,
                            });

                            if rel_layer > shape.max_relative_layer() {
                                continue;
                            }

                            let target_du = u as i32 - origin_u as i32;
                            let target_dv = v as i32 - origin_v as i32;

                            if shape.has_log_at(target_du, target_dv, rel_layer) {
                                return Some(log_block);
                            }

                            if shape.has_leaf_at(target_du, target_dv, rel_layer) {
                                return Some(leaf_block);
                            }
                        }
                    }
                }

                CompiledFloraFeature::Cluster {
                    block,
                    radius_max_m,
                    ..
                } => {
                    let surface = target_column.height as u32;

                    if layer != surface.saturating_add(1) {
                        continue;
                    }

                    let radius = self.geometry.meters_to_voxels_ceil(radius_max_m) as i32;

                    for origin_du in -radius..=radius {
                        for origin_dv in -radius..=radius {
                            let horizontal_m =
                                ((origin_du * origin_du + origin_dv * origin_dv) as f32).sqrt()
                                    * self.geometry.voxel_size_m;

                            if horizontal_m > radius_max_m {
                                continue;
                            }

                            let Some((origin_u, origin_v)) =
                                self.offset_column(u, v, origin_du, origin_dv)
                            else {
                                continue;
                            };

                            let origin_column = self.column(face, origin_u, origin_v);
                            let origin_biome = &self.biomes[origin_column.biome_index as usize];

                            if !tags_match(
                                &flora.data.required_tags,
                                &flora.data.forbidden_tags,
                                &origin_biome.data.provided_tags,
                            ) {
                                continue;
                            }

                            if self.flora_origin_for_column(
                                face,
                                origin_u,
                                origin_v,
                                origin_column,
                                flora,
                            ) {
                                return Some(block);
                            }
                        }
                    }
                }
            }
        }

        None
    }
    pub(super) fn flora_origin(
        &self,
        face: u8,
        u: u32,
        v: u32,
        biome: &TerrainBiome,
        flora: &TerrainFlora,
    ) -> bool {
        self.flora_origin_for_column(face, u, v, self.column(face, u, v), flora)
            && tags_match(
                &flora.data.required_tags,
                &flora.data.forbidden_tags,
                &biome.data.provided_tags,
            )
    }

    pub(super) fn flora_origin_for_column(
        &self,
        face: u8,
        u: u32,
        v: u32,
        column: super::TerrainColumn,
        flora: &TerrainFlora,
    ) -> bool {
        let placement = flora.data.placement;

        let surface = self.geometry.layer_radius_m(column.height as u32) - self.geometry.radius_m;

        if placement
            .altitude_max_m
            .is_some_and(|altitude_max| surface > altitude_max)
        {
            return false;
        }

        let cell_area_m2 = self.geometry.voxel_size_m * self.geometry.voxel_size_m;
        let origin_chance = (placement.density_base * cell_area_m2).clamp(0.0, 1.0);

        hash01(face, u, v, 0, flora.index) < origin_chance
    }

    fn add_plant_blocks(
        &self,
        face: u8,
        u: u32,
        v: u32,
        surface: u32,
        block: ContentBlockId,
        height_max_m: f32,
        target: (u32, u32, u32, u32),
        blocks: &mut HashMap<BlockId, ContentBlockId>,
    ) {
        let height_layers = self.geometry.meters_to_voxels_ceil(height_max_m);

        for layer in surface.saturating_add(1)..=surface.saturating_add(height_layers) {
            self.insert_feature_block(face, u, v, layer, block, target, blocks);
        }
    }

    fn add_tree_blocks(
        &self,
        face: u8,
        u: u32,
        v: u32,
        surface: u32,
        log_block: ContentBlockId,
        leaf_block: ContentBlockId,
        shape: &TreeShape,
        target: (u32, u32, u32, u32),
        blocks: &mut HashMap<BlockId, ContentBlockId>,
    ) {
        let radius = shape.scan_radius_layers();

        for du in -radius..=radius {
            for dv in -radius..=radius {
                let Some((ou, ov)) = self.offset_column(u, v, du, dv) else {
                    continue;
                };

                for rel_layer in 1..=shape.max_relative_layer() {
                    let layer = surface.saturating_add(rel_layer);

                    if shape.has_log_at(du, dv, rel_layer) {
                        self.insert_feature_block(face, ou, ov, layer, log_block, target, blocks);
                    }
                }
            }
        }

        for du in -radius..=radius {
            for dv in -radius..=radius {
                let Some((ou, ov)) = self.offset_column(u, v, du, dv) else {
                    continue;
                };

                for rel_layer in 1..=shape.max_relative_layer() {
                    let layer = surface.saturating_add(rel_layer);

                    if shape.has_leaf_at(du, dv, rel_layer) {
                        self.insert_feature_block(face, ou, ov, layer, leaf_block, target, blocks);
                    }
                }
            }
        }
    }

    fn add_cluster_blocks(
        &self,
        face: u8,
        u: u32,
        v: u32,
        block: ContentBlockId,
        radius_max_m: f32,
        target: (u32, u32, u32, u32),
        blocks: &mut HashMap<BlockId, ContentBlockId>,
    ) {
        let radius = self.geometry.meters_to_voxels_ceil(radius_max_m) as i32;

        for du in -radius..=radius {
            for dv in -radius..=radius {
                let Some((ou, ov)) = self.offset_column(u, v, du, dv) else {
                    continue;
                };

                let horizontal_m = ((du * du + dv * dv) as f32).sqrt() * self.geometry.voxel_size_m;

                if horizontal_m > radius_max_m {
                    continue;
                }

                let surface = self.column(face, ou, ov).height as u32;

                self.insert_feature_block(
                    face,
                    ou,
                    ov,
                    surface.saturating_add(1),
                    block,
                    target,
                    blocks,
                );
            }
        }
    }

    fn insert_feature_block(
        &self,
        face: u8,
        u: u32,
        v: u32,
        layer: u32,
        block: ContentBlockId,
        target: (u32, u32, u32, u32),
        blocks: &mut HashMap<BlockId, ContentBlockId>,
    ) {
        let (u_start, v_start, u_end, v_end) = target;

        if u >= u_start && u < u_end && v >= v_start && v < v_end {
            blocks.entry(BlockId { face, layer, u, v }).or_insert(block);
        }
    }

    fn offset_column(&self, u: u32, v: u32, du: i32, dv: i32) -> Option<(u32, u32)> {
        let ou = u as i32 + du;
        let ov = v as i32 + dv;

        if ou < 0 || ov < 0 {
            return None;
        }

        let ou = ou as u32;
        let ov = ov as u32;

        if ou >= self.geometry.resolution || ov >= self.geometry.resolution {
            return None;
        }

        Some((ou, ov))
    }
}

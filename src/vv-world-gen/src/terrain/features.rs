use std::collections::HashMap;

use vv_core::BlockId;
use vv_registry::BlockId as ContentBlockId;

use crate::tags_match;

use super::PlanetTerrain;

impl PlanetTerrain {
    pub fn generated_feature_block(
        &self,
        face: u8,
        u: u32,
        v: u32,
        layer: u32,
    ) -> Option<ContentBlockId> {
        if u >= self.geometry.resolution || v >= self.geometry.resolution {
            return None;
        }

        let column = self.column(face, u, v);

        if layer <= column.height as u32 {
            return None;
        }

        let blocks =
            self.feature_blocks_in_region(face, u, v, u.saturating_add(1), v.saturating_add(1));

        blocks.get(&BlockId { face, layer, u, v }).copied()
    }

    pub fn feature_candidate_layers(
        &self,
        face: u8,
        u: u32,
        v: u32,
    ) -> std::ops::RangeInclusive<u32> {
        let height = self.get_height(face, u, v);

        let max_layers = self
            .geometry
            .meters_to_voxels_ceil(self.max_feature_height_m.max(self.geometry.voxel_size_m));

        height.saturating_add(1)..=height.saturating_add(max_layers)
    }

    pub fn feature_blocks_in_region(
        &self,
        face: u8,
        u_start: u32,
        v_start: u32,
        u_end: u32,
        v_end: u32,
    ) -> HashMap<BlockId, ContentBlockId> {
        let u_end = u_end.min(self.geometry.resolution);
        let v_end = v_end.min(self.geometry.resolution);

        if u_start >= u_end || v_start >= v_end {
            return HashMap::new();
        }

        let margin = self
            .geometry
            .meters_to_voxels_ceil(self.max_feature_radius_m.max(0.0));

        let scan_u_start = u_start.saturating_sub(margin);
        let scan_v_start = v_start.saturating_sub(margin);
        let scan_u_end = u_end.saturating_add(margin).min(self.geometry.resolution);
        let scan_v_end = v_end.saturating_add(margin).min(self.geometry.resolution);

        let mut blocks = HashMap::new();

        for origin_u in scan_u_start..scan_u_end {
            for origin_v in scan_v_start..scan_v_end {
                let column = self.column(face, origin_u, origin_v);
                let biome = &self.biomes[column.biome_index as usize];

                for flora in self.flora.iter() {
                    if !tags_match(
                        &flora.data.required_tags,
                        &flora.data.forbidden_tags,
                        &biome.data.provided_tags,
                    ) {
                        continue;
                    }

                    if !self.flora_origin_for_column(face, origin_u, origin_v, column, flora) {
                        continue;
                    }

                    self.add_flora_feature_blocks(
                        face,
                        origin_u,
                        origin_v,
                        column.height as u32,
                        flora,
                        (u_start, v_start, u_end, v_end),
                        &mut blocks,
                    );
                }
            }
        }

        blocks
    }
}

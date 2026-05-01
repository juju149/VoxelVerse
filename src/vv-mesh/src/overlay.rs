use std::collections::HashMap;

use vv_core::{BlockId, ChunkKey, CHUNK_SIZE};
use vv_registry::BlockId as ContentBlockId;
use vv_world_runtime::PlanetData;

pub(crate) struct FeatureOverlay {
    blocks: HashMap<BlockId, ContentBlockId>,
    target: (u32, u32, u32, u32),
}

impl FeatureOverlay {
    pub(crate) fn new(data: &PlanetData, key: ChunkKey) -> Self {
        let res = data.resolution;

        let u_start = key.u_idx * CHUNK_SIZE;
        let v_start = key.v_idx * CHUNK_SIZE;
        let u_end = (u_start + CHUNK_SIZE).min(res);
        let v_end = (v_start + CHUNK_SIZE).min(res);

        let query_u_start = u_start.saturating_sub(1);
        let query_v_start = v_start.saturating_sub(1);
        let query_u_end = u_end.saturating_add(1).min(res);
        let query_v_end = v_end.saturating_add(1).min(res);

        let blocks = data.terrain.feature_blocks_in_region(
            key.face,
            query_u_start,
            query_v_start,
            query_u_end,
            query_v_end,
        );

        Self {
            blocks,
            target: (u_start, v_start, u_end, v_end),
        }
    }

    #[inline]
    pub(crate) fn get(&self, id: BlockId) -> Option<ContentBlockId> {
        self.blocks.get(&id).copied()
    }

    #[inline]
    pub(crate) fn contains(&self, id: BlockId) -> bool {
        self.blocks.contains_key(&id)
    }

    #[inline]
    pub(crate) fn is_in_target(&self, id: BlockId) -> bool {
        let (u_start, v_start, u_end, v_end) = self.target;
        id.u >= u_start && id.u < u_end && id.v >= v_start && id.v < v_end
    }

    #[inline]
    pub(crate) fn target(&self) -> (u32, u32, u32, u32) {
        self.target
    }

    pub(crate) fn target_feature_keys(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.blocks
            .keys()
            .copied()
            .filter(|id| self.is_in_target(*id))
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.blocks.len()
    }
}

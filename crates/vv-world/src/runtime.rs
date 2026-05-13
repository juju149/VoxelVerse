use vv_voxel::{VoxelChunk, VoxelChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};
use std::collections::HashMap;

#[derive(Clone)]
pub struct VoxelRuntime {
    chunks: HashMap<VoxelChunkKey, VoxelChunk>,
}

impl VoxelRuntime {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn get_override(&self, coord: VoxelCoord) -> Option<VoxelId> {
        self.chunks
            .get(&coord.chunk_key())
            .and_then(|chunk| chunk.get_override(coord.local_coord()))
    }

    pub fn set_override(&mut self, coord: VoxelCoord, voxel: Option<VoxelId>) {
        let key = coord.chunk_key();
        let local = coord.local_coord();

        if let Some(voxel) = voxel {
            self.chunks
                .entry(key)
                .or_default()
                .set_override(local, Some(voxel));
            return;
        }

        if let Some(chunk) = self.chunks.get_mut(&key) {
            chunk.set_override(local, None);
            if chunk.is_empty() {
                self.chunks.remove(&key);
            }
        }
    }

    pub fn iter_column_overrides(
        &self,
        face: u8,
        u_idx: u32,
        v_idx: u32,
    ) -> impl Iterator<Item = (VoxelCoord, VoxelId)> + '_ {
        self.chunks
            .iter()
            .filter_map(move |(key, chunk)| {
                (key.face == face && key.u_idx == u_idx && key.v_idx == v_idx)
                    .then_some((key, chunk))
            })
            .flat_map(|(key, chunk)| {
                chunk.iter_overrides(
                    key.face,
                    key.layer_idx * CHUNK_SIZE,
                    key.u_idx * CHUNK_SIZE,
                    key.v_idx * CHUNK_SIZE,
                )
            })
    }

    /// Number of chunks currently holding at least one override.
    #[cfg(test)]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(face: u8, layer: u32, u: u32, v: u32) -> VoxelCoord {
        VoxelCoord { face, layer, u, v }
    }

    const STONE: VoxelId = VoxelId::new(1);
    const DIRT: VoxelId = VoxelId::new(2);

    #[test]
    fn set_and_get_override() {
        let mut rt = VoxelRuntime::new();
        let c = coord(0, 5, 3, 7);
        assert_eq!(rt.get_override(c), None);
        rt.set_override(c, Some(STONE));
        assert_eq!(rt.get_override(c), Some(STONE));
    }

    #[test]
    fn override_air_clears_voxel() {
        let mut rt = VoxelRuntime::new();
        let c = coord(0, 5, 3, 7);
        rt.set_override(c, Some(STONE));
        rt.set_override(c, Some(VoxelId::AIR));
        // AIR is id=0, not UNSET — it should be stored and returned
        assert_eq!(rt.get_override(c), Some(VoxelId::AIR));
    }

    #[test]
    fn remove_override_returns_none() {
        let mut rt = VoxelRuntime::new();
        let c = coord(0, 5, 3, 7);
        rt.set_override(c, Some(STONE));
        rt.set_override(c, None); // explicit removal
        assert_eq!(rt.get_override(c), None);
    }

    #[test]
    fn empty_chunk_is_dropped_after_last_override_removed() {
        let mut rt = VoxelRuntime::new();
        let c = coord(0, 5, 3, 7);
        rt.set_override(c, Some(STONE));
        assert_eq!(rt.chunk_count(), 1);
        rt.set_override(c, None);
        // Chunk must be evicted once empty — no stale allocations
        assert_eq!(rt.chunk_count(), 0);
    }

    #[test]
    fn two_overrides_in_same_chunk_share_allocation() {
        let mut rt = VoxelRuntime::new();
        // Both coords land in the same VoxelChunkKey
        let c1 = coord(0, 0, 0, 0);
        let c2 = coord(0, 0, 1, 0);
        rt.set_override(c1, Some(STONE));
        rt.set_override(c2, Some(DIRT));
        assert_eq!(rt.chunk_count(), 1);
        assert_eq!(rt.get_override(c1), Some(STONE));
        assert_eq!(rt.get_override(c2), Some(DIRT));
    }

    #[test]
    fn removing_one_of_two_overrides_keeps_chunk() {
        let mut rt = VoxelRuntime::new();
        let c1 = coord(0, 0, 0, 0);
        let c2 = coord(0, 0, 1, 0);
        rt.set_override(c1, Some(STONE));
        rt.set_override(c2, Some(DIRT));
        rt.set_override(c1, None);
        assert_eq!(
            rt.chunk_count(),
            1,
            "Chunk with remaining override must be kept"
        );
        assert_eq!(rt.get_override(c1), None);
        assert_eq!(rt.get_override(c2), Some(DIRT));
    }

    #[test]
    fn iter_column_overrides_returns_all_in_column() {
        let mut rt = VoxelRuntime::new();
        // Put two overrides in face=0, u_idx=0, v_idx=0
        let c1 = coord(0, 0, 0, 0);
        let c2 = coord(0, 1, 0, 0);
        rt.set_override(c1, Some(STONE));
        rt.set_override(c2, Some(DIRT));

        let results: Vec<_> = rt.iter_column_overrides(0, 0, 0).collect();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn clear_removes_all_chunks() {
        let mut rt = VoxelRuntime::new();
        for l in 0..10 {
            rt.set_override(coord(0, l, 0, 0), Some(STONE));
        }
        assert!(rt.chunk_count() > 0);
        rt.clear();
        assert_eq!(rt.chunk_count(), 0);
    }
}


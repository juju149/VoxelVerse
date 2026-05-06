use crate::voxel::{VoxelChunk, VoxelChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};
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
                .or_insert_with(VoxelChunk::new)
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
}

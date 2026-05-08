use crate::{LocalVoxelCoord, VoxelChunkKey, CHUNK_SIZE};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct VoxelCoord {
    pub face: u8,
    pub layer: u32,
    pub u: u32,
    pub v: u32,
}

impl VoxelCoord {
    pub fn chunk_key(self) -> VoxelChunkKey {
        VoxelChunkKey {
            face: self.face,
            layer_idx: self.layer / CHUNK_SIZE,
            u_idx: self.u / CHUNK_SIZE,
            v_idx: self.v / CHUNK_SIZE,
        }
    }

    pub fn local_coord(self) -> LocalVoxelCoord {
        LocalVoxelCoord {
            layer: (self.layer % CHUNK_SIZE) as u8,
            u: (self.u % CHUNK_SIZE) as u8,
            v: (self.v % CHUNK_SIZE) as u8,
        }
    }
}

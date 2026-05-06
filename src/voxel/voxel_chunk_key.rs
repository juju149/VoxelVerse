#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct VoxelChunkKey {
    pub face: u8,
    pub layer_idx: u32,
    pub u_idx: u32,
    pub v_idx: u32,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ChunkKey {
    pub face: u8,
    pub u_idx: u32,
    pub v_idx: u32,
}

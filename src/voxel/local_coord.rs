#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct LocalVoxelCoord {
    pub layer: u8,
    pub u: u8,
    pub v: u8,
}

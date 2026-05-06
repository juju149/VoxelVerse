#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct BlockId {
    pub face: u8,
    pub layer: u32,
    pub u: u32,
    pub v: u32,
}

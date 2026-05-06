#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct LodKey {
    pub face: u8,
    pub x: u32,
    pub y: u32,
    pub size: u32,
}

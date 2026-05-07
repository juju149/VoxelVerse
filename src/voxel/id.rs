#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VoxelId(u16);

impl VoxelId {
    /// Air is always ID 0 — this is a structural engine constant, not content.
    pub const AIR: Self = Self::new(0);
    /// Sentinel value meaning "no override stored". Never appears in world queries.
    pub const UNSET: Self = Self::new(u16::MAX);

    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }

    pub const fn is_unset(self) -> bool {
        self.0 == Self::UNSET.0
    }
}

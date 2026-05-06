#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VoxelId(u16);

impl VoxelId {
    pub const AIR: Self = Self::new(0);
    pub const CORE: Self = Self::new(1);
    pub const DIRT: Self = Self::new(2);
    pub const GRASS: Self = Self::new(3);
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

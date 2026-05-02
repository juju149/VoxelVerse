#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SoftCubeFace {
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
}

impl SoftCubeFace {
    #[inline]
    pub(crate) fn id(self) -> u32 {
        match self {
            Self::Top => 0,
            Self::Bottom => 1,
            Self::Front => 2,
            Self::Back => 3,
            Self::Left => 4,
            Self::Right => 5,
        }
    }
}

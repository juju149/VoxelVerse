#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelOcclusion {
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
}

impl VoxelOcclusion {
    #[inline]
    pub(crate) fn all_occluded(self) -> bool {
        self.top && self.bottom && self.front && self.back && self.left && self.right
    }

    #[inline]
    pub(crate) fn visible_array(self) -> [bool; 6] {
        [
            !self.top,
            !self.bottom,
            !self.front,
            !self.back,
            !self.left,
            !self.right,
        ]
    }
}

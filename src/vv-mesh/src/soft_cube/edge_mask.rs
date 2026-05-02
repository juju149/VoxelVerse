use crate::shape::VoxelOcclusion;

/// Tells a soft cube face which local edges are truly exposed.
///
/// If an edge touches another visible block, we keep it full and straight.
/// If an edge is open to air, we allow rounding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SoftCubeEdgeMask {
    pub min_u: bool,
    pub max_u: bool,
    pub min_v: bool,
    pub max_v: bool,
}

impl SoftCubeEdgeMask {
    #[inline]
    pub(crate) fn top(occ: VoxelOcclusion) -> Self {
        Self {
            min_u: !occ.left,
            max_u: !occ.right,
            min_v: !occ.front,
            max_v: !occ.back,
        }
    }

    #[inline]
    pub(crate) fn bottom(occ: VoxelOcclusion) -> Self {
        Self {
            min_u: !occ.left,
            max_u: !occ.right,
            min_v: !occ.back,
            max_v: !occ.front,
        }
    }

    #[inline]
    pub(crate) fn front(occ: VoxelOcclusion) -> Self {
        Self {
            min_u: !occ.left,
            max_u: !occ.right,
            min_v: !occ.bottom,
            max_v: !occ.top,
        }
    }

    #[inline]
    pub(crate) fn back(occ: VoxelOcclusion) -> Self {
        Self {
            min_u: !occ.right,
            max_u: !occ.left,
            min_v: !occ.bottom,
            max_v: !occ.top,
        }
    }

    #[inline]
    pub(crate) fn left(occ: VoxelOcclusion) -> Self {
        Self {
            min_u: !occ.back,
            max_u: !occ.front,
            min_v: !occ.bottom,
            max_v: !occ.top,
        }
    }

    #[inline]
    pub(crate) fn right(occ: VoxelOcclusion) -> Self {
        Self {
            min_u: !occ.front,
            max_u: !occ.back,
            min_v: !occ.bottom,
            max_v: !occ.top,
        }
    }
}

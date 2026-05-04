#[derive(Debug, Clone, Copy)]
pub(crate) struct SoftCubeParams {
    pub radius: f32,
    pub pillow: f32,
    pub segments: u8,
}

impl SoftCubeParams {
    #[inline]
    pub(crate) fn polished_default() -> Self {
        Self {
            radius: 0.10,
            pillow: 0.0,
            segments: 3,
        }
    }

    #[inline]
    pub(crate) fn sanitized(self) -> Self {
        Self {
            radius: self.radius.clamp(0.0, 0.16),
            pillow: 0.0,
            segments: self.segments.clamp(1, 3),
        }
    }
}

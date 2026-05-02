#[derive(Debug, Clone, Copy)]
pub(crate) struct SoftCubeParams {
    /// Rounded corner radius in local voxel space.
    /// 0.0 = hard cube, 0.5 = sphere-like cube.
    pub radius: f32,

    /// Face inflation. Gives the "polished toy" pillow feeling.
    pub pillow: f32,

    /// Number of subdivisions per visible face.
    /// 4 = cheap, 6 = good, 8 = hero quality.
    pub segments: u8,
}

impl SoftCubeParams {
    #[inline]
    pub(crate) fn polished_default() -> Self {
        Self {
            radius: 0.16,
            pillow: 0.035,
            segments: 6,
        }
    }

    #[inline]
    pub(crate) fn sanitized(self) -> Self {
        Self {
            radius: self.radius.clamp(0.02, 0.24),
            pillow: self.pillow.clamp(0.0, 0.08),
            segments: self.segments.clamp(3, 10),
        }
    }
}

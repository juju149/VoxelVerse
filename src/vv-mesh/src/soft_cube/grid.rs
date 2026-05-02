#[inline]
pub(crate) fn grid_t(index: u8, segments: u8) -> f32 {
    if segments == 0 {
        return 0.0;
    }

    index as f32 / segments as f32
}

#[inline]
pub(crate) fn local_axis(t: f32) -> f32 {
    t * 2.0 - 1.0
}

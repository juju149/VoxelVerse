use glam::Vec3;

#[inline]
pub(super) fn safe_normalize(v: Vec3) -> Vec3 {
    let len_sq = v.length_squared();
    if len_sq <= 1e-8 {
        Vec3::Y
    } else {
        v / len_sq.sqrt()
    }
}

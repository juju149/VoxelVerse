pub(crate) fn meters_to_voxels(meters: f32, voxel_size_m: f32) -> u32 {
    (meters.max(0.0) / voxel_size_m.max(0.01)).ceil() as u32
}

pub(crate) fn radius_at_height(base: f32, top: f32, t: f32, taper: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let curved = t.powf(1.0 + taper.max(0.0));
    base + (top - base) * curved
}

pub(crate) fn ellipsoid_score(
    du: f32,
    dv: f32,
    dy: f32,
    radius_u: f32,
    radius_v: f32,
    radius_y: f32,
) -> f32 {
    let u = du / radius_u.max(0.001);
    let v = dv / radius_v.max(0.001);
    let y = dy / radius_y.max(0.001);
    u * u + v * v + y * y
}

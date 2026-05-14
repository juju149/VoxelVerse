fn vv_sun_elevation() -> f32 {
    return vv_sun_direction().y;
}

fn vv_day_factor() -> f32 {
    return vv_saturate(vv_sun_elevation() * 4.0 + 0.15);
}

fn vv_dawn_factor() -> f32 {
    let e = vv_sun_elevation();
    return vv_saturate(1.0 - abs(e) * 5.5) * vv_saturate(e * 6.0 + 0.8);
}

fn vv_night_factor() -> f32 {
    return vv_saturate((-vv_sun_elevation() - 0.08) * 6.0);
}

fn vv_sun_color() -> vec3<f32> {
    let horizon_sun = vec3<f32>(0.95, 0.42, 0.16);
    let noon_sun = vec3<f32>(0.86, 0.84, 0.76);
    let t = vv_saturate(vv_sun_elevation() * 2.6 + 0.32);
    return mix(horizon_sun, noon_sun, t) * min(vv_sun_intensity(), 1.0) * 0.88;
}

fn vv_sun_wrap_diffuse(normal: vec3<f32>, sun_dir: vec3<f32>) -> f32 {
    return max(dot(normal, sun_dir) * 0.92 + 0.02, 0.0);
}

fn vv_soft_backlight(normal: vec3<f32>, sun_dir: vec3<f32>) -> f32 {
    return pow(vv_saturate(dot(normal, -sun_dir) * 0.5 + 0.5), 3.0) * 0.035;
}

fn vv_moon_direction() -> vec3<f32> {
    let s = vv_sun_direction();
    return normalize(vec3<f32>(-s.x, -s.y + 0.08, -s.z));
}

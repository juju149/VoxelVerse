fn vv_forward_scatter(view_dir: vec3<f32>) -> f32 {
    let sun_align = max(dot(view_dir, normalize(global.sun_dir.xyz)), 0.0);
    return pow(sun_align, 7.0) * 0.55 * global.sky_zenith.w;
}


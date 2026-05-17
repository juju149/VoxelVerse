struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
    sky_horizon: vec4<f32>,
    sky_zenith: vec4<f32>,
    render_params: vec4<f32>,
    atmosphere_params: vec4<f32>,
    cloud_params: vec4<f32>,
    water_params: vec4<f32>,
    weather_params: vec4<f32>,
    celestial_params: vec4<f32>,
    celestial_moon: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;

fn vv_safe_normalize(v: vec3<f32>) -> vec3<f32> {
    let len_sq = max(dot(v, v), 0.00001);
    return v * inverseSqrt(len_sq);
}

fn vv_quality_bits() -> u32 {
    return u32(global.render_params.y);
}

fn vv_viewport_size() -> vec2<f32> {
    return max(global.render_params.zw, vec2<f32>(1.0));
}
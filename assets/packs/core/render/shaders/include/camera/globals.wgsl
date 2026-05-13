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
}

@group(0) @binding(0) var<uniform> global: Global;

fn vv_quality_bits() -> u32 {
    return u32(global.render_params.y);
}

fn vv_has_quality_flag(flag: u32) -> bool {
    return (vv_quality_bits() & flag) != 0u;
}

fn vv_shadow_pcf_level() -> u32 {
    return (vv_quality_bits() >> 1u) & 3u;
}

fn vv_time_seconds() -> f32 {
    return global.render_params.x;
}

fn vv_viewport_size() -> vec2<f32> {
    return max(global.render_params.zw, vec2<f32>(1.0));
}

fn vv_camera_position() -> vec3<f32> {
    return global.camera_pos.xyz;
}

fn vv_sun_direction() -> vec3<f32> {
    return normalize(global.sun_dir.xyz);
}

fn vv_sun_intensity() -> f32 {
    return global.sky_zenith.w;
}

fn vv_fog_density() -> f32 {
    return global.atmosphere_params.x;
}

fn vv_height_fog_strength() -> f32 {
    return global.atmosphere_params.y;
}

fn vv_volumetric_fog_strength() -> f32 {
    return global.atmosphere_params.z;
}

fn vv_exposure() -> f32 {
    return max(global.atmosphere_params.w, 0.01);
}

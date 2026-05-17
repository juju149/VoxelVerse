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
    // x = precipitation intensity [0,1],
    // y = wind direction x (horizontal),
    // z = wind direction z (horizontal),
    // w = precipitation kind (0 = none, 1 = rain, 2 = snow, 3 = sleet,
    //                         4 = sand,  5 = ash,  6 = toxic_mist)
    weather_params: vec4<f32>,
    // x = eclipse_factor (0 = clear, 1 = totality),
    // y = stars_visibility [0,1],
    // z = aurora_intensity [0,1],
    // w = sun angular radius (radians)
    celestial_params: vec4<f32>,
    // xyz = primary moon direction in world frame (unit, 0 if absent),
    // w   = primary moon angular radius (radians, 0 if absent)
    celestial_moon: vec4<f32>,
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

fn vv_precip_intensity() -> f32 {
    return clamp(global.weather_params.x, 0.0, 1.0);
}

fn vv_wind_dir_xz() -> vec2<f32> {
    return global.weather_params.yz;
}

fn vv_precip_kind() -> u32 {
    return u32(round(global.weather_params.w));
}

fn vv_eclipse_factor() -> f32 {
    return clamp(global.celestial_params.x, 0.0, 1.0);
}

fn vv_stars_visibility() -> f32 {
    return clamp(global.celestial_params.y, 0.0, 1.0);
}

fn vv_aurora_intensity() -> f32 {
    return clamp(global.celestial_params.z, 0.0, 1.0);
}

fn vv_sun_angular_radius() -> f32 {
    return max(global.celestial_params.w, 0.0);
}

fn vv_moon_dir() -> vec3<f32> {
    return global.celestial_moon.xyz;
}

fn vv_moon_angular_radius() -> f32 {
    return max(global.celestial_moon.w, 0.0);
}

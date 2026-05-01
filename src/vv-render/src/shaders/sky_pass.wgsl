@vertex
fn vs_sky(@builtin(vertex_index) vertex_index: u32) -> SkyOut {
    var out: SkyOut;

    var pos = vec2<f32>(-1.0, -3.0);

    switch vertex_index {
        case 0u: {
            pos = vec2<f32>(-1.0, -3.0);
        }
        case 1u: {
            pos = vec2<f32>(-1.0, 1.0);
        }
        default: {
            pos = vec2<f32>(3.0, 1.0);
        }
    }

    out.clip_pos = vec4<f32>(pos, 0.0, 1.0);
    out.ndc = pos;

    return out;
}

fn reconstruct_sky_ray(ndc: vec2<f32>) -> vec3<f32> {
    let clip = vec4<f32>(ndc, 1.0, 1.0);
    let world_far = global.inv_view_proj * clip;
    return safe_normalize(world_far.xyz / world_far.w - global.camera_pos.xyz);
}

fn star_field(ray_dir: vec3<f32>, night_amount: f32) -> vec3<f32> {
    let star_strength = global.atmosphere.sky_params.z;

    let p = floor(ray_dir * 950.0);
    let h = hash13(p);

    let star_mask = select(0.0, 1.0, h > 0.9965);
    let twinkle = hash13(p + vec3<f32>(13.1, 71.7, 9.2));

    let star = star_mask * mix(0.35, 1.0, twinkle) * star_strength * night_amount;

    return vec3<f32>(star);
}

fn moon_light(ray_dir: vec3<f32>, night_amount: f32) -> vec3<f32> {
    let moon_dir = safe_normalize(global.atmosphere.moon_direction.xyz);
    let moon_cos = saturate(dot(ray_dir, moon_dir));

    let moon_disk = pow(moon_cos, 640.0) * 2.5;
    let moon_halo = pow(moon_cos, 48.0) * 0.18;

    return global.atmosphere.moon_color.xyz * (moon_disk + moon_halo) * night_amount;
}

fn sky_gradient(ray_dir: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let raw_sky_t = saturate(dot(ray_dir, up));
    let sky_t = pow(raw_sky_t, 0.72);
    let horizon_color = global.atmosphere.horizon_glow_color.xyz;
    let mid_color = global.atmosphere.sky_color.xyz;
    let zenith_color = global.atmosphere.zenith_color.xyz;

    let mid_blend = smoothstep(0.06, 0.62, sky_t);
    let zenith_blend = smoothstep(0.42, 1.0, sky_t);

    var color = mix(horizon_color, mid_color, mid_blend);
    color = mix(color, zenith_color, zenith_blend);

    let horizon_band = 1.0 - smoothstep(0.00, 0.24, raw_sky_t);
    color = color + horizon_color * horizon_band * 0.16;
    color = mix(vec3<f32>(luminance(color)), color, 1.18);

    return max(color, vec3<f32>(0.0));
}

fn sun_glow(ray_dir: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let sun_dir = safe_normalize(global.atmosphere.sun_direction.xyz);

    let sun_cos = saturate(dot(ray_dir, sun_dir));
    let sun_height = dot(sun_dir, up);

    let sun_above_horizon = smoothstep(-0.08, 0.10, sun_height);

    let sun_bloom = pow(sun_cos, 10.0) * 0.12;
    let sun_halo = pow(sun_cos, 36.0) * 0.42;
    let sun_disk = pow(sun_cos, 720.0) * 1.85;

    return global.atmosphere.sun_color.xyz
        * (sun_bloom + sun_halo + sun_disk)
        * sun_above_horizon;
}

fn twilight_glow(ray_dir: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let sun_dir = safe_normalize(global.atmosphere.sun_direction.xyz);

    let sky_t = saturate(dot(ray_dir, up));
    let horizon_mask = pow(1.0 - sky_t, 2.8);

    let sun_height = dot(sun_dir, up);

    // Strongest glow when sun is close to horizon.
    let near_horizon = 1.0 - smoothstep(0.08, 0.38, abs(sun_height));

    // Glow mostly in the direction of the sun.
    let sun_facing = pow(saturate(dot(ray_dir, sun_dir) * 0.5 + 0.5), 4.0);

    return global.atmosphere.horizon_glow_color.xyz
        * horizon_mask
        * near_horizon
        * sun_facing
        * 1.35;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
    let ray_dir = reconstruct_sky_ray(in.ndc);

    // Round planet sky orientation.
    // Camera position from planet center gives local up.
    let up = safe_normalize(global.camera_pos.xyz);

    let night_amount = saturate(global.atmosphere.sky_params.w);

    var color = sky_gradient(ray_dir, up);

    color = color + twilight_glow(ray_dir, up);
    color = color + sun_glow(ray_dir, up);
    color = mix(color, color * 0.32, night_amount);

    color = color + star_field(ray_dir, night_amount);
    color = color + moon_light(ray_dir, night_amount);

    let encoded = encode_final_color(color);

    return vec4<f32>(encoded, 1.0);
}
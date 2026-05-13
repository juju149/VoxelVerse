#include "include/camera/globals.wgsl"
#include "include/atmosphere/sky.wgsl"

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn vv_disc_glow(uv: vec2<f32>, dir: vec3<f32>, radius: f32, glow: f32) -> vec3<f32> {
    let pixel_ndc = uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);
    let world = global.camera_pos.xyz + dir * 9000.0;
    let clip = global.view_proj * vec4<f32>(world, 1.0);

    if clip.w <= 0.001 || clip.z < 0.0 {
        return vec3<f32>(0.0);
    }

    let ndc = clip.xy / clip.w;
    let dist_sq = dot(pixel_ndc - ndc, pixel_ndc - ndc);

    let corona = exp(-dist_sq * glow);
    let disc = 1.0 - smoothstep(radius * 0.45, radius, dist_sq);

    return vec3<f32>(corona * 0.45 + disc);
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let sun_dir = normalize(global.sun_dir.xyz);
    let sun_elev = sun_dir.y;

    var sky = vv_sky_color(in.uv);

    let sun_col = mix(
        vec3<f32>(1.05, 0.42, 0.08),
        vec3<f32>(1.25, 1.15, 0.92),
        clamp(sun_elev * 2.8 + 0.25, 0.0, 1.0)
    );

    sky += vv_disc_glow(in.uv, sun_dir, 0.0026, 8.0) * sun_col * global.sky_zenith.w;

    let moon_dir = normalize(vec3<f32>(-sun_dir.x, -sun_dir.y + 0.08, -sun_dir.z));
    let moon_vis = clamp((-sun_elev - 0.05) * 5.0, 0.0, 1.0);

    sky += vv_disc_glow(in.uv, moon_dir, 0.0018, 18.0)
        * vec3<f32>(0.42, 0.46, 0.60)
        * moon_vis;

    return vec4<f32>(max(sky, vec3<f32>(0.0)), 1.0);
}

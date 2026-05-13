#include "include/camera/globals.wgsl"
#include "include/math/color_space.wgsl"

@group(1) @binding(0) var t_scene: texture_2d<f32>;
@group(1) @binding(1) var s_scene: sampler;

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn vv_sample_scene(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(t_scene, s_scene, uv).rgb;
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let qbits = vv_quality_bits();
    let resolution = max(global.render_params.zw, vec2<f32>(1.0));
    var color = vv_sample_scene(in.uv);
    if (qbits & 64u) != 0u {
        let px = 1.0 / resolution;
        let l = vv_sample_scene(in.uv - vec2<f32>(px.x, 0.0));
        let r = vv_sample_scene(in.uv + vec2<f32>(px.x, 0.0));
        let u = vv_sample_scene(in.uv - vec2<f32>(0.0, px.y));
        let d = vv_sample_scene(in.uv + vec2<f32>(0.0, px.y));
        color = color * 0.64 + (l + r + u + d) * 0.09;
    }
    if (qbits & 128u) != 0u {
        let bloom = max(color - vec3<f32>(1.05), vec3<f32>(0.0)) * 0.18;
        color += bloom;
    }
    let vignette = smoothstep(0.94, 0.18, distance(in.uv, vec2<f32>(0.5)));
    color *= mix(0.94, 1.0, vignette);
    color = vv_srgb_encode(vv_tonemap_agx(color, global.atmosphere_params.w));
    return vec4<f32>(color, 1.0);
}


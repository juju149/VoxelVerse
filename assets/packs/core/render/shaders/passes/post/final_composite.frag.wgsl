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

fn vv_grade_scene(color: vec3<f32>) -> vec3<f32> {
    let sun_elev = normalize(global.sun_dir.xyz).y;
    let day = clamp(sun_elev * 3.0 + 0.18, 0.0, 1.0);
    let night = clamp((-sun_elev - 0.06) * 4.0, 0.0, 1.0);

    let contrast = mix(0.92, 1.02, day) * mix(1.0, 0.92, night);
    var graded = (color - vec3<f32>(0.5)) * contrast + vec3<f32>(0.5);
    graded = max(graded, vec3<f32>(0.0));

    let saturation = mix(0.90, 1.04, day) * mix(1.0, 0.82, night);
    graded = vv_preserve_natural_saturation(graded, saturation);

    let night_lift = vec3<f32>(0.012, 0.014, 0.020) * night;
    return clamp(graded + night_lift, vec3<f32>(0.0), vec3<f32>(1.0));
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
    color = vv_grade_scene(vv_tonemap_agx(color, global.atmosphere_params.w * 0.78));
    // Output target is expected to be sRGB. Do not encode twice.
    return vec4<f32>(color, 1.0);
}


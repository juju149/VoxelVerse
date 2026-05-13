const VV_PI: f32 = 3.14159265359;
const VV_TAU: f32 = 6.28318530718;
const VV_EPSILON: f32 = 0.00001;

const VV_MATERIAL_INDEX_MASK: u32 = 0x0000FFFFu;
const VV_VERTEX_COLOR_ONLY: u32 = 0x0000FFFFu;

// Quality flags packed in global.render_params.y.
// bit 0      : triplanar material grain
// bits 1-2   : shadow PCF level, 0 low, 1 medium, 2 high
// bit 3      : material color only debug
// bit 4      : volumetric fog veil
// bit 5      : high quality clouds
// bit 6      : post-process soft anti-aliasing
// bit 7      : bloom
const VV_Q_TRIPLANAR: u32 = 1u;
const VV_Q_COLOR_ONLY: u32 = 8u;
const VV_Q_VOLUMETRIC_FOG: u32 = 16u;
const VV_Q_HIGH_CLOUDS: u32 = 32u;
const VV_Q_FXAA: u32 = 64u;
const VV_Q_BLOOM: u32 = 128u;

fn vv_saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn vv_smooth5(t: f32) -> f32 {
    let c = vv_saturate(t);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

fn vv_luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn vv_safe_normalize(v: vec3<f32>) -> vec3<f32> {
    let len_sq = max(dot(v, v), VV_EPSILON);
    return v * inverseSqrt(len_sq);
}

fn vv_remap01(v: f32, min_v: f32, max_v: f32) -> f32 {
    return vv_saturate((v - min_v) / max(max_v - min_v, VV_EPSILON));
}

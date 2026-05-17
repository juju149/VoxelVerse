const VV_PI: f32 = 3.14159265359;
const VV_TAU: f32 = 6.28318530718;
const VV_EPSILON: f32 = 0.00001;

const VV_MATERIAL_INDEX_MASK: u32 = 0x0000FFFFu;
const VV_VERTEX_COLOR_ONLY: u32 = 0x0000FFFFu;

const VV_Q_TRIPLANAR: u32 = 1u;
const VV_Q_PCF_SHIFT: u32 = 1u;
const VV_Q_PCF_MASK: u32 = 3u;
const VV_Q_COLOR_ONLY: u32 = 8u;
const VV_Q_VOLUMETRIC_FOG: u32 = 16u;
const VV_Q_VOLUMETRIC_CLOUDS: u32 = 32u;
const VV_Q_SOFT_AA: u32 = 64u;
const VV_Q_HIGHLIGHT_LIFT: u32 = 128u;

fn vv_saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn vv_smooth5(t: f32) -> f32 {
    let c = vv_saturate(t);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

fn vv_safe_normalize(v: vec3<f32>) -> vec3<f32> {
    let len_sq = max(dot(v, v), VV_EPSILON);
    return v * inverseSqrt(len_sq);
}

fn vv_luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}
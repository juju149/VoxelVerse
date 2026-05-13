const VV_PI: f32 = 3.14159265359;
const VV_TAU: f32 = 6.28318530718;
const VV_EPSILON: f32 = 0.00001;
const VV_MATERIAL_INDEX_MASK: u32 = 0x0000FFFFu;
const VV_VERTEX_COLOR_ONLY: u32 = 0x0000FFFFu;

fn vv_saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn vv_smooth5(t: f32) -> f32 {
    let c = vv_saturate(t);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}


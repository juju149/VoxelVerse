const VV_PI: f32 = 3.141592653589793;
const VV_EPSILON: f32 = 0.00001;

fn vv_saturate(value: f32) -> f32 {
    return clamp(value, 0.0, 1.0);
}


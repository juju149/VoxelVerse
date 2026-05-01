pub(crate) fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

pub(crate) fn centered(value: f32) -> f32 {
    value * 2.0 - 1.0
}

pub(crate) fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

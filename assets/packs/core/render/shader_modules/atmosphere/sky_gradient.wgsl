struct SkyFragmentIn {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: SkyFragmentIn) -> @location(0) vec4<f32> {
    let horizon = vec3<f32>(0.72, 0.84, 1.0);
    let zenith = vec3<f32>(0.18, 0.42, 0.86);
    return vec4<f32>(mix(horizon, zenith, clamp(in.uv.y, 0.0, 1.0)), 1.0);
}


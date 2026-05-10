struct WaterFragmentIn {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
}

@fragment
fn fs_main(in: WaterFragmentIn) -> @location(0) vec4<f32> {
    let wave = sin(in.world_pos.x * 0.08 + in.world_pos.z * 0.05) * 0.03;
    let color = vec3<f32>(0.18 + wave, 0.48 + wave, 0.76);
    return vec4<f32>(color, 0.72);
}


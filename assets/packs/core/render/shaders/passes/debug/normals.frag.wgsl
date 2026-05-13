struct DebugIn {
    @builtin(position) clip_pos: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
}

@fragment
fn fs_main(in: DebugIn) -> @location(0) vec4<f32> {
    return vec4<f32>(normalize(in.world_normal) * 0.5 + vec3<f32>(0.5), 1.0);
}


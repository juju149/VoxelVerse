struct DebugIn {
    @builtin(position) clip_pos: vec4<f32>,
}

@fragment
fn fs_main(in: DebugIn) -> @location(0) vec4<f32> {
    let d = clamp(in.clip_pos.z, 0.0, 1.0);
    return vec4<f32>(vec3<f32>(d), 1.0);
}


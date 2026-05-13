fn vv_apply_aerial_perspective(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    return vv_aerial_fog(color, world_pos);
}

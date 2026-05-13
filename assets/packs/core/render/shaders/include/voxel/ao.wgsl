fn vv_vertex_ao(vertex_color: vec3<f32>) -> f32 {
    let ao = dot(vertex_color, vec3<f32>(0.3333));
    return clamp(ao, 0.38, 1.16);
}

fn vv_soft_contact_occlusion(normal: vec3<f32>, world_pos: vec3<f32>) -> f32 {
    let up = vv_planet_up(world_pos);
    let downward = max(dot(normal, -up), 0.0);
    return mix(1.0, 0.88, downward);
}

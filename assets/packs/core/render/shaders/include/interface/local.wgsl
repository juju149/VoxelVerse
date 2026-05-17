struct Local {
    model: mat4x4<f32>,
    params: vec4<f32>,
}

@group(1) @binding(0) var<uniform> local: Local;

fn vv_local_opacity() -> f32 {
    return clamp(local.params.x, 0.0, 1.0);
}

fn vv_local_edge_radius() -> f32 {
    return max(local.params.y, 0.0);
}
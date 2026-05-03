@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    let world_pos = local.model * vec4<f32>(in.pos, 1.0);

    out.world_pos = world_pos.xyz;
    out.clip_pos = global.view_proj * world_pos;

    let normal_mat = mat3x3<f32>(
        local.model[0].xyz,
        local.model[1].xyz,
        local.model[2].xyz,
    );

    out.world_normal = safe_normalize(normal_mat * in.normal);
    out.color = in.color;
    out.uv = in.uv;
    out.block_id = in.block_id;
    out.block_visual_id = in.block_visual_id;
    out.face_id = in.face_id;
    out.voxel_pos = in.voxel_pos;
    out.variation_seed = in.variation_seed;
    out.ao = in.ao;

    let pos_light = global.light_view_proj * vec4<f32>(
        out.world_pos + out.world_normal * 0.05,
        1.0,
    );

    out.shadow_pos = vec3<f32>(
        pos_light.x * 0.5 + 0.5,
        -pos_light.y * 0.5 + 0.5,
        pos_light.z,
    );

    return out;
}

@fragment
fn fs_feedback(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, local.params.x * 0.82);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    if (local.params.x < 1.0 && dither_opacity(in.clip_pos, local.params.x)) {
        discard;
    }

    if (in.block_id < 0) {
        return vec4<f32>(in.color, local.params.x);
    }

    let visual = visual_for(in.block_visual_id);
    let alpha = visual.surface.z;

    if (alpha < 1.0 && dither_opacity(in.clip_pos, alpha)) {
        discard;
    }

    let N = safe_normalize(in.world_normal);
    let V = safe_normalize(global.camera_pos.xyz - in.world_pos);
    let up = local_up_at(in.world_pos);

    let albedo = procedural_block_albedo(
        in.world_pos,
        N,
        in.uv,
        in.block_id,
        in.block_visual_id,
        in.face_id,
        in.voxel_pos,
        in.variation_seed,
        up,
    );

    let encoded = pow(max(albedo, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
    return vec4<f32>(encoded, alpha);
}
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

    out.shadow_pos = vec3<f32>(0.0);

    return out;
}

fn vv_clean_mesh_ao(vertex_ao: f32, visual: BlockVisual) -> f32 {
    let receives_ao = select(0.0, 1.0, (visual.palette.w & 16u) != 0u);
    let authored_ao = clamp(visual.variation_b.w, 0.45, 1.0);
    return mix(1.0, saturate(vertex_ao), receives_ao * authored_ao * 0.62);
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
    let up = vec3<f32>(0.0, 1.0, 0.0);

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

    let roughness = clamp(visual.surface.x, 0.08, 1.0);
    let ao = vv_clean_mesh_ao(in.ao, visual);

    let lit = apply_planetary_lighting(
        albedo,
        visual.emission.xyz,
        in.world_pos,
        N,
        V,
        in.shadow_pos,
        ao,
        0.82,
        roughness,
        0.12,
        0.04,
    );

    let encoded = encode_final_color(lit);
    return vec4<f32>(encoded, alpha * local.params.x);
}
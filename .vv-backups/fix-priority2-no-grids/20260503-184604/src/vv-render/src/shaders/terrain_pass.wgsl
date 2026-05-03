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

    // Small normal offset avoids self-shadow acne on soft cubes / bevels.
    let pos_light = global.light_view_proj * vec4<f32>(
        out.world_pos + out.world_normal * 0.055,
        1.0,
    );

    out.shadow_pos = vec3<f32>(
        pos_light.x * 0.5 + 0.5,
        -pos_light.y * 0.5 + 0.5,
        pos_light.z,
    );

    return out;
}

fn vv_face_edge_contact(uv: vec2<f32>, edge_darkening: f32) -> f32 {
    let edge_distance = min(
        min(uv.x, 1.0 - uv.x),
        min(uv.y, 1.0 - uv.y),
    );

    let edge_mask = 1.0 - smoothstep(0.018, 0.155, edge_distance);
    let strength = mix(0.08, 0.26, saturate(edge_darkening));

    return 1.0 - edge_mask * strength;
}

fn vv_ao_for_visual(vertex_ao: f32, visual: BlockVisual, uv: vec2<f32>) -> f32 {
    let ao_influence = visual.variation_b.w;
    let edge_darkening = visual.variation_b.z;

    // Old meshes or overlays may carry 1.0. Keep them clean.
    let mesh_ao = mix(1.0, saturate(vertex_ao), mix(0.72, 1.0, saturate(ao_influence)));

    // Tiny contact darkening around block borders. This creates depth without
    // geometry queries or screen-space passes.
    let edge_contact = vv_face_edge_contact(uv, edge_darkening);

    return saturate(mesh_ao * edge_contact);
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

    let roughness = clamp(visual.surface.x, 0.04, 1.0);
    let metallic = saturate(visual.surface.y);
    let ao = vv_ao_for_visual(in.ao, visual, in.uv);

    let ao_strength = mix(0.70, 1.0, saturate(visual.variation_b.w));

    // Material response is intentionally simple for now:
    // - non-metallic voxels stay soft and readable
    // - lower roughness gets a small sun highlight
    let surface_response = mix(0.10, 0.28, 1.0 - roughness);
    let specular_strength = (1.0 - metallic) * mix(0.025, 0.115, 1.0 - roughness);

    var lit = apply_planetary_lighting(
        albedo,
        visual.emission.xyz,
        in.world_pos,
        N,
        V,
        in.shadow_pos,
        ao,
        ao_strength,
        roughness,
        surface_response,
        specular_strength,
    );

    lit = apply_planetary_fog(lit, in.world_pos);

    let encoded = encode_final_color(lit);
    return vec4<f32>(encoded, alpha * local.params.x);
}
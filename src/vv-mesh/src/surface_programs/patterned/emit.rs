use glam::Vec3;
use vv_registry::{pattern_has_geometry, RuntimePatternedProgram};

use crate::{
    shape::VoxelCorners,
    soft_cube::{
        local_to_world, sample_soft_cube_uv, SoftCubeEdgeMask, SoftCubeFace, SoftCubeParams,
        SoftCubeWorldFrame,
    },
    MeshGen, Vertex,
};

use super::{PatternedLayout, PatternedMeshConfig};

#[derive(Debug, Clone, Copy)]
struct PatternedSample {
    position: Vec3,
    normal: Vec3,
    uv: [f32; 2],
}

impl MeshGen {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_patterned_soft_cube_face(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        corners: VoxelCorners,
        face: SoftCubeFace,
        params: SoftCubeParams,
        edge_mask: SoftCubeEdgeMask,
        corner_colors: [[f32; 3]; 4],
        texture_id: i32,
        block_id: i32,
        block_visual_id: u32,
        voxel_pos: [i32; 3],
        variation_seed: u32,
        program: RuntimePatternedProgram,
    ) {
        let params = params.sanitized();
        let config = PatternedMeshConfig::from_runtime(program);

        // Shader-only patterns (e.g. rings) leave the mesh as a clean soft cube
        // and rely on the fragment shader for visual structure.
        if !pattern_has_geometry(config.kind) {
            Self::add_soft_cube_face(
                verts,
                inds,
                idx,
                corners,
                face,
                params,
                edge_mask,
                corner_colors,
                texture_id,
                block_id,
                block_visual_id,
                voxel_pos,
                variation_seed,
            );
            return;
        }

        let frame = SoftCubeWorldFrame::from_corners(corners);
        let base_depth = config.gap_depth.clamp(0.0, 0.20);

        emit_recessed_mortar(
            verts,
            inds,
            idx,
            corners,
            face,
            params,
            edge_mask,
            frame,
            base_depth,
            corner_colors,
            texture_id,
            block_id,
            block_visual_id,
            voxel_pos,
            variation_seed,
        );

        let layout = PatternedLayout::build(config, variation_seed ^ face.id());

        for cell in layout.cells {
            let width = cell.uv_max[0] - cell.uv_min[0];
            let height = cell.uv_max[1] - cell.uv_min[1];

            if width <= 0.012 || height <= 0.012 {
                continue;
            }

            let max_inset = width.min(height) * 0.42;
            let gap_inset = (config.gap_width * 0.5).clamp(0.002, max_inset);
            let bevel_inset = config.cell_bevel.clamp(0.0, max_inset * 0.65);
            let inset = gap_inset + bevel_inset * 0.35;

            let top_min = [
                (cell.uv_min[0] + inset).clamp(0.0, 1.0),
                (cell.uv_min[1] + inset).clamp(0.0, 1.0),
            ];
            let top_max = [
                (cell.uv_max[0] - inset).clamp(0.0, 1.0),
                (cell.uv_max[1] - inset).clamp(0.0, 1.0),
            ];

            if top_max[0] <= top_min[0] || top_max[1] <= top_min[1] {
                continue;
            }

            let panel_depth = (base_depth - cell.depth).clamp(0.0, base_depth);
            let panel_boost = 1.0 + cell.color_variation * 0.08;

            emit_panel_top(
                verts,
                inds,
                idx,
                corners,
                face,
                params,
                edge_mask,
                frame,
                top_min,
                top_max,
                panel_depth,
                corner_colors,
                panel_boost,
                texture_id,
                block_id,
                block_visual_id,
                voxel_pos,
                variation_seed ^ stable_cell_seed(cell.center()),
            );

            if bevel_inset > 0.001 && base_depth - panel_depth > 0.001 {
                let base_min = [
                    (top_min[0] - bevel_inset)
                        .max(cell.uv_min[0])
                        .clamp(0.0, 1.0),
                    (top_min[1] - bevel_inset)
                        .max(cell.uv_min[1])
                        .clamp(0.0, 1.0),
                ];
                let base_max = [
                    (top_max[0] + bevel_inset)
                        .min(cell.uv_max[0])
                        .clamp(0.0, 1.0),
                    (top_max[1] + bevel_inset)
                        .min(cell.uv_max[1])
                        .clamp(0.0, 1.0),
                ];

                emit_panel_bevels(
                    verts,
                    inds,
                    idx,
                    corners,
                    face,
                    params,
                    edge_mask,
                    base_min,
                    base_max,
                    top_min,
                    top_max,
                    base_depth,
                    panel_depth,
                    corner_colors,
                    panel_boost * 0.86,
                    texture_id,
                    block_id,
                    block_visual_id,
                    voxel_pos,
                    variation_seed ^ stable_cell_seed(cell.center()).rotate_left(7),
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_recessed_mortar(
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
    corners: VoxelCorners,
    face: SoftCubeFace,
    params: SoftCubeParams,
    edge_mask: SoftCubeEdgeMask,
    frame: SoftCubeWorldFrame,
    depth: f32,
    corner_colors: [[f32; 3]; 4],
    texture_id: i32,
    block_id: i32,
    block_visual_id: u32,
    voxel_pos: [i32; 3],
    variation_seed: u32,
) {
    let segments = params.segments.max(3);

    for y in 0..segments {
        for x in 0..segments {
            let u0 = x as f32 / segments as f32;
            let u1 = (x + 1) as f32 / segments as f32;
            let v0 = y as f32 / segments as f32;
            let v1 = (y + 1) as f32 / segments as f32;

            let s00 = sample_at_shader_uv(
                corners,
                face,
                params,
                edge_mask,
                frame,
                [u0, 1.0 - v0],
                depth,
            );
            let s10 = sample_at_shader_uv(
                corners,
                face,
                params,
                edge_mask,
                frame,
                [u1, 1.0 - v0],
                depth,
            );
            let s11 = sample_at_shader_uv(
                corners,
                face,
                params,
                edge_mask,
                frame,
                [u1, 1.0 - v1],
                depth,
            );
            let s01 = sample_at_shader_uv(
                corners,
                face,
                params,
                edge_mask,
                frame,
                [u0, 1.0 - v1],
                depth,
            );

            push_patterned_quad(
                verts,
                inds,
                idx,
                [s00.position, s10.position, s11.position, s01.position],
                [
                    scale_color(color_at(corner_colors, s00.uv), 0.72),
                    scale_color(color_at(corner_colors, s10.uv), 0.72),
                    scale_color(color_at(corner_colors, s11.uv), 0.72),
                    scale_color(color_at(corner_colors, s01.uv), 0.72),
                ],
                [s00.uv, s10.uv, s11.uv, s01.uv],
                Some([s00.normal, s10.normal, s11.normal, s01.normal]),
                texture_id,
                block_id,
                block_visual_id,
                face.id(),
                voxel_pos,
                variation_seed,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_panel_top(
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
    corners: VoxelCorners,
    face: SoftCubeFace,
    params: SoftCubeParams,
    edge_mask: SoftCubeEdgeMask,
    frame: SoftCubeWorldFrame,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    depth: f32,
    corner_colors: [[f32; 3]; 4],
    color_scale: f32,
    texture_id: i32,
    block_id: i32,
    block_visual_id: u32,
    voxel_pos: [i32; 3],
    variation_seed: u32,
) {
    let s00 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [uv_min[0], uv_max[1]],
        depth,
    );
    let s10 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [uv_max[0], uv_max[1]],
        depth,
    );
    let s11 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [uv_max[0], uv_min[1]],
        depth,
    );
    let s01 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [uv_min[0], uv_min[1]],
        depth,
    );

    push_patterned_quad(
        verts,
        inds,
        idx,
        [s00.position, s10.position, s11.position, s01.position],
        [
            scale_color(color_at(corner_colors, s00.uv), color_scale),
            scale_color(color_at(corner_colors, s10.uv), color_scale),
            scale_color(color_at(corner_colors, s11.uv), color_scale),
            scale_color(color_at(corner_colors, s01.uv), color_scale),
        ],
        [s00.uv, s10.uv, s11.uv, s01.uv],
        Some([s00.normal, s10.normal, s11.normal, s01.normal]),
        texture_id,
        block_id,
        block_visual_id,
        face.id(),
        voxel_pos,
        variation_seed,
    );
}

#[allow(clippy::too_many_arguments)]
fn emit_panel_bevels(
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
    corners: VoxelCorners,
    face: SoftCubeFace,
    params: SoftCubeParams,
    edge_mask: SoftCubeEdgeMask,
    base_min: [f32; 2],
    base_max: [f32; 2],
    top_min: [f32; 2],
    top_max: [f32; 2],
    base_depth: f32,
    top_depth: f32,
    corner_colors: [[f32; 3]; 4],
    color_scale: f32,
    texture_id: i32,
    block_id: i32,
    block_visual_id: u32,
    voxel_pos: [i32; 3],
    variation_seed: u32,
) {
    let frame = SoftCubeWorldFrame::from_corners(corners);

    let b00 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [base_min[0], base_max[1]],
        base_depth,
    );
    let b10 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [base_max[0], base_max[1]],
        base_depth,
    );
    let b11 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [base_max[0], base_min[1]],
        base_depth,
    );
    let b01 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [base_min[0], base_min[1]],
        base_depth,
    );

    let t00 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [top_min[0], top_max[1]],
        top_depth,
    );
    let t10 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [top_max[0], top_max[1]],
        top_depth,
    );
    let t11 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [top_max[0], top_min[1]],
        top_depth,
    );
    let t01 = sample_at_shader_uv(
        corners,
        face,
        params,
        edge_mask,
        frame,
        [top_min[0], top_min[1]],
        top_depth,
    );

    let quads = [
        [b00, b10, t10, t00],
        [t01, t11, b11, b01],
        [b01, b00, t00, t01],
        [t10, b10, b11, t11],
    ];

    for quad in quads {
        push_patterned_quad(
            verts,
            inds,
            idx,
            [
                quad[0].position,
                quad[1].position,
                quad[2].position,
                quad[3].position,
            ],
            [
                scale_color(color_at(corner_colors, quad[0].uv), color_scale),
                scale_color(color_at(corner_colors, quad[1].uv), color_scale),
                scale_color(color_at(corner_colors, quad[2].uv), color_scale),
                scale_color(color_at(corner_colors, quad[3].uv), color_scale),
            ],
            [quad[0].uv, quad[1].uv, quad[2].uv, quad[3].uv],
            None,
            texture_id,
            block_id,
            block_visual_id,
            face.id(),
            voxel_pos,
            variation_seed,
        );
    }
}

fn sample_at_shader_uv(
    corners: VoxelCorners,
    face: SoftCubeFace,
    params: SoftCubeParams,
    edge_mask: SoftCubeEdgeMask,
    frame: SoftCubeWorldFrame,
    uv: [f32; 2],
    depth: f32,
) -> PatternedSample {
    let p = sample_soft_cube_uv(face, uv[0], 1.0 - uv[1], params, edge_mask);
    let local = p.position - p.normal * depth.max(0.0);

    PatternedSample {
        position: local_to_world(corners, local),
        normal: frame.normal_to_world(p.normal),
        uv: p.uv,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_patterned_quad(
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
    pos: [Vec3; 4],
    colors: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
    normals: Option<[Vec3; 4]>,
    texture_id: i32,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: [i32; 3],
    variation_seed: u32,
) {
    let fallback_normal = (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize_or_zero();
    let normals = normals.unwrap_or([fallback_normal; 4]);
    let base = *idx;

    for (((p, c), uv), normal) in pos.iter().zip(colors.iter()).zip(uvs).zip(normals) {
        verts.push(Vertex {
            pos: p.to_array(),
            color: *c,
            normal: normal.to_array(),
            uv,
            texture_id,
            block_id,
            block_visual_id,
            face_id,
            voxel_pos,
            variation_seed,
            ao: c[0].max(c[1]).max(c[2]).clamp(0.0, 1.0),
        });
    }

    inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    *idx += 4;
}

fn color_at(corner_colors: [[f32; 3]; 4], uv: [f32; 2]) -> [f32; 3] {
    let x = uv[0].clamp(0.0, 1.0);
    let y = (1.0 - uv[1]).clamp(0.0, 1.0);

    let bottom = mix_color(corner_colors[0], corner_colors[1], x);
    let top = mix_color(corner_colors[3], corner_colors[2], x);

    mix_color(bottom, top, y)
}

fn scale_color(color: [f32; 3], scale: f32) -> [f32; 3] {
    [
        (color[0] * scale).clamp(0.0, 1.0),
        (color[1] * scale).clamp(0.0, 1.0),
        (color[2] * scale).clamp(0.0, 1.0),
    ]
}

fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn stable_cell_seed(center: [f32; 2]) -> u32 {
    let x = (center[0] * 4096.0) as u32;
    let y = (center[1] * 4096.0) as u32;

    x.wrapping_mul(0x9E37_79B9) ^ y.wrapping_mul(0x85EB_CA6B)
}

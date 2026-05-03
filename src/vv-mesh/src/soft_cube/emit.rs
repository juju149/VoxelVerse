use vv_registry::CompiledSurfaceProgram;

use crate::{shape::VoxelCorners, MeshGen, Vertex};

use super::{
    local_to_world, sample_soft_cube, SoftCubeEdgeMask, SoftCubeFace, SoftCubeParams,
    SoftCubeWorldFrame,
};

impl MeshGen {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_surface_programmed_soft_cube_face(
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
        surface_program: CompiledSurfaceProgram,
    ) {
        match surface_program {
            CompiledSurfaceProgram::Patterned(program) => Self::add_patterned_soft_cube_face(
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
                program,
            ),
            CompiledSurfaceProgram::Flat => Self::add_soft_cube_face(
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
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_soft_cube_face(
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
    ) {
        let params = params.sanitized();
        let frame = SoftCubeWorldFrame::from_corners(corners);
        let segments = params.segments;

        for y in 0..segments {
            for x in 0..segments {
                let p00 = sample_soft_cube(face, x, y, params, edge_mask);
                let p10 = sample_soft_cube(face, x + 1, y, params, edge_mask);
                let p11 = sample_soft_cube(face, x + 1, y + 1, params, edge_mask);
                let p01 = sample_soft_cube(face, x, y + 1, params, edge_mask);

                let t00 = uv_to_color_t(p00.uv);
                let t10 = uv_to_color_t(p10.uv);
                let t11 = uv_to_color_t(p11.uv);
                let t01 = uv_to_color_t(p01.uv);

                let colors = [
                    bilinear_color(corner_colors, t00[0], t00[1]),
                    bilinear_color(corner_colors, t10[0], t10[1]),
                    bilinear_color(corner_colors, t11[0], t11[1]),
                    bilinear_color(corner_colors, t01[0], t01[1]),
                ];

                Self::quad_with_uvs(
                    verts,
                    inds,
                    idx,
                    [
                        local_to_world(corners, p00.position),
                        local_to_world(corners, p10.position),
                        local_to_world(corners, p11.position),
                        local_to_world(corners, p01.position),
                    ],
                    colors,
                    [p00.uv, p10.uv, p11.uv, p01.uv],
                    texture_id,
                    block_id,
                    block_visual_id,
                    face.id(),
                    voxel_pos,
                    variation_seed,
                    false,
                    Some([
                        frame.normal_to_world(p00.normal),
                        frame.normal_to_world(p10.normal),
                        frame.normal_to_world(p11.normal),
                        frame.normal_to_world(p01.normal),
                    ]),
                );
            }
        }
    }
}

fn uv_to_color_t(uv: [f32; 2]) -> [f32; 2] {
    [uv[0].clamp(0.0, 1.0), (1.0 - uv[1]).clamp(0.0, 1.0)]
}

fn bilinear_color(c: [[f32; 3]; 4], x: f32, y: f32) -> [f32; 3] {
    let bottom = mix_color(c[0], c[1], x);
    let top = mix_color(c[3], c[2], x);
    mix_color(bottom, top, y)
}

fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

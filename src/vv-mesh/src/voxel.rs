use vv_core::BlockId;
use vv_registry::{BlockId as ContentBlockId, BlockRenderSource, CompiledBlockFace};
use vv_world_runtime::PlanetData;

use crate::{overlay::FeatureOverlay, shape::VoxelOcclusion, MeshGen, Vertex};

impl MeshGen {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_voxel(
        id: BlockId,
        block_id: ContentBlockId,
        data: &PlanetData,
        blocks: &impl BlockRenderSource,
        overlay: &FeatureOverlay,
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        let Some(render) = blocks.block_render(block_id) else {
            return;
        };

        let res = data.resolution;

        let check = |d_layer: i32, d_u: i32, d_v: i32| -> bool {
            let layer = id.layer as i32 + d_layer;
            let u = id.u as i32 + d_u;
            let v = id.v as i32 + d_v;

            if layer >= 0 && u >= 0 && u < res as i32 && v >= 0 && v < res as i32 {
                let neighbor = BlockId {
                    face: id.face,
                    layer: layer as u32,
                    u: u as u32,
                    v: v as u32,
                };

                let Some(neighbor_block) = Self::mesh_block_at(data, neighbor, overlay) else {
                    return false;
                };

                return blocks
                    .block_render(neighbor_block)
                    .map(|neighbor_render| neighbor_render.meshing.occludes)
                    .unwrap_or(false);
            }

            layer < 0
        };

        let occ = VoxelOcclusion {
            top: check(1, 0, 0),
            bottom: check(-1, 0, 0),
            right: check(0, 1, 0),
            left: check(0, -1, 0),
            back: check(0, 0, 1),
            front: check(0, 0, -1),
        };

        if occ.all_occluded() {
            return;
        }

        let mut light_val = 1.0f32;

        for offset in 1..=8 {
            if check(offset, 0, 0) {
                light_val = 0.15;
                break;
            }
        }

        let natural_h = data.terrain.get_height(id.face, id.u, id.v);
        if id.layer >= natural_h {
            light_val = 1.0;
        }

        let mut base_color = render.color;
        base_color[0] *= light_val;
        base_color[1] *= light_val;
        base_color[2] *= light_val;

        let visual_bevel = Self::visual_bevel(render);
        let corners = Self::voxel_corners(id, data);
        let face_normals = Self::voxel_face_normals(corners);
        let face_positions = Self::sculpted_face_positions(corners, occ, visual_bevel.edge_width);

        let block_raw_id = block_id.raw() as i32;
        let block_visual_id = render.visual_id.raw();
        let voxel_pos = [id.u as i32, id.v as i32, id.layer as i32];
        let planet_seed = data.terrain.world_seed();

        let apply = |ao: f32, texture_id: i32| -> [f32; 3] {
            if texture_id >= 0 {
                let light = light_val * ao;
                [light, light, light]
            } else {
                [base_color[0] * ao, base_color[1] * ao, base_color[2] * ao]
            }
        };

        let top_texture_id = Self::face_texture_id(render.texture_for_face(CompiledBlockFace::Top));
        let bottom_texture_id =
            Self::face_texture_id(render.texture_for_face(CompiledBlockFace::Bottom));
        let front_texture_id =
            Self::face_texture_id(render.texture_for_face(CompiledBlockFace::North));
        let back_texture_id =
            Self::face_texture_id(render.texture_for_face(CompiledBlockFace::South));
        let left_texture_id =
            Self::face_texture_id(render.texture_for_face(CompiledBlockFace::West));
        let right_texture_id =
            Self::face_texture_id(render.texture_for_face(CompiledBlockFace::East));

        let top_visible = !occ.top;
        let bottom_visible = !occ.bottom;
        let front_visible = !occ.front;
        let back_visible = !occ.back;
        let left_visible = !occ.left;
        let right_visible = !occ.right;

        let top_ao = if top_visible {
            let n = |u, v| check(1, u, v);
            [
                Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1)),
                Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1)),
                Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1)),
                Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1)),
            ]
        } else {
            [1.0; 4]
        };

        let top_colors = [
            apply(top_ao[0], top_texture_id),
            apply(top_ao[1], top_texture_id),
            apply(top_ao[2], top_texture_id),
            apply(top_ao[3], top_texture_id),
        ];

        // Per-corner AO for all faces. Directional base factor baked in alongside AO
        // so corners in recessed geometry darken naturally without a separate pass.
        let apply4 = |base: f32, ao: [f32; 4], tid: i32| -> [[f32; 3]; 4] {
            [
                apply(base * ao[0], tid),
                apply(base * ao[1], tid),
                apply(base * ao[2], tid),
                apply(base * ao[3], tid),
            ]
        };

        let bottom_ao = if bottom_visible {
            let n = |du: i32, dv: i32| check(-1, du, dv);
            [
                Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1)),
                Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1)),
                Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1)),
                Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1)),
            ]
        } else {
            [1.0; 4]
        };

        // front face = v-1 side; corners [i_bl, i_br, o_br, o_bl] → vary layer & u
        let front_ao = if front_visible {
            let n = |dl: i32, du: i32| check(dl, du, -1);
            [
                Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1)),
                Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1)),
                Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1)),
                Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1)),
            ]
        } else {
            [1.0; 4]
        };

        // back face = v+1 side; corners [o_tl, o_tr, i_tr, i_tl]
        let back_ao = if back_visible {
            let n = |dl: i32, du: i32| check(dl, du, 1);
            [
                Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1)),
                Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1)),
                Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1)),
                Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1)),
            ]
        } else {
            [1.0; 4]
        };

        // left face = u-1 side; corners [i_tl, i_bl, o_bl, o_tl] → vary layer & v
        let left_ao = if left_visible {
            let n = |dl: i32, dv: i32| check(dl, -1, dv);
            [
                Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1)),
                Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1)),
                Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1)),
                Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1)),
            ]
        } else {
            [1.0; 4]
        };

        // right face = u+1 side; corners [i_br, i_tr, o_tr, o_br]
        let right_ao = if right_visible {
            let n = |dl: i32, dv: i32| check(dl, 1, dv);
            [
                Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1)),
                Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1)),
                Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1)),
                Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1)),
            ]
        } else {
            [1.0; 4]
        };

        let bottom_colors = apply4(0.40, bottom_ao, bottom_texture_id);
        let front_colors = apply4(0.80, front_ao, front_texture_id);
        let back_colors = apply4(0.80, back_ao, back_texture_id);
        let left_colors = apply4(0.80, left_ao, left_texture_id);
        let right_colors = apply4(0.80, right_ao, right_texture_id);

        if top_visible {
            Self::quad(
                verts,
                inds,
                idx,
                face_positions.top,
                top_colors,
                top_texture_id,
                block_raw_id,
                block_visual_id,
                0,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 0, planet_seed),
                true,
                Some(Self::top_corner_normals(
                    face_normals,
                    occ,
                    visual_bevel.top_edge,
                )),
            );
        }

        if bottom_visible {
            Self::quad(
                verts,
                inds,
                idx,
                face_positions.bottom,
                bottom_colors,
                bottom_texture_id,
                block_raw_id,
                block_visual_id,
                1,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 1, planet_seed),
                true,
                Some(Self::bottom_corner_normals(
                    face_normals,
                    occ,
                    visual_bevel.top_edge,
                )),
            );
        }

        if front_visible {
            Self::quad(
                verts,
                inds,
                idx,
                face_positions.front,
                front_colors,
                front_texture_id,
                block_raw_id,
                block_visual_id,
                2,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 2, planet_seed),
                false,
                Some(Self::front_corner_normals(
                    face_normals,
                    occ,
                    visual_bevel.side_edge,
                )),
            );
        }

        if back_visible {
            Self::quad(
                verts,
                inds,
                idx,
                face_positions.back,
                back_colors,
                back_texture_id,
                block_raw_id,
                block_visual_id,
                3,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 3, planet_seed),
                false,
                Some(Self::back_corner_normals(
                    face_normals,
                    occ,
                    visual_bevel.side_edge,
                )),
            );
        }

        if left_visible {
            Self::quad(
                verts,
                inds,
                idx,
                face_positions.left,
                left_colors,
                left_texture_id,
                block_raw_id,
                block_visual_id,
                4,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 4, planet_seed),
                false,
                Some(Self::left_corner_normals(
                    face_normals,
                    occ,
                    visual_bevel.side_edge,
                )),
            );
        }

        if right_visible {
            Self::quad(
                verts,
                inds,
                idx,
                face_positions.right,
                right_colors,
                right_texture_id,
                block_raw_id,
                block_visual_id,
                5,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 5, planet_seed),
                false,
                Some(Self::right_corner_normals(
                    face_normals,
                    occ,
                    visual_bevel.side_edge,
                )),
            );
        }

        if visual_bevel.is_enabled() {
            Self::add_edge_bevels(
                verts,
                inds,
                idx,
                block_raw_id,
                occ.visible_array(),
                face_positions.as_array(),
                [
                    (top_texture_id, top_colors),
                    (bottom_texture_id, bottom_colors),
                    (front_texture_id, front_colors),
                    (back_texture_id, back_colors),
                    (left_texture_id, left_colors),
                    (right_texture_id, right_colors),
                ],
                face_normals.as_array(),
                block_visual_id,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 6, planet_seed),
            );
        }
    }
}

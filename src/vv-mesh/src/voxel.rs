use vv_core::BlockId;
use vv_planet::CoordSystem;
use vv_registry::{BlockId as ContentBlockId, BlockRenderSource, CompiledBlockFace};
use vv_world_runtime::PlanetData;

use crate::{overlay::FeatureOverlay, MeshGen, Vertex};

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

        let has_top = check(1, 0, 0);
        let has_btm = check(-1, 0, 0);
        let has_right = check(0, 1, 0);
        let has_left = check(0, -1, 0);
        let has_back = check(0, 0, 1);
        let has_front = check(0, 0, -1);

        if has_top && has_btm && has_left && has_right && has_front && has_back {
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

        let p = |u_off: u32, v_off: u32, l_off: u32| {
            CoordSystem::get_vertex_pos(
                id.face,
                id.u + u_off,
                id.v + v_off,
                id.layer + l_off,
                data.geometry,
            )
        };

        let i_bl = p(0, 0, 0);
        let i_br = p(1, 0, 0);
        let i_tl = p(0, 1, 0);
        let i_tr = p(1, 1, 0);

        let o_bl = p(0, 0, 1);
        let o_br = p(1, 0, 1);
        let o_tl = p(0, 1, 1);
        let o_tr = p(1, 1, 1);

        let visual_bevel = Self::visual_bevel(render);

        let top_radial = ((o_bl + o_br + o_tr + o_tl) * 0.25).normalize();
        let bottom_radial = ((i_tl + i_tr + i_br + i_bl) * 0.25).normalize();

        let front_normal = Self::face_normal([i_bl, i_br, o_br, o_bl]);
        let back_normal = Self::face_normal([o_tl, o_tr, i_tr, i_tl]);
        let left_normal = Self::face_normal([i_tl, i_bl, o_bl, o_tl]);
        let right_normal = Self::face_normal([i_br, i_tr, o_tr, o_br]);

        let top_normals = Self::rounded_corner_normals(
            top_radial,
            [
                [(!has_left, left_normal), (!has_front, front_normal)],
                [(!has_right, right_normal), (!has_front, front_normal)],
                [(!has_right, right_normal), (!has_back, back_normal)],
                [(!has_left, left_normal), (!has_back, back_normal)],
            ],
            visual_bevel.top_edge,
        );

        let bottom_normals = Self::rounded_corner_normals(
            bottom_radial,
            [
                [(!has_left, left_normal), (!has_back, back_normal)],
                [(!has_right, right_normal), (!has_back, back_normal)],
                [(!has_right, right_normal), (!has_front, front_normal)],
                [(!has_left, left_normal), (!has_front, front_normal)],
            ],
            visual_bevel.top_edge,
        );

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

        let top_visible = !has_top;
        let bottom_visible = !has_btm;
        let front_visible = !has_front;
        let back_visible = !has_back;
        let left_visible = !has_left;
        let right_visible = !has_right;

        let bevel_width = visual_bevel.edge_width;

        let top_pos = Self::inset_face(
            [o_bl, o_br, o_tr, o_tl],
            [front_visible, right_visible, back_visible, left_visible],
            bevel_width,
        );

        let bottom_pos = Self::inset_face(
            [i_tl, i_tr, i_br, i_bl],
            [back_visible, right_visible, front_visible, left_visible],
            bevel_width,
        );

        let front_pos = Self::inset_face(
            [i_bl, i_br, o_br, o_bl],
            [bottom_visible, right_visible, top_visible, left_visible],
            bevel_width,
        );

        let back_pos = Self::inset_face(
            [o_tl, o_tr, i_tr, i_tl],
            [top_visible, right_visible, bottom_visible, left_visible],
            bevel_width,
        );

        let left_pos = Self::inset_face(
            [i_tl, i_bl, o_bl, o_tl],
            [bottom_visible, front_visible, top_visible, back_visible],
            bevel_width,
        );

        let right_pos = Self::inset_face(
            [i_br, i_tr, o_tr, o_br],
            [bottom_visible, back_visible, top_visible, front_visible],
            bevel_width,
        );

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

        let bottom_c = apply(0.4, bottom_texture_id);
        let front_c = apply(0.8, front_texture_id);
        let back_c = apply(0.8, back_texture_id);
        let left_c = apply(0.8, left_texture_id);
        let right_c = apply(0.8, right_texture_id);

        if top_visible {
            Self::quad(
                verts,
                inds,
                idx,
                top_pos,
                top_colors,
                top_texture_id,
                block_raw_id,
                block_visual_id,
                0,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 0, planet_seed),
                true,
                Some(top_normals),
            );
        }

        if bottom_visible {
            Self::quad(
                verts,
                inds,
                idx,
                bottom_pos,
                [bottom_c; 4],
                bottom_texture_id,
                block_raw_id,
                block_visual_id,
                1,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 1, planet_seed),
                true,
                Some(bottom_normals),
            );
        }

        if front_visible {
            Self::quad(
                verts,
                inds,
                idx,
                front_pos,
                [front_c; 4],
                front_texture_id,
                block_raw_id,
                block_visual_id,
                2,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 2, planet_seed),
                false,
                Some(Self::rounded_corner_normals(
                    front_normal,
                    [
                        [(!has_btm, bottom_radial), (!has_left, left_normal)],
                        [(!has_btm, bottom_radial), (!has_right, right_normal)],
                        [(!has_top, top_radial), (!has_right, right_normal)],
                        [(!has_top, top_radial), (!has_left, left_normal)],
                    ],
                    visual_bevel.side_edge,
                )),
            );
        }

        if back_visible {
            Self::quad(
                verts,
                inds,
                idx,
                back_pos,
                [back_c; 4],
                back_texture_id,
                block_raw_id,
                block_visual_id,
                3,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 3, planet_seed),
                false,
                Some(Self::rounded_corner_normals(
                    back_normal,
                    [
                        [(!has_top, top_radial), (!has_left, left_normal)],
                        [(!has_top, top_radial), (!has_right, right_normal)],
                        [(!has_btm, bottom_radial), (!has_right, right_normal)],
                        [(!has_btm, bottom_radial), (!has_left, left_normal)],
                    ],
                    visual_bevel.side_edge,
                )),
            );
        }

        if left_visible {
            Self::quad(
                verts,
                inds,
                idx,
                left_pos,
                [left_c; 4],
                left_texture_id,
                block_raw_id,
                block_visual_id,
                4,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 4, planet_seed),
                false,
                Some(Self::rounded_corner_normals(
                    left_normal,
                    [
                        [(!has_btm, bottom_radial), (!has_back, back_normal)],
                        [(!has_btm, bottom_radial), (!has_front, front_normal)],
                        [(!has_top, top_radial), (!has_front, front_normal)],
                        [(!has_top, top_radial), (!has_back, back_normal)],
                    ],
                    visual_bevel.side_edge,
                )),
            );
        }

        if right_visible {
            Self::quad(
                verts,
                inds,
                idx,
                right_pos,
                [right_c; 4],
                right_texture_id,
                block_raw_id,
                block_visual_id,
                5,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 5, planet_seed),
                false,
                Some(Self::rounded_corner_normals(
                    right_normal,
                    [
                        [(!has_btm, bottom_radial), (!has_front, front_normal)],
                        [(!has_btm, bottom_radial), (!has_back, back_normal)],
                        [(!has_top, top_radial), (!has_back, back_normal)],
                        [(!has_top, top_radial), (!has_front, front_normal)],
                    ],
                    visual_bevel.side_edge,
                )),
            );
        }

        if bevel_width > 0.0 {
            Self::add_edge_bevels(
                verts,
                inds,
                idx,
                block_raw_id,
                [
                    top_visible,
                    bottom_visible,
                    front_visible,
                    back_visible,
                    left_visible,
                    right_visible,
                ],
                [
                    top_pos, bottom_pos, front_pos, back_pos, left_pos, right_pos,
                ],
                [
                    (top_texture_id, top_colors),
                    (bottom_texture_id, [bottom_c; 4]),
                    (front_texture_id, [front_c; 4]),
                    (back_texture_id, [back_c; 4]),
                    (left_texture_id, [left_c; 4]),
                    (right_texture_id, [right_c; 4]),
                ],
                [
                    top_radial,
                    bottom_radial,
                    front_normal,
                    back_normal,
                    left_normal,
                    right_normal,
                ],
                block_visual_id,
                voxel_pos,
                Self::stable_variation_seed(id, block_id, 6, planet_seed),
            );
        }
    }
}

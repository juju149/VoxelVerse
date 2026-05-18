use glam::Vec3;
use vv_math::{CoordSystem, SphericalGrid};
use vv_voxel::PlanetProfile;

use super::material_packing::{pack_material_edges, FaceEdgeMask};
use crate::ambient_occlusion;
use crate::cpu_mesh::CpuVertex;

pub(crate) fn grid_from_profile(p: PlanetProfile) -> SphericalGrid {
    SphericalGrid::new(p.resolution, p.inner_radius, p.layer_height)
}

/// One quad ready to be emitted as two triangles.
pub(crate) struct QuadFace {
    pub(crate) pos: [Vec3; 4],
    pub(crate) colors: [[f32; 3]; 4],
    pub(crate) force_radial: bool,
    pub(crate) packed_tex_index: u32,
    pub(crate) flip_u: bool,
    pub(crate) flip_v: bool,
}

/// Stateless helpers used by the mesher.
pub(crate) struct FaceEmitter;

impl FaceEmitter {
    pub(crate) fn quad(
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        face: QuadFace,
    ) {
        Self::quad_tiled(verts, inds, idx, face, [1.0, 1.0]);
    }

    pub(crate) fn quad_tiled(
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        face: QuadFace,
        uv_span: [f32; 2],
    ) {
        let u0 = if face.flip_u { uv_span[0] } else { 0.0_f32 };
        let u1 = if face.flip_u { 0.0_f32 } else { uv_span[0] };
        let v0 = if face.flip_v { uv_span[1] } else { 0.0_f32 };
        let v1 = if face.flip_v { 0.0_f32 } else { uv_span[1] };
        let uvs: [[f32; 2]; 4] = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];

        let geometric_normal = (face.pos[1] - face.pos[0])
            .cross(face.pos[2] - face.pos[0])
            .normalize();
        let normal = if face.force_radial {
            let center = (face.pos[0] + face.pos[1] + face.pos[2] + face.pos[3]) * 0.25;
            center.normalize().to_array()
        } else {
            geometric_normal.to_array()
        };

        for (i, uv) in uvs.iter().enumerate() {
            verts.push(CpuVertex {
                pos: face.pos[i].to_array(),
                uv: *uv,
                color: face.colors[i],
                normal,
                tex_index: face.packed_tex_index,
            });
        }

        let desired_normal = Vec3::from_array(normal);
        if geometric_normal.dot(desired_normal) < 0.0 {
            inds.extend_from_slice(&[*idx, *idx + 2, *idx + 1, *idx + 2, *idx, *idx + 3]);
        } else {
            inds.extend_from_slice(&[*idx, *idx + 1, *idx + 2, *idx + 2, *idx + 3, *idx]);
        }
        *idx += 4;
    }
}

/// Emit all visible cube faces for the voxel at `(face, layer, u, v)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_cube_voxel(
    planet_face: u8,
    layer: u32,
    u: u32,
    v: u32,
    profile: PlanetProfile,
    accessor: &super::face_culling::VoxelAccessor<'_>,
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
) {
    let voxel_id = accessor.voxel_id(layer, u, v);
    let materials = accessor.materials;
    let res = accessor.voxels.resolution;

    // Face-hiding check (returns true = face is hidden = don't emit).
    let check = |dl: i32, du: i32, dv: i32| -> bool {
        accessor.check_hides(voxel_id, layer as i32 + dl, u as i32 + du, v as i32 + dv)
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

    // Skylight
    let natural_h = accessor.surface_height(u, v);
    let at_or_above_surface = layer >= natural_h;
    let light_val = super::lighting::skylight(|i| check(i, 0, 0), at_or_above_surface, 8);

    let mesh_class = materials.mesh_class(voxel_id);
    let visual = materials.visual(voxel_id);
    let mut fallback_color = materials.color(voxel_id);
    fallback_color[0] *= light_val;
    fallback_color[1] *= light_val;
    fallback_color[2] *= light_val;

    let grid = grid_from_profile(profile);
    let p = |u_off: u32, v_off: u32, l_off: u32| {
        CoordSystem::get_vertex_pos(planet_face, u + u_off, v + v_off, layer + l_off, grid)
    };
    let i_bl = p(0, 0, 0);
    let i_br = p(1, 0, 0);
    let i_tl = p(0, 1, 0);
    let i_tr = p(1, 1, 0);
    let o_bl = p(0, 0, 1);
    let o_br = p(1, 0, 1);
    let o_tl = p(0, 1, 1);
    let o_tr = p(1, 1, 1);

    let face_color = |mat_layer: u32, ao: f32| -> [f32; 3] {
        use super::material_packing::VoxelMeshClass;
        let c = if mat_layer == 0 {
            if mesh_class == VoxelMeshClass::Water {
                [
                    visual.tint[0] * light_val,
                    visual.tint[1] * light_val,
                    visual.tint[2] * light_val,
                ]
            } else {
                fallback_color
            }
        } else {
            [
                visual.tint[0] * light_val,
                visual.tint[1] * light_val,
                visual.tint[2] * light_val,
            ]
        };
        [c[0] * ao, c[1] * ao, c[2] * ao]
    };

    if !has_top {
        let layer_idx = visual.layers.top;
        let edges = FaceEdgeMask {
            min_u: !has_left,
            max_u: !has_right,
            min_v: !has_front,
            max_v: !has_back,
        };
        let n = |du: i32, dv: i32| check(1, du, dv);
        let ao_bl = ambient_occlusion::calculate(n(-1, 0), n(0, -1), n(-1, -1));
        let ao_br = ambient_occlusion::calculate(n(1, 0), n(0, -1), n(1, -1));
        let ao_tr = ambient_occlusion::calculate(n(1, 0), n(0, 1), n(1, 1));
        let ao_tl = ambient_occlusion::calculate(n(-1, 0), n(0, 1), n(-1, 1));
        FaceEmitter::quad(
            verts,
            inds,
            idx,
            QuadFace {
                pos: [o_bl, o_br, o_tr, o_tl],
                colors: [
                    face_color(layer_idx, ao_bl),
                    face_color(layer_idx, ao_br),
                    face_color(layer_idx, ao_tr),
                    face_color(layer_idx, ao_tl),
                ],
                force_radial: true,
                packed_tex_index: pack_material_edges(layer_idx, edges),
                flip_u: false,
                flip_v: false,
            },
        );
    }

    if !has_btm {
        let layer_idx = visual.layers.bottom;
        let edges = FaceEdgeMask {
            min_u: !has_left,
            max_u: !has_right,
            min_v: !has_back,
            max_v: !has_front,
        };
        let c = face_color(layer_idx, 0.4);
        FaceEmitter::quad(
            verts,
            inds,
            idx,
            QuadFace {
                pos: [i_tl, i_tr, i_br, i_bl],
                colors: [c, c, c, c],
                force_radial: false,
                packed_tex_index: pack_material_edges(layer_idx, edges),
                flip_u: false,
                flip_v: true,
            },
        );
    }

    if !has_front {
        let layer_idx = visual.layers.front;
        let edges = FaceEdgeMask {
            min_u: !has_left,
            max_u: !has_right,
            min_v: !has_top,
            max_v: !has_btm,
        };
        let c = face_color(layer_idx, 0.8);
        FaceEmitter::quad(
            verts,
            inds,
            idx,
            QuadFace {
                pos: [i_bl, i_br, o_br, o_bl],
                colors: [c, c, c, c],
                force_radial: false,
                packed_tex_index: pack_material_edges(layer_idx, edges),
                flip_u: false,
                flip_v: true,
            },
        );
    }

    if !has_back {
        let layer_idx = visual.layers.back;
        let edges = FaceEdgeMask {
            min_u: !has_left,
            max_u: !has_right,
            min_v: !has_top,
            max_v: !has_btm,
        };
        let c = face_color(layer_idx, 0.8);
        FaceEmitter::quad(
            verts,
            inds,
            idx,
            QuadFace {
                pos: [i_tr, i_tl, o_tl, o_tr],
                colors: [c, c, c, c],
                force_radial: false,
                packed_tex_index: pack_material_edges(layer_idx, edges),
                flip_u: false,
                flip_v: true,
            },
        );
    }

    if !has_left {
        let layer_idx = visual.layers.left;
        let edges = FaceEdgeMask {
            min_u: !has_back,
            max_u: !has_front,
            min_v: !has_top,
            max_v: !has_btm,
        };
        let c = face_color(layer_idx, 0.8);
        FaceEmitter::quad(
            verts,
            inds,
            idx,
            QuadFace {
                pos: [i_tl, i_bl, o_bl, o_tl],
                colors: [c, c, c, c],
                force_radial: false,
                packed_tex_index: pack_material_edges(layer_idx, edges),
                flip_u: false,
                flip_v: true,
            },
        );
    }

    if !has_right {
        let layer_idx = visual.layers.right;
        let edges = FaceEdgeMask {
            min_u: !has_front,
            max_u: !has_back,
            min_v: !has_top,
            max_v: !has_btm,
        };
        let c = face_color(layer_idx, 0.8);
        FaceEmitter::quad(
            verts,
            inds,
            idx,
            QuadFace {
                pos: [i_br, i_tr, o_tr, o_br],
                colors: [c, c, c, c],
                force_radial: false,
                packed_tex_index: pack_material_edges(layer_idx, edges),
                flip_u: false,
                flip_v: true,
            },
        );
    }
    let _ = res; // suppress unused warning
}

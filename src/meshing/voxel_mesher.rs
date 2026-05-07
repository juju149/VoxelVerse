use super::{ambient_occlusion, CpuMesh, CpuVertex, MeshGen};
use crate::generation::CoordSystem;
use crate::voxel::{SurfaceChunkKey, VoxelCoord, CHUNK_SIZE};
use crate::world::PlanetData;
use glam::Vec3;
use std::collections::HashSet;

impl MeshGen {
    fn add_modified_candidates(id: VoxelCoord, candidates: &mut HashSet<VoxelCoord>, res: u32) {
        candidates.insert(id);
        if id.layer + 1 < res {
            candidates.insert(VoxelCoord {
                layer: id.layer + 1,
                ..id
            });
        }
        if id.layer > 0 {
            candidates.insert(VoxelCoord {
                layer: id.layer - 1,
                ..id
            });
        }
        if id.u > 0 {
            candidates.insert(VoxelCoord { u: id.u - 1, ..id });
        }
        if id.u < res - 1 {
            candidates.insert(VoxelCoord { u: id.u + 1, ..id });
        }
        if id.v > 0 {
            candidates.insert(VoxelCoord { v: id.v - 1, ..id });
        }
        if id.v < res - 1 {
            candidates.insert(VoxelCoord { v: id.v + 1, ..id });
        }
    }

    pub fn build_chunk(key: SurfaceChunkKey, data: &PlanetData) -> CpuMesh {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;
        let res = data.resolution;
        let mut candidates = HashSet::new();

        let u_start = key.u_idx * CHUNK_SIZE;
        let v_start = key.v_idx * CHUNK_SIZE;
        // Ensure we don't iterate past resolution even if key exists
        let u_end = (u_start + CHUNK_SIZE).min(res);
        let v_end = (v_start + CHUNK_SIZE).min(res);

        // natural Surface (with slope filling)
        // need to check neighbors to see how far down the cliff goes.
        // if a neighbor is lower than us, we must generate the blocks between our height and theirs.

        // safely get height from the terrain map
        let get_h = |f, u, v| -> u32 {
            if u >= res || v >= res {
                return 0;
            }
            // using 0 here means "very deep", so we might generate extra mesh at face edges, which is safer than holes.
            data.terrain.get_height(f, u, v)
        };

        for u in u_start..u_end {
            for v in v_start..v_end {
                let h = get_h(key.face, u, v);
                if h == 0 {
                    continue;
                }

                // always add the top surface block
                candidates.insert(VoxelCoord {
                    face: key.face,
                    layer: h,
                    u,
                    v,
                });

                // check immediate neighbors to find the lowest exposed point
                let mut min_h = h;

                if u > 0 {
                    min_h = min_h.min(get_h(key.face, u - 1, v));
                }
                if u < res - 1 {
                    min_h = min_h.min(get_h(key.face, u + 1, v));
                }
                if v > 0 {
                    min_h = min_h.min(get_h(key.face, u, v - 1));
                }
                if v < res - 1 {
                    min_h = min_h.min(get_h(key.face, u, v + 1));
                }

                if min_h < h {
                    let bottom = min_h.max(h.saturating_sub(20));

                    for l in (bottom + 1)..h {
                        candidates.insert(VoxelCoord {
                            face: key.face,
                            layer: l,
                            u,
                            v,
                        });
                    }
                }
            }
        }

        // runtime voxel overrides in this surface tile and its direct neighbors.
        let neighbor_keys = [
            key,
            SurfaceChunkKey {
                u_idx: key.u_idx.wrapping_sub(1),
                ..key
            },
            SurfaceChunkKey {
                u_idx: key.u_idx + 1,
                ..key
            },
            SurfaceChunkKey {
                v_idx: key.v_idx.wrapping_sub(1),
                ..key
            },
            SurfaceChunkKey {
                v_idx: key.v_idx + 1,
                ..key
            },
        ];

        for n_key in neighbor_keys {
            for (id, _) in data.modified_voxels_in_chunk_column(n_key) {
                Self::add_modified_candidates(id, &mut candidates, res);
            }
        }

        // generate Mesh
        for id in candidates {
            if id.u >= u_start && id.u < u_end && id.v >= v_start && id.v < v_end && data.exists(id)
            {
                Self::add_voxel(id, data, &mut verts, &mut inds, &mut idx);
            }
        }
        CpuMesh::new(verts, inds)
    }

    fn add_voxel(
        id: VoxelCoord,
        data: &PlanetData,
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        let res = data.resolution;

        // neighbor existence check
        let check = |d_face: u8, d_layer: i32, d_u: i32, d_v: i32| -> bool {
            let l = id.layer as i32 + d_layer;
            let u = id.u as i32 + d_u;
            let v = id.v as i32 + d_v;
            if l >= 0 && u >= 0 && u < res as i32 && v >= 0 && v < res as i32 {
                return data.exists(VoxelCoord {
                    face: d_face,
                    layer: l as u32,
                    u: u as u32,
                    v: v as u32,
                });
            }
            l < 0 // Core is solid
        };

        // --- FACE CHECKS ---
        let has_top = check(id.face, 1, 0, 0);
        let has_btm = check(id.face, -1, 0, 0);
        let has_right = check(id.face, 0, 1, 0);
        let has_left = check(id.face, 0, -1, 0);
        let has_back = check(id.face, 0, 0, 1);
        let has_front = check(id.face, 0, 0, -1);

        if has_top && has_btm && has_left && has_right && has_front && has_back {
            return;
        }

        // --- LIGHTING CALCULATION ( this is simple, i will change this later)---
        // we cast a short ray (8 blocks)
        // if we hit nothing, we assume we are near the surface
        // if we hit blocks, we darken

        let mut light_val: f32 = 1.0;

        for i in 1..=8 {
            if check(id.face, i, 0, 0) {
                light_val = 0.15; // Dark shadow immediately
                break;
            }
        }

        // boost light if it's the natural surface (Grass) to ensure terrain looks bright
        let natural_h = data.terrain.get_height(id.face, id.u, id.v);
        if id.layer >= natural_h {
            light_val = 1.0;
        }

        let voxel_id = data.get_voxel(id);
        let tex_index = voxel_id.raw() as u32;
        let mut base_color = data.content.color(voxel_id);

        // apply Skylight
        base_color[0] *= light_val;
        base_color[1] *= light_val;
        base_color[2] *= light_val;

        // geometry Helpers
        let p = |u_off: u32, v_off: u32, l_off: u32| {
            CoordSystem::get_vertex_pos(id.face, id.u + u_off, id.v + v_off, id.layer + l_off, res)
        };
        let i_bl = p(0, 0, 0);
        let i_br = p(1, 0, 0);
        let i_tl = p(0, 1, 0);
        let i_tr = p(1, 1, 0);
        let o_bl = p(0, 0, 1);
        let o_br = p(1, 0, 1);
        let o_tl = p(0, 1, 1);
        let o_tr = p(1, 1, 1);

        let apply =
            |ao: f32| -> [f32; 3] { [base_color[0] * ao, base_color[1] * ao, base_color[2] * ao] };

        if !has_top {
            let n = |u, v| check(id.face, 1, u, v);
            let ao_bl = ambient_occlusion::calculate(n(-1, 0), n(0, -1), n(-1, -1));
            let ao_br = ambient_occlusion::calculate(n(1, 0), n(0, -1), n(1, -1));
            let ao_tr = ambient_occlusion::calculate(n(1, 0), n(0, 1), n(1, 1));
            let ao_tl = ambient_occlusion::calculate(n(-1, 0), n(0, 1), n(-1, 1));
            Self::quad(
                verts,
                inds,
                idx,
                [o_bl, o_br, o_tr, o_tl],
                [apply(ao_bl), apply(ao_br), apply(ao_tr), apply(ao_tl)],
                true,
                tex_index,
            );
        }

        if !has_btm {
            let c = apply(0.4);
            Self::quad(
                verts,
                inds,
                idx,
                [i_tl, i_tr, i_br, i_bl],
                [c, c, c, c],
                true,
                tex_index,
            );
        }

        let side_c = apply(0.8);
        let colors = [side_c, side_c, side_c, side_c];

        if !has_front {
            Self::quad(
                verts,
                inds,
                idx,
                [i_bl, i_br, o_br, o_bl],
                colors,
                false,
                tex_index,
            );
        }
        if !has_back {
            Self::quad(
                verts,
                inds,
                idx,
                [o_tl, o_tr, i_tr, i_tl],
                colors,
                false,
                tex_index,
            );
        }
        if !has_left {
            Self::quad(
                verts,
                inds,
                idx,
                [i_tl, i_bl, o_bl, o_tl],
                colors,
                false,
                tex_index,
            );
        }
        if !has_right {
            Self::quad(
                verts,
                inds,
                idx,
                [i_br, i_tr, o_tr, o_br],
                colors,
                false,
                tex_index,
            );
        }
    }

    fn quad(
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        pos: [Vec3; 4],
        colors: [[f32; 3]; 4],
        force_radial: bool,
        tex_index: u32,
    ) {
        // UV corners: (0,0) bl, (1,0) br, (1,1) tr, (0,1) tl
        let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let normal = if force_radial {
            let center = (pos[0] + pos[1] + pos[2] + pos[3]) * 0.25;
            center.normalize().to_array()
        } else {
            (pos[1] - pos[0])
                .cross(pos[2] - pos[0])
                .normalize()
                .to_array()
        };

        for i in 0..4 {
            verts.push(CpuVertex {
                pos: pos[i].to_array(),
                uv: uvs[i],
                color: colors[i],
                normal,
                tex_index,
            });
        }

        inds.push(*idx);
        inds.push(*idx + 1);
        inds.push(*idx + 2);
        inds.push(*idx + 2);
        inds.push(*idx + 3);
        inds.push(*idx);
        *idx += 4;
    }
}

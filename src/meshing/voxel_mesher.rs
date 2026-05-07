use super::{ambient_occlusion, CpuMesh, CpuVertex, MeshGen};
use crate::generation::CoordSystem;
use crate::voxel::{SurfaceChunkKey, VoxelCoord, CHUNK_SIZE};
use crate::world::PlanetData;
use glam::Vec3;

#[derive(Default)]
struct CandidateBuffer {
    coords: Vec<VoxelCoord>,
}

impl CandidateBuffer {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            coords: Vec::with_capacity(capacity),
        }
    }

    fn push(&mut self, coord: VoxelCoord) {
        self.coords.push(coord);
    }

    fn finish(mut self) -> Vec<VoxelCoord> {
        self.coords
            .sort_by_key(|id| (id.face, id.layer, id.u, id.v));
        self.coords
            .dedup_by_key(|id| (id.face, id.layer, id.u, id.v));
        self.coords
    }
}

impl MeshGen {
    fn add_modified_candidates(id: VoxelCoord, candidates: &mut CandidateBuffer, res: u32) {
        candidates.push(id);
        if id.layer + 1 < res {
            candidates.push(VoxelCoord {
                layer: id.layer + 1,
                ..id
            });
        }
        if id.layer > 0 {
            candidates.push(VoxelCoord {
                layer: id.layer - 1,
                ..id
            });
        }
        if id.u > 0 {
            candidates.push(VoxelCoord { u: id.u - 1, ..id });
        }
        if id.u < res - 1 {
            candidates.push(VoxelCoord { u: id.u + 1, ..id });
        }
        if id.v > 0 {
            candidates.push(VoxelCoord { v: id.v - 1, ..id });
        }
        if id.v < res - 1 {
            candidates.push(VoxelCoord { v: id.v + 1, ..id });
        }
    }

    pub fn build_chunk(key: SurfaceChunkKey, data: &PlanetData) -> CpuMesh {
        let mut verts = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE * 4) as usize);
        let mut inds = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE * 6) as usize);
        let mut idx = 0u32;
        let res = data.resolution;
        let mut candidates = CandidateBuffer::with_capacity((CHUNK_SIZE * CHUNK_SIZE * 2) as usize);

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
                candidates.push(VoxelCoord {
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
                        candidates.push(VoxelCoord {
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
        for id in candidates.finish() {
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
        let visual = data.content.visual(voxel_id);
        let mut fallback_color = data.content.color(voxel_id);

        // apply Skylight
        fallback_color[0] *= light_val;
        fallback_color[1] *= light_val;
        fallback_color[2] *= light_val;

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

        let face_color = |layer: u32, ao: f32| -> [f32; 3] {
            let c = if layer == 0 {
                fallback_color
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
            let layer = visual.layers.top;
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
                [
                    face_color(layer, ao_bl),
                    face_color(layer, ao_br),
                    face_color(layer, ao_tr),
                    face_color(layer, ao_tl),
                ],
                true,
                layer,
            );
        }

        if !has_btm {
            let layer = visual.layers.bottom;
            let c = face_color(layer, 0.4);
            Self::quad(
                verts,
                inds,
                idx,
                [i_tl, i_tr, i_br, i_bl],
                [c, c, c, c],
                true,
                layer,
            );
        }

        if !has_front {
            let layer = visual.layers.front;
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                [i_bl, i_br, o_br, o_bl],
                [c, c, c, c],
                false,
                layer,
            );
        }
        if !has_back {
            let layer = visual.layers.back;
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                [o_tl, o_tr, i_tr, i_tl],
                [c, c, c, c],
                false,
                layer,
            );
        }
        if !has_left {
            let layer = visual.layers.left;
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                [i_tl, i_bl, o_bl, o_tl],
                [c, c, c, c],
                false,
                layer,
            );
        }
        if !has_right {
            let layer = visual.layers.right;
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                [i_br, i_tr, o_tr, o_br],
                [c, c, c, c],
                false,
                layer,
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

#[cfg(test)]
mod tests {
    use super::{CandidateBuffer, MeshGen};
    use crate::voxel::VoxelCoord;

    fn coord(layer: u32, u: u32, v: u32) -> VoxelCoord {
        VoxelCoord {
            face: 0,
            layer,
            u,
            v,
        }
    }

    #[test]
    fn candidate_buffer_deduplicates_deterministically() {
        let mut candidates = CandidateBuffer::default();
        candidates.push(coord(2, 1, 1));
        candidates.push(coord(1, 1, 1));
        candidates.push(coord(2, 1, 1));

        let ids = candidates.finish();
        assert_eq!(ids, vec![coord(1, 1, 1), coord(2, 1, 1)]);
    }

    #[test]
    fn modified_candidates_include_six_neighborhood_without_duplicates() {
        let mut candidates = CandidateBuffer::default();
        MeshGen::add_modified_candidates(coord(1, 1, 1), &mut candidates, 4);

        let ids = candidates.finish();
        assert_eq!(ids.len(), 7);
        assert!(ids.contains(&coord(1, 1, 1)));
        assert!(ids.contains(&coord(0, 1, 1)));
        assert!(ids.contains(&coord(2, 1, 1)));
        assert!(ids.contains(&coord(1, 0, 1)));
        assert!(ids.contains(&coord(1, 2, 1)));
        assert!(ids.contains(&coord(1, 1, 0)));
        assert!(ids.contains(&coord(1, 1, 2)));
    }
}

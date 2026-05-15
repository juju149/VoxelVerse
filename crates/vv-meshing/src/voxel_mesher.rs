use super::{
    ambient_occlusion, pack_material_edges, prop_baker::bake_props, CpuMesh, CpuVertex,
    FaceEdgeMask, GreedyMesher, MeshGen,
};
use glam::Vec3;
use std::sync::OnceLock;
use vv_math::CoordSystem;
use vv_pack_compiler::CompiledMesh;
use vv_voxel::{SurfaceChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};
use vv_world::PlanetData;
use vv_worldgen::ChunkFeatureMap;

/// Read-only voxel accessor used during meshing.
///
/// `PlanetData::get_voxel` works but resolves above-surface cells through the
/// expensive tree-neighbourhood scan. The mesher already paid for that scan
/// once via [`PlanetData::bake_chunk_features`]; this struct lets us reuse the
/// resulting map and short-circuit every above-surface lookup to an O(1)
/// hash-map probe.
pub(super) struct ChunkAccessor<'a> {
    data: &'a PlanetData,
    features: &'a ChunkFeatureMap,
}

impl<'a> ChunkAccessor<'a> {
    pub(super) fn new(data: &'a PlanetData, features: &'a ChunkFeatureMap) -> Self {
        Self { data, features }
    }

    pub(super) fn voxel_id(&self, coord: VoxelCoord) -> VoxelId {
        let res = self.data.resolution;
        if coord.u >= res || coord.v >= res || coord.layer >= res {
            return VoxelId::AIR;
        }
        if let Some(v) = self.data.voxels.get_override(coord) {
            return v;
        }
        let surface_h = self.data.terrain.get_height(coord.face, coord.u, coord.v);
        if coord.layer > surface_h {
            return self.features.get(coord).unwrap_or(VoxelId::AIR);
        }
        self.data.get_voxel(coord)
    }

    pub(super) fn has_renderable(&self, coord: VoxelCoord) -> bool {
        self.data.content.is_renderable(self.voxel_id(coord))
    }

    pub(super) fn is_opaque_cube(&self, coord: VoxelCoord) -> bool {
        self.data.content.is_opaque_cube(self.voxel_id(coord))
    }

    pub(super) fn uses_greedy_opaque_meshing(&self, coord: VoxelCoord) -> bool {
        self.data
            .content
            .uses_greedy_opaque_meshing(self.voxel_id(coord))
    }

    pub(super) fn data(&self) -> &'a PlanetData {
        self.data
    }
}

#[derive(Default)]
struct CandidateBuffer {
    coords: Vec<VoxelCoord>,
}

pub(super) struct QuadFace {
    pub(super) pos: [Vec3; 4],
    pub(super) colors: [[f32; 3]; 4],
    pub(super) force_radial: bool,
    pub(super) packed_tex_index: u32,
    pub(super) flip_u: bool,
    pub(super) flip_v: bool,
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
        let u_end = (u_start + CHUNK_SIZE).min(res);
        let v_end = (v_start + CHUNK_SIZE).min(res);

        // Bake all tree + visual-detail voxels for this chunk + a 1-voxel margin
        // (face culling needs to see across the chunk edge).
        let feature_map = data.bake_chunk_features(key, 1);
        let accessor = ChunkAccessor::new(data, &feature_map);

        let get_h = |f, u, v| -> u32 {
            if u >= res || v >= res {
                return 0;
            }
            data.terrain.get_height(f, u, v)
        };

        for u in u_start..u_end {
            for v in v_start..v_end {
                let h = get_h(key.face, u, v);
                if h == 0 {
                    continue;
                }

                // Always add the top surface block
                candidates.push(VoxelCoord {
                    face: key.face,
                    layer: h,
                    u,
                    v,
                });

                // Cliff fill: if a neighbour is lower, expose blocks down to it.
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

        // Pull every above-surface feature inside the chunk straight from the
        // baked map — replaces the previous 32-layer probe loop.
        for &coord in feature_map.blocks.keys() {
            if coord.face == key.face
                && coord.u >= u_start
                && coord.u < u_end
                && coord.v >= v_start
                && coord.v < v_end
            {
                candidates.push(coord);
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

        let candidates = candidates.finish();
        let greedy_enabled = greedy_meshing_enabled();
        if greedy_enabled {
            GreedyMesher::append_opaque_cubes(
                &accessor,
                &candidates,
                &mut verts,
                &mut inds,
                &mut idx,
            );
        }

        for id in candidates {
            if id.u >= u_start
                && id.u < u_end
                && id.v >= v_start
                && id.v < v_end
                && accessor.has_renderable(id)
                && (!greedy_enabled || !accessor.uses_greedy_opaque_meshing(id))
            {
                Self::add_voxel(id, &accessor, &mut verts, &mut inds, &mut idx);
            }
        }

        let mut mesh = CpuMesh::new(verts, inds);

        // Append .vox prop geometry (grass, flowers, mushrooms, …).
        // Props are NOT in the voxel grid — query the terrain procedural layer.
        // Filter out columns where the prop or its support block was broken by the player.
        //
        // No LOD gate — all loaded chunks receive full prop geometry.
        // Performance is bounded by the jittered-grid (~113 candidates/chunk)
        // and the per-chunk quad cap (MAX_QUADS_PER_CHUNK) already baked into
        // the baker, so culling by distance is not needed.
        {
            let stamps = data.terrain.props_for_chunk(key);
            if !stamps.is_empty() {
                let alive_stamps: Vec<_> = stamps
                    .into_iter()
                    .filter(|s| data.broken_props.is_alive(s.face, s.u, s.v))
                    .collect();
                if !alive_stamps.is_empty() {
                    bake_props(&alive_stamps, &data.prop_models, data.profile, &mut mesh);
                }
            }
        }

        mesh
    }

    fn add_voxel(
        id: VoxelCoord,
        accessor: &ChunkAccessor,
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        let voxel_id = accessor.voxel_id(id);
        let model = accessor.data.content.model_of(voxel_id);
        match &model.mesh {
            CompiledMesh::Cube { .. } | CompiledMesh::CubeColumn { .. } => {
                Self::add_cube_voxel(id, accessor, verts, inds, idx)
            }
            CompiledMesh::None => {
                // Air-like blocks with no geometry; the candidate buffer
                // already filters them via `is_renderable`, so reaching
                // here means an engine bug rather than benign content.
                debug_assert!(false, "mesher visited a None-mesh block");
            }
        }
    }

    fn add_cube_voxel(
        id: VoxelCoord,
        accessor: &ChunkAccessor,
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        let data = accessor.data;
        let res = data.resolution;

        // Neighbour cube-occlusion check (cross-plane neighbours never occlude).
        let check = |d_face: u8, d_layer: i32, d_u: i32, d_v: i32| -> bool {
            let l = id.layer as i32 + d_layer;
            let u = id.u as i32 + d_u;
            let v = id.v as i32 + d_v;
            if l >= 0 && u >= 0 && u < res as i32 && v >= 0 && v < res as i32 {
                return accessor.is_opaque_cube(VoxelCoord {
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

        let voxel_id = accessor.voxel_id(id);
        let visual = data.content.visual(voxel_id);
        let mut fallback_color = data.content.color(voxel_id);

        // apply Skylight
        fallback_color[0] *= light_val;
        fallback_color[1] *= light_val;
        fallback_color[2] *= light_val;

        // geometry Helpers
        let p = |u_off: u32, v_off: u32, l_off: u32| {
            CoordSystem::get_vertex_pos(
                id.face,
                id.u + u_off,
                id.v + v_off,
                id.layer + l_off,
                data.profile,
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
            let edges = FaceEdgeMask {
                min_u: !has_left,
                max_u: !has_right,
                min_v: !has_front,
                max_v: !has_back,
            };
            let n = |u, v| check(id.face, 1, u, v);
            let ao_bl = ambient_occlusion::calculate(n(-1, 0), n(0, -1), n(-1, -1));
            let ao_br = ambient_occlusion::calculate(n(1, 0), n(0, -1), n(1, -1));
            let ao_tr = ambient_occlusion::calculate(n(1, 0), n(0, 1), n(1, 1));
            let ao_tl = ambient_occlusion::calculate(n(-1, 0), n(0, 1), n(-1, 1));
            Self::quad(
                verts,
                inds,
                idx,
                QuadFace {
                    pos: [o_bl, o_br, o_tr, o_tl],
                    colors: [
                        face_color(layer, ao_bl),
                        face_color(layer, ao_br),
                        face_color(layer, ao_tr),
                        face_color(layer, ao_tl),
                    ],
                    force_radial: true,
                    packed_tex_index: pack_material_edges(layer, edges),
                    flip_u: false,
                    flip_v: false,
                },
            );
        }

        if !has_btm {
            let layer = visual.layers.bottom;
            let edges = FaceEdgeMask {
                min_u: !has_left,
                max_u: !has_right,
                min_v: !has_back,
                max_v: !has_front,
            };
            let c = face_color(layer, 0.4);
            Self::quad(
                verts,
                inds,
                idx,
                QuadFace {
                    pos: [i_tl, i_tr, i_br, i_bl],
                    colors: [c, c, c, c],
                    force_radial: true,
                    packed_tex_index: pack_material_edges(layer, edges),
                    flip_u: false,
                    flip_v: true, // v is flipped when viewed from below
                },
            );
        }

        if !has_front {
            let layer = visual.layers.front;
            let edges = FaceEdgeMask {
                min_u: !has_left,
                max_u: !has_right,
                min_v: !has_top,
                max_v: !has_btm,
            };
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                QuadFace {
                    pos: [i_bl, i_br, o_br, o_bl],
                    colors: [c, c, c, c],
                    force_radial: false,
                    packed_tex_index: pack_material_edges(layer, edges),
                    flip_u: false,
                    flip_v: true,
                },
            );
        }
        if !has_back {
            let layer = visual.layers.back;
            let edges = FaceEdgeMask {
                min_u: !has_left,
                max_u: !has_right,
                min_v: !has_top,
                max_v: !has_btm,
            };
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                QuadFace {
                    pos: [i_tl, i_tr, o_tr, o_tl],
                    colors: [c, c, c, c],
                    force_radial: false,
                    packed_tex_index: pack_material_edges(layer, edges),
                    flip_u: false,
                    flip_v: true,
                },
            );
        }
        if !has_left {
            let layer = visual.layers.left;
            let edges = FaceEdgeMask {
                min_u: !has_back,
                max_u: !has_front,
                min_v: !has_top,
                max_v: !has_btm,
            };
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                QuadFace {
                    pos: [i_bl, i_tl, o_tl, o_bl],
                    colors: [c, c, c, c],
                    force_radial: false,
                    packed_tex_index: pack_material_edges(layer, edges),
                    flip_u: false,
                    flip_v: true,
                },
            );
        }
        if !has_right {
            let layer = visual.layers.right;
            let edges = FaceEdgeMask {
                min_u: !has_front,
                max_u: !has_back,
                min_v: !has_top,
                max_v: !has_btm,
            };
            let c = face_color(layer, 0.8);
            Self::quad(
                verts,
                inds,
                idx,
                QuadFace {
                    pos: [i_br, i_tr, o_tr, o_br],
                    colors: [c, c, c, c],
                    force_radial: false,
                    packed_tex_index: pack_material_edges(layer, edges),
                    flip_u: false,
                    flip_v: true,
                },
            );
        }
    }

    pub(super) fn quad(
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        face: QuadFace,
    ) {
        Self::quad_tiled(verts, inds, idx, face, [1.0, 1.0]);
    }

    pub(super) fn quad_tiled(
        verts: &mut Vec<CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        face: QuadFace,
        uv_span: [f32; 2],
    ) {
        // UV corners: (0,0) bl, (1,0) br, (1,1) tr, (0,1) tl
        // flip_u mirrors horizontally, flip_v mirrors vertically.
        let u0 = if face.flip_u { uv_span[0] } else { 0.0_f32 };
        let u1 = if face.flip_u { 0.0_f32 } else { uv_span[0] };
        let v0 = if face.flip_v { uv_span[1] } else { 0.0_f32 };
        let v1 = if face.flip_v { 0.0_f32 } else { uv_span[1] };
        let uvs: [[f32; 2]; 4] = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];

        let normal = if face.force_radial {
            let center = (face.pos[0] + face.pos[1] + face.pos[2] + face.pos[3]) * 0.25;
            center.normalize().to_array()
        } else {
            (face.pos[1] - face.pos[0])
                .cross(face.pos[2] - face.pos[0])
                .normalize()
                .to_array()
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
    use super::{CandidateBuffer, MeshGen, QuadFace};
    use glam::Vec3;
    use vv_voxel::VoxelCoord;

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

    #[test]
    fn quad_flip_v_maps_texture_top_to_last_edge() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0;
        MeshGen::quad(
            &mut verts,
            &mut inds,
            &mut idx,
            QuadFace {
                pos: [
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(1.0, 1.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                ],
                colors: [[1.0, 1.0, 1.0]; 4],
                force_radial: false,
                packed_tex_index: 0,
                flip_u: false,
                flip_v: true,
            },
        );

        assert_eq!(
            verts.iter().map(|v| v.uv).collect::<Vec<_>>(),
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],]
        );
        assert_eq!(inds, [0, 1, 2, 2, 3, 0]);
        assert_eq!(idx, 4);
    }

    #[test]
    fn quad_tiled_preserves_texture_repeat_span() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0;
        MeshGen::quad_tiled(
            &mut verts,
            &mut inds,
            &mut idx,
            QuadFace {
                pos: [
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(2.0, 0.0, 0.0),
                    Vec3::new(2.0, 3.0, 0.0),
                    Vec3::new(0.0, 3.0, 0.0),
                ],
                colors: [[1.0, 1.0, 1.0]; 4],
                force_radial: false,
                packed_tex_index: 0,
                flip_u: false,
                flip_v: false,
            },
            [2.0, 3.0],
        );

        assert_eq!(
            verts.iter().map(|v| v.uv).collect::<Vec<_>>(),
            [[0.0, 0.0], [2.0, 0.0], [2.0, 3.0], [0.0, 3.0],]
        );
        assert_eq!(inds, [0, 1, 2, 2, 3, 0]);
    }
}

fn greedy_meshing_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        !std::env::var("VV_DISABLE_GREEDY_MESH")
            .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    })
}

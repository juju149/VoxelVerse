use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use std::collections::{HashMap, HashSet};
use vv_core::{BlockId, ChunkKey, LodKey, CHUNK_SIZE};
use vv_planet::CoordSystem;
use vv_registry::{
    BlockId as ContentBlockId, BlockRenderSource, CompiledBlockFace, CompiledBlockRender,
    CompiledVisualMaterialType, TextureId,
};
use vv_world_runtime::{ChunkMods, PlanetData};

// --- Vertex format ----------------------------------------------------------

/// GPU-ready vertex: position, per-vertex colour, surface normal, atlas UV and texture id.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// -1 means no block texture; shader uses the fallback color directly.
    pub texture_id: i32,
    /// -1 means non-block helper geometry; shader uses a neutral fallback material.
    pub block_id: i32,
}

impl Vertex {
    pub fn untextured(pos: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            pos,
            color,
            normal,
            uv: [0.0, 0.0],
            texture_id: -1,
            block_id: -1,
        }
    }
}

// --- CPU mesh builder -------------------------------------------------------

pub struct MeshGen;

#[derive(Clone, Copy, Debug)]
struct VisualBevel {
    top_edge: f32,
    side_edge: f32,
}

impl MeshGen {
    // --- Chunk mesh ---------------------------------------------------------

    pub fn build_chunk(
        key: ChunkKey,
        data: &PlanetData,
        blocks: &impl BlockRenderSource,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let mut idx = 0u32;
        let res = data.resolution;
        let mut candidates: HashSet<BlockId> = HashSet::new();

        let u_start = key.u_idx * CHUNK_SIZE;
        let v_start = key.v_idx * CHUNK_SIZE;
        let u_end = (u_start + CHUNK_SIZE).min(res);
        let v_end = (v_start + CHUNK_SIZE).min(res);
        let feature_blocks = data
            .terrain
            .feature_blocks_in_region(key.face, u_start, v_start, u_end, v_end);

        let get_h = |f: u8, u: u32, v: u32| -> u32 {
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
                candidates.insert(BlockId {
                    face: key.face,
                    layer: h,
                    u,
                    v,
                });
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
                        candidates.insert(BlockId {
                            face: key.face,
                            layer: l,
                            u,
                            v,
                        });
                    }
                }
            }
        }
        candidates.extend(feature_blocks.keys().copied());

        if let Some(mods) = data.chunks.get(&key) {
            for &id in mods.placed.keys() {
                candidates.insert(id);
            }
            Self::add_mined_candidates(mods, &mut candidates, res);
        }

        let neighbor_keys = [
            ChunkKey {
                u_idx: key.u_idx.wrapping_sub(1),
                ..key
            },
            ChunkKey {
                u_idx: key.u_idx + 1,
                ..key
            },
            ChunkKey {
                v_idx: key.v_idx.wrapping_sub(1),
                ..key
            },
            ChunkKey {
                v_idx: key.v_idx + 1,
                ..key
            },
        ];
        for n_key in neighbor_keys {
            if let Some(mods) = data.chunks.get(&n_key) {
                Self::add_mined_candidates(mods, &mut candidates, res);
            }
        }

        for id in candidates {
            if id.u >= u_start && id.u < u_end && id.v >= v_start && id.v < v_end {
                if feature_blocks.contains_key(&id) || data.exists(id) {
                    Self::add_voxel(
                        id,
                        data,
                        blocks,
                        &feature_blocks,
                        (u_start, v_start, u_end, v_end),
                        &mut verts,
                        &mut inds,
                        &mut idx,
                    );
                }
            }
        }
        (verts, inds)
    }

    fn add_mined_candidates(mods: &ChunkMods, candidates: &mut HashSet<BlockId>, res: u32) {
        for &id in &mods.mined {
            candidates.insert(BlockId {
                layer: id.layer + 1,
                ..id
            });
            if id.layer > 0 {
                candidates.insert(BlockId {
                    layer: id.layer - 1,
                    ..id
                });
            }
            if id.u > 0 {
                candidates.insert(BlockId { u: id.u - 1, ..id });
            }
            if id.u < res - 1 {
                candidates.insert(BlockId { u: id.u + 1, ..id });
            }
            if id.v > 0 {
                candidates.insert(BlockId { v: id.v - 1, ..id });
            }
            if id.v < res - 1 {
                candidates.insert(BlockId { v: id.v + 1, ..id });
            }
        }
    }

    fn add_voxel(
        id: BlockId,
        data: &PlanetData,
        blocks: &impl BlockRenderSource,
        feature_blocks: &HashMap<BlockId, ContentBlockId>,
        target_region: (u32, u32, u32, u32),
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        let res = data.resolution;
        let Some(block_id) = Self::mesh_block_at(data, id, feature_blocks, target_region) else {
            return;
        };
        let Some(render) = blocks.block_render(block_id) else {
            return;
        };

        let check = |_face: u8, d_layer: i32, d_u: i32, d_v: i32| -> bool {
            let l = id.layer as i32 + d_layer;
            let u = id.u as i32 + d_u;
            let v = id.v as i32 + d_v;
            if l >= 0 && u >= 0 && u < res as i32 && v >= 0 && v < res as i32 {
                let neighbor = BlockId {
                    face: id.face,
                    layer: l as u32,
                    u: u as u32,
                    v: v as u32,
                };
                let Some(neighbor_block) =
                    Self::mesh_block_at(data, neighbor, feature_blocks, target_region)
                else {
                    return false;
                };
                return blocks
                    .block_render(neighbor_block)
                    .map(|neighbor_render| !neighbor_render.translucent)
                    .unwrap_or(false);
            }
            l < 0
        };

        let has_top = check(id.face, 1, 0, 0);
        let has_btm = check(id.face, -1, 0, 0);
        let has_right = check(id.face, 0, 1, 0);
        let has_left = check(id.face, 0, -1, 0);
        let has_back = check(id.face, 0, 0, 1);
        let has_front = check(id.face, 0, 0, -1);

        if has_top && has_btm && has_left && has_right && has_front && has_back {
            return;
        }

        let mut light_val: f32 = 1.0;
        for i in 1..=8 {
            if check(id.face, i, 0, 0) {
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
        let apply = |ao: f32, texture_id: i32| -> [f32; 3] {
            if texture_id >= 0 {
                let light = light_val * ao;
                [light, light, light]
            } else {
                [base_color[0] * ao, base_color[1] * ao, base_color[2] * ao]
            }
        };

        if !has_top {
            let texture_id = Self::face_texture_id(render.texture_for_face(CompiledBlockFace::Top));
            let n = |u, v| check(id.face, 1, u, v);
            let ao_bl = Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1));
            let ao_br = Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1));
            let ao_tr = Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1));
            let ao_tl = Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1));
            Self::quad(
                verts,
                inds,
                idx,
                [o_bl, o_br, o_tr, o_tl],
                [
                    apply(ao_bl, texture_id),
                    apply(ao_br, texture_id),
                    apply(ao_tr, texture_id),
                    apply(ao_tl, texture_id),
                ],
                texture_id,
                block_raw_id,
                true,
                Some(top_normals),
            );
        }
        if !has_btm {
            let texture_id =
                Self::face_texture_id(render.texture_for_face(CompiledBlockFace::Bottom));
            let c = apply(0.4, texture_id);
            Self::quad(
                verts,
                inds,
                idx,
                [i_tl, i_tr, i_br, i_bl],
                [c, c, c, c],
                texture_id,
                block_raw_id,
                true,
                Some(bottom_normals),
            );
        }
        if !has_front {
            let texture_id =
                Self::face_texture_id(render.texture_for_face(CompiledBlockFace::North));
            let side_c = apply(0.8, texture_id);
            let sc = [side_c, side_c, side_c, side_c];
            Self::quad(
                verts,
                inds,
                idx,
                [i_bl, i_br, o_br, o_bl],
                sc,
                texture_id,
                block_raw_id,
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
        if !has_back {
            let texture_id =
                Self::face_texture_id(render.texture_for_face(CompiledBlockFace::South));
            let side_c = apply(0.8, texture_id);
            let sc = [side_c, side_c, side_c, side_c];
            Self::quad(
                verts,
                inds,
                idx,
                [o_tl, o_tr, i_tr, i_tl],
                sc,
                texture_id,
                block_raw_id,
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
        if !has_left {
            let texture_id =
                Self::face_texture_id(render.texture_for_face(CompiledBlockFace::West));
            let side_c = apply(0.8, texture_id);
            let sc = [side_c, side_c, side_c, side_c];
            Self::quad(
                verts,
                inds,
                idx,
                [i_tl, i_bl, o_bl, o_tl],
                sc,
                texture_id,
                block_raw_id,
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
        if !has_right {
            let texture_id =
                Self::face_texture_id(render.texture_for_face(CompiledBlockFace::East));
            let side_c = apply(0.8, texture_id);
            let sc = [side_c, side_c, side_c, side_c];
            Self::quad(
                verts,
                inds,
                idx,
                [i_br, i_tr, o_tr, o_br],
                sc,
                texture_id,
                block_raw_id,
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
    }

    fn mesh_block_at(
        data: &PlanetData,
        id: BlockId,
        feature_blocks: &HashMap<BlockId, ContentBlockId>,
        target_region: (u32, u32, u32, u32),
    ) -> Option<ContentBlockId> {
        if let Some(block_id) = feature_blocks.get(&id) {
            return Some(*block_id);
        }

        let (u_start, v_start, u_end, v_end) = target_region;
        let inside_region = id.u >= u_start && id.u < u_end && id.v >= v_start && id.v < v_end;
        if !inside_region {
            return data.block_at(id);
        }

        let key = PlanetData::chunk_key(id);
        if let Some(mods) = data.chunks.get(&key) {
            if let Some(block_id) = mods.placed.get(&id) {
                return Some(*block_id);
            }
            if mods.mined.contains(&id) {
                return None;
            }
        }

        let height = data.terrain.get_height(id.face, id.u, id.v);
        if id.layer <= height {
            Some(data.terrain.get_block(id.face, id.u, id.v, id.layer))
        } else {
            None
        }
    }

    fn calculate_ao(side1: bool, side2: bool, corner: bool) -> f32 {
        let mut occ = 0;
        if side1 {
            occ += 1;
        }
        if side2 {
            occ += 1;
        }
        if corner && (side1 || side2) {
            occ += 1;
        }
        match occ {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            _ => 0.4,
        }
    }

    fn visual_bevel(render: &CompiledBlockRender) -> VisualBevel {
        match render.material.visual_type {
            CompiledVisualMaterialType::Grass
            | CompiledVisualMaterialType::Dirt
            | CompiledVisualMaterialType::Snow
            | CompiledVisualMaterialType::Sand
            | CompiledVisualMaterialType::Leaves => VisualBevel {
                top_edge: 0.30,
                side_edge: 0.24,
            },
            CompiledVisualMaterialType::Stone
            | CompiledVisualMaterialType::Ore
            | CompiledVisualMaterialType::Ice => VisualBevel {
                top_edge: 0.22,
                side_edge: 0.17,
            },
            CompiledVisualMaterialType::Wood => VisualBevel {
                top_edge: 0.16,
                side_edge: 0.12,
            },
            CompiledVisualMaterialType::CutStone | CompiledVisualMaterialType::Planks => {
                VisualBevel {
                    top_edge: 0.06,
                    side_edge: 0.045,
                }
            }
            CompiledVisualMaterialType::Generic | CompiledVisualMaterialType::Water => {
                VisualBevel {
                    top_edge: 0.0,
                    side_edge: 0.0,
                }
            }
        }
    }

    fn face_normal(pos: [Vec3; 4]) -> Vec3 {
        (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize()
    }

    fn rounded_corner_normals(
        base: Vec3,
        adjacent_faces: [[(bool, Vec3); 2]; 4],
        strength: f32,
    ) -> [Vec3; 4] {
        if strength <= 0.0 {
            return [base; 4];
        }

        let mut normals = [base; 4];
        for (normal, adjacent) in normals.iter_mut().zip(adjacent_faces) {
            let mut blended = base;
            for (visible, face_normal) in adjacent {
                if visible {
                    blended += face_normal * strength;
                }
            }
            *normal = blended.normalize();
        }
        normals
    }

    fn quad(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        pos: [Vec3; 4],
        colors: [[f32; 3]; 4],
        texture_id: i32,
        block_id: i32,
        force_radial: bool,
        normals: Option<[Vec3; 4]>,
    ) {
        let fallback_normal = if force_radial {
            ((pos[0] + pos[1] + pos[2] + pos[3]) * 0.25).normalize()
        } else {
            (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize()
        };
        let normals = normals.unwrap_or([fallback_normal; 4]);
        let base = *idx;
        let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        for (((p, c), uv), normal) in pos.iter().zip(colors.iter()).zip(uvs).zip(normals) {
            verts.push(Vertex {
                pos: p.to_array(),
                color: *c,
                normal: normal.to_array(),
                uv,
                texture_id,
                block_id,
            });
        }
        inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        *idx += 4;
    }

    fn face_texture_id(texture: Option<TextureId>) -> i32 {
        texture.map(|id| id.raw() as i32).unwrap_or(-1)
    }

    // --- LOD heightmap tile -------------------------------------------------

    /// Build a simplified heightmap mesh for a distant LOD tile.
    /// `grid_res` controls vertex density (from `LodConfig::tile_grid_res`).
    pub fn generate_lod_mesh(
        key: LodKey,
        data: &PlanetData,
        grid_res: u32,
        blocks: &impl BlockRenderSource,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let row_len = grid_res + 1;

        let sample = |gx: i32, gy: i32| -> (Vec3, [f32; 3], [f32; 2], i32, i32) {
            let step_u = (gx as i64 * key.size as i64) / grid_res as i64;
            let step_v = (gy as i64 * key.size as i64) / grid_res as i64;
            let abs_u = (key.x as i64 + step_u).clamp(0, data.resolution as i64) as u32;
            let abs_v = (key.y as i64 + step_v).clamp(0, data.resolution as i64) as u32;
            let (layer, color, texture_id, block_id) =
                Self::lod_visual_surface(key.face, abs_u, abs_v, data, blocks);
            let pos = CoordSystem::get_vertex_pos(key.face, abs_u, abs_v, layer, data.geometry);
            let uv = [
                (abs_u as f32 / data.resolution.max(1) as f32).fract(),
                (abs_v as f32 / data.resolution.max(1) as f32).fract(),
            ];
            (pos, color, uv, texture_id, block_id)
        };
        let padded_len = grid_res as usize + 3;
        let mut samples = Vec::with_capacity(padded_len * padded_len);
        for vy in -1..=(grid_res as i32 + 1) {
            for ux in -1..=(grid_res as i32 + 1) {
                samples.push(sample(ux, vy));
            }
        }
        let sample_at = |ux: i32,
                         vy: i32,
                         samples: &[(Vec3, [f32; 3], [f32; 2], i32, i32)]|
         -> (Vec3, [f32; 3], [f32; 2], i32, i32) {
            let x = (ux + 1) as usize;
            let y = (vy + 1) as usize;
            samples[y * padded_len + x]
        };

        for vy in 0..=grid_res {
            for ux in 0..=grid_res {
                let ux = ux as i32;
                let vy = vy as i32;
                let (pos, mut color, uv, texture_id, block_id) = sample_at(ux, vy, &samples);
                let (p_right, _, _, _, _) = sample_at(ux + 1, vy, &samples);
                let (p_left, _, _, _, _) = sample_at(ux - 1, vy, &samples);
                let (p_down, _, _, _, _) = sample_at(ux, vy + 1, &samples);
                let (p_up, _, _, _, _) = sample_at(ux, vy - 1, &samples);
                let tangent_u = p_right - p_left;
                let tangent_v = p_down - p_up;
                let mut normal = tangent_u.cross(tangent_v).normalize();
                if normal.dot(pos.normalize()) < 0.0 {
                    normal = -normal;
                }

                let slope = normal.dot(pos.normalize()).abs();
                if slope < 0.85 {
                    color = [color[0] * 0.75, color[1] * 0.75, color[2] * 0.75];
                }

                verts.push(Vertex {
                    pos: pos.to_array(),
                    color,
                    normal: normal.to_array(),
                    uv,
                    texture_id,
                    block_id,
                });
            }
        }

        for y in 0..grid_res {
            for x in 0..grid_res {
                let tl = y * row_len + x;
                let tr = tl + 1;
                let bl = (y + 1) * row_len + x;
                let br = bl + 1;
                inds.push(tl);
                inds.push(bl);
                inds.push(tr);
                inds.push(tr);
                inds.push(bl);
                inds.push(br);
            }
        }

        // Skirts to hide seams between LOD levels
        let radius = CoordSystem::get_layer_radius(data.geometry.surface_layer(), data.geometry);
        let chunk_phys_size = (key.size as f32 / data.resolution as f32) * radius;
        let skirt_depth = (chunk_phys_size * 0.15).clamp(4.0, 500.0);

        let mut add_skirt_edge = |coord_pairs: &[(u32, u32)], reverse: bool| {
            let base_idx = verts.len() as u32;
            for &(ux, vy) in coord_pairs {
                let src_idx = vy * row_len + ux;
                let src_v = verts[src_idx as usize];
                let p = Vec3::from_array(src_v.pos);
                let down = -p.normalize() * skirt_depth;
                verts.push(Vertex {
                    pos: (p + down).to_array(),
                    color: src_v.color,
                    normal: src_v.normal,
                    uv: src_v.uv,
                    texture_id: src_v.texture_id,
                    block_id: src_v.block_id,
                });
            }
            let len = coord_pairs.len() as u32;
            for i in 0..(len - 1) {
                let s1 = coord_pairs[i as usize].1 * row_len + coord_pairs[i as usize].0;
                let s2 =
                    coord_pairs[(i + 1) as usize].1 * row_len + coord_pairs[(i + 1) as usize].0;
                let k1 = base_idx + i;
                let k2 = base_idx + i + 1;
                if reverse {
                    inds.push(s1);
                    inds.push(k2);
                    inds.push(k1);
                    inds.push(s1);
                    inds.push(s2);
                    inds.push(k2);
                } else {
                    inds.push(s1);
                    inds.push(k1);
                    inds.push(k2);
                    inds.push(s1);
                    inds.push(k2);
                    inds.push(s2);
                }
            }
        };

        let top: Vec<(u32, u32)> = (0..=grid_res).map(|x| (x, 0)).collect();
        let bottom: Vec<(u32, u32)> = (0..=grid_res).map(|x| (x, grid_res)).collect();
        let left: Vec<(u32, u32)> = (0..=grid_res).map(|y| (0, y)).collect();
        let right: Vec<(u32, u32)> = (0..=grid_res).map(|y| (grid_res, y)).collect();
        add_skirt_edge(&top, false);
        add_skirt_edge(&bottom, true);
        add_skirt_edge(&left, true);
        add_skirt_edge(&right, false);

        (verts, inds)
    }

    fn lod_visual_surface(
        face: u8,
        u: u32,
        v: u32,
        data: &PlanetData,
        blocks: &impl BlockRenderSource,
    ) -> (u32, [f32; 3], i32, i32) {
        let surface = data.terrain.get_height(face, u, v);
        for offset in 0..=8 {
            let layer = surface.saturating_sub(offset);
            let block = data.terrain.get_block(face, u, v, layer);
            if let Some(render) = blocks.block_render(block) {
                if !render.translucent {
                    let texture_id =
                        Self::face_texture_id(render.texture_for_face(CompiledBlockFace::Top));
                    let color = if texture_id >= 0 {
                        [1.0, 1.0, 1.0]
                    } else {
                        render.color
                    };
                    return (layer, color, texture_id, block.raw() as i32);
                }
            }
        }

        let block = data.terrain.get_surface_block(face, u, v);
        let color = blocks
            .block_render(block)
            .map(|render| render.color)
            .unwrap_or([0.45, 0.70, 0.45]);
        let texture_id = blocks
            .block_render(block)
            .and_then(|render| render.texture_for_face(CompiledBlockFace::Top))
            .map(|id| id.raw() as i32)
            .unwrap_or(-1);
        let color = if texture_id >= 0 {
            [1.0, 1.0, 1.0]
        } else {
            color
        };
        (surface, color, texture_id, block.raw() as i32)
    }

    // --- Collision debug mesh -----------------------------------------------

    pub fn generate_collision_debug(
        player_pos: Vec3,
        planet: &PlanetData,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let res = planet.resolution;
        let color = [1.0, 0.0, 0.0];
        let normal = [0.0, 1.0, 0.0];
        let range: i32 = 2;
        let mut idx: u32 = 0;

        if let Some((center_id, _)) = CoordSystem::get_local_coords(player_pos, planet.geometry) {
            let su = (center_id.u as i32 - range).max(0);
            let eu = (center_id.u as i32 + range).min(res as i32 - 1);
            let sv = (center_id.v as i32 - range).max(0);
            let ev = (center_id.v as i32 + range).min(res as i32 - 1);
            let sl = (center_id.layer as i32 - range).max(0);
            let el = (center_id.layer as i32 + range).min(res as i32 - 1);

            for l in sl..=el {
                for v in sv..=ev {
                    for u in su..=eu {
                        let id = BlockId {
                            face: center_id.face,
                            layer: l as u32,
                            u: u as u32,
                            v: v as u32,
                        };
                        if !planet.exists(id) {
                            continue;
                        }
                        let gp = |uu, vv, ll| {
                            CoordSystem::get_vertex_pos(
                                id.face,
                                id.u + uu,
                                id.v + vv,
                                id.layer + ll,
                                planet.geometry,
                            )
                        };
                        let c000 = gp(0, 0, 0);
                        let c100 = gp(1, 0, 0);
                        let c010 = gp(0, 1, 0);
                        let c110 = gp(1, 1, 0);
                        let c001 = gp(0, 0, 1);
                        let c101 = gp(1, 0, 1);
                        let c011 = gp(0, 1, 1);
                        let c111 = gp(1, 1, 1);
                        let center =
                            (c000 + c100 + c010 + c110 + c001 + c101 + c011 + c111) * 0.125;
                        let sh = 0.90f32;
                        let vi = |p: Vec3| {
                            Vertex::untextured(
                                (center + (p - center) * sh).to_array(),
                                color,
                                normal,
                            )
                        };
                        let corners = [
                            vi(c000),
                            vi(c100),
                            vi(c110),
                            vi(c010),
                            vi(c001),
                            vi(c101),
                            vi(c111),
                            vi(c011),
                        ];
                        for c in &corners {
                            verts.push(*c);
                        }
                        let b = idx;
                        let lines = [
                            (0, 1),
                            (1, 2),
                            (2, 3),
                            (3, 0),
                            (4, 5),
                            (5, 6),
                            (6, 7),
                            (7, 4),
                            (0, 4),
                            (1, 5),
                            (2, 6),
                            (3, 7),
                        ];
                        for (s, e) in lines {
                            inds.push(b + s);
                            inds.push(b + e);
                        }
                        idx += 8;
                    }
                }
            }
        }
        (verts, inds)
    }

    // --- Utility meshes -----------------------------------------------------

    pub fn generate_cylinder(radius: f32, height: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let color = [0.0, 0.5, 1.0];
        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * radius;
            let z = theta.sin() * radius;
            let n = Vec3::new(x, 0.0, z).normalize().to_array();
            verts.push(Vertex::untextured([x, 0.0, z], color, n));
            verts.push(Vertex::untextured([x, height, z], color, n));
        }
        for i in 0..segments {
            let b1 = i * 2;
            let t1 = b1 + 1;
            let b2 = b1 + 2;
            let t2 = b1 + 3;
            inds.extend_from_slice(&[b1, t1, b2, b2, t1, t2]);
        }
        let ci = verts.len() as u32;
        verts.push(Vertex::untextured(
            [0.0, height, 0.0],
            color,
            [0.0, 1.0, 0.0],
        ));
        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            verts.push(Vertex::untextured(
                [theta.cos() * radius, height, theta.sin() * radius],
                color,
                [0.0, 1.0, 0.0],
            ));
        }
        for i in 0..segments {
            inds.push(ci);
            inds.push(ci + 1 + i);
            inds.push(ci + 2 + i);
        }
        (verts, inds)
    }

    pub fn generate_sphere_guide(radius: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let color = [1.0, 1.0, 1.0];
        for y in 0..=segments {
            for x in 0..=segments {
                let xs = x as f32 / segments as f32;
                let ys = y as f32 / segments as f32;
                let xp = (xs * std::f32::consts::TAU).cos() * (ys * std::f32::consts::PI).sin();
                let yp = (ys * std::f32::consts::PI).cos();
                let zp = (xs * std::f32::consts::TAU).sin() * (ys * std::f32::consts::PI).sin();
                verts.push(Vertex::untextured(
                    [xp * radius, yp * radius, zp * radius],
                    color,
                    [xp, yp, zp],
                ));
            }
        }
        for y in 0..segments {
            for x in 0..segments {
                let i = y * (segments + 1) + x;
                inds.extend_from_slice(&[
                    i,
                    i + segments + 1,
                    i + segments + 2,
                    i + segments + 2,
                    i + 1,
                    i,
                ]);
            }
        }
        (verts, inds)
    }

    pub fn generate_crosshair() -> (Vec<Vertex>, Vec<u32>) {
        let s = 0.02f32;
        let color = [1.0, 1.0, 1.0];
        let normal = [0.0, 0.0, 1.0];
        let verts = vec![
            Vertex::untextured([-s, 0.0, 0.0], color, normal),
            Vertex::untextured([s, 0.0, 0.0], color, normal),
            Vertex::untextured([0.0, -s, 0.0], color, normal),
            Vertex::untextured([0.0, s, 0.0], color, normal),
        ];
        (verts, vec![0, 1, 2, 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vv_compiler::compile_assets_root;
    use vv_config::WorldGenConfig;
    use vv_world_gen::PlanetTerrain;

    #[test]
    fn chunk_mesh_uses_registry_block_render_color() {
        let geometry = vv_planet::PlanetGeometry::with_resolution(8.0, 0.5, 8);
        let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let block_content = content.to_block_content();
        let terrain = PlanetTerrain::generate_for_geometry(
            geometry,
            &WorldGenConfig::default(),
            &content.worldgen_content(),
        )
        .expect("terrain should generate");
        let planet = PlanetData::new(geometry, terrain, 0);

        let (verts, _) = MeshGen::build_chunk(
            ChunkKey {
                face: 0,
                u_idx: 0,
                v_idx: 0,
            },
            &planet,
            &block_content,
        );

        let vertex = verts
            .iter()
            .find(|vertex| vertex.color[0] > 0.0)
            .expect("generated chunk should contain colored vertices");
        assert!(vertex.color[1] >= vertex.color[0]);
        assert!(
            verts.iter().any(|vertex| vertex.texture_id >= 0),
            "generated chunk should contain textured block vertices"
        );
    }
}

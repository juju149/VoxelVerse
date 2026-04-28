use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use std::collections::HashSet;
use vv_core::{BlockId, ChunkKey, LodKey, CHUNK_SIZE};
use vv_planet::CoordSystem;
use vv_registry::BlockRenderSource;
use vv_world_runtime::{ChunkMods, PlanetData};

// --- Vertex format ----------------------------------------------------------

/// GPU-ready vertex: position, per-vertex colour, surface normal.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

// --- CPU mesh builder -------------------------------------------------------

pub struct MeshGen;

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
                if data.exists(id) {
                    Self::add_voxel(id, data, blocks, &mut verts, &mut inds, &mut idx);
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
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
    ) {
        let res = data.resolution;
        let Some(block_id) = data.block_at(id) else {
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
                let Some(neighbor_block) = data.block_at(neighbor) else {
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
            let ao_bl = Self::calculate_ao(n(-1, 0), n(0, -1), n(-1, -1));
            let ao_br = Self::calculate_ao(n(1, 0), n(0, -1), n(1, -1));
            let ao_tr = Self::calculate_ao(n(1, 0), n(0, 1), n(1, 1));
            let ao_tl = Self::calculate_ao(n(-1, 0), n(0, 1), n(-1, 1));
            Self::quad(
                verts,
                inds,
                idx,
                [o_bl, o_br, o_tr, o_tl],
                [apply(ao_bl), apply(ao_br), apply(ao_tr), apply(ao_tl)],
                true,
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
            );
        }
        let side_c = apply(0.8);
        let sc = [side_c, side_c, side_c, side_c];
        if !has_front {
            Self::quad(verts, inds, idx, [i_bl, i_br, o_br, o_bl], sc, false);
        }
        if !has_back {
            Self::quad(verts, inds, idx, [o_tl, o_tr, i_tr, i_tl], sc, false);
        }
        if !has_left {
            Self::quad(verts, inds, idx, [i_tl, i_bl, o_bl, o_tl], sc, false);
        }
        if !has_right {
            Self::quad(verts, inds, idx, [i_br, i_tr, o_tr, o_br], sc, false);
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

    fn quad(
        verts: &mut Vec<Vertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        pos: [Vec3; 4],
        colors: [[f32; 3]; 4],
        force_radial: bool,
    ) {
        let normal = if force_radial {
            ((pos[0] + pos[1] + pos[2] + pos[3]) * 0.25)
                .normalize()
                .to_array()
        } else {
            (pos[1] - pos[0])
                .cross(pos[2] - pos[0])
                .normalize()
                .to_array()
        };
        let base = *idx;
        for (p, c) in pos.iter().zip(colors.iter()) {
            verts.push(Vertex {
                pos: p.to_array(),
                color: *c,
                normal,
            });
        }
        inds.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        *idx += 4;
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
        let Some(terrain_render) = blocks.block_render(data.terrain_block) else {
            return (Vec::new(), Vec::new());
        };
        let terrain_color = terrain_render.color;
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let row_len = grid_res + 1;

        let get_sample_pos = |gx: i32, gy: i32| -> Vec3 {
            let step_u = (gx as i64 * key.size as i64) / grid_res as i64;
            let step_v = (gy as i64 * key.size as i64) / grid_res as i64;
            let abs_u = (key.x as i64 + step_u).clamp(0, data.resolution as i64) as u32;
            let abs_v = (key.y as i64 + step_v).clamp(0, data.resolution as i64) as u32;
            let h = data.terrain.get_height(key.face, abs_u, abs_v);
            CoordSystem::get_vertex_pos(key.face, abs_u, abs_v, h, data.resolution)
        };

        for vy in 0..=grid_res {
            for ux in 0..=grid_res {
                let pos = get_sample_pos(ux as i32, vy as i32);
                let p_right = get_sample_pos(ux as i32 + 1, vy as i32);
                let p_left = get_sample_pos(ux as i32 - 1, vy as i32);
                let p_down = get_sample_pos(ux as i32, vy as i32 + 1);
                let p_up = get_sample_pos(ux as i32, vy as i32 - 1);
                let tangent_u = p_right - p_left;
                let tangent_v = p_down - p_up;
                let mut normal = tangent_u.cross(tangent_v).normalize();
                if normal.dot(pos.normalize()) < 0.0 {
                    normal = -normal;
                }

                let slope = normal.dot(pos.normalize()).abs();
                let mut color = terrain_color;
                if slope < 0.85 {
                    color = [color[0] * 0.75, color[1] * 0.75, color[2] * 0.75];
                }

                verts.push(Vertex {
                    pos: pos.to_array(),
                    color,
                    normal: normal.to_array(),
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
        let radius = CoordSystem::get_layer_radius(data.resolution / 2, data.resolution);
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

        if let Some((center_id, _)) = CoordSystem::get_local_coords(player_pos, res) {
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
                                res,
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
                        let vi = |p: Vec3| Vertex {
                            pos: (center + (p - center) * sh).to_array(),
                            color,
                            normal,
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
            verts.push(Vertex {
                pos: [x, 0.0, z],
                color,
                normal: n,
            });
            verts.push(Vertex {
                pos: [x, height, z],
                color,
                normal: n,
            });
        }
        for i in 0..segments {
            let b1 = i * 2;
            let t1 = b1 + 1;
            let b2 = b1 + 2;
            let t2 = b1 + 3;
            inds.extend_from_slice(&[b1, t1, b2, b2, t1, t2]);
        }
        let ci = verts.len() as u32;
        verts.push(Vertex {
            pos: [0.0, height, 0.0],
            color,
            normal: [0.0, 1.0, 0.0],
        });
        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            verts.push(Vertex {
                pos: [theta.cos() * radius, height, theta.sin() * radius],
                color,
                normal: [0.0, 1.0, 0.0],
            });
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
                verts.push(Vertex {
                    pos: [xp * radius, yp * radius, zp * radius],
                    color,
                    normal: [xp, yp, zp],
                });
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
            Vertex {
                pos: [-s, 0.0, 0.0],
                color,
                normal,
            },
            Vertex {
                pos: [s, 0.0, 0.0],
                color,
                normal,
            },
            Vertex {
                pos: [0.0, -s, 0.0],
                color,
                normal,
            },
            Vertex {
                pos: [0.0, s, 0.0],
                color,
                normal,
            },
        ];
        (verts, vec![0, 1, 2, 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vv_config::WorldGenConfig;
    use vv_registry::{
        BlockContent, CompiledBlock, CompiledBlockMining, CompiledBlockPhysics,
        CompiledBlockRender, CompiledDrops, CompiledMaterialPhase, CompiledTextureLayout,
        ContentKey, RegistryTable,
    };
    use vv_world_gen::PlanetTerrain;

    #[test]
    fn chunk_mesh_uses_registry_block_render_color() {
        let mut blocks = RegistryTable::default();
        let block = blocks.push(
            ContentKey::new("test", "surface").unwrap(),
            CompiledBlock {
                display_key: None,
                stack_max: 64,
                tags: Vec::new(),
                mining: CompiledBlockMining {
                    hardness: 1.0,
                    tool_tier_min: 0,
                    drop_xp: 0,
                },
                physics: CompiledBlockPhysics {
                    phase: CompiledMaterialPhase::Solid,
                    density: 1.0,
                    friction: 1.0,
                    drag: 0.0,
                },
                render: CompiledBlockRender {
                    color: [0.3, 0.6, 0.9],
                    roughness: 1.0,
                    translucent: false,
                    emits_light: 0,
                    texture_layout: CompiledTextureLayout::Single,
                    model: None,
                },
                drops: CompiledDrops::None,
            },
        );
        let block_content = BlockContent::new(blocks);
        let resolution = 8;
        let terrain = PlanetTerrain::new(resolution, &WorldGenConfig::default());
        let planet = PlanetData::new(resolution, terrain, 0, block);

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
        assert!((vertex.color[1] / vertex.color[0] - 2.0).abs() < 0.001);
        assert!((vertex.color[2] / vertex.color[0] - 3.0).abs() < 0.001);
    }
}

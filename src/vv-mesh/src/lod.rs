use glam::Vec3;

use vv_planet::CoordSystem;
use vv_registry::{BlockRenderSource, CompiledBlockFace, CompiledRenderMode};
use vv_voxel::LodKey;

use crate::{MeshGen, Vertex};

impl MeshGen {
    pub fn generate_lod_mesh(
        key: LodKey,
        data: &vv_world_runtime::PlanetData,
        grid_res: u32,
        blocks: &impl BlockRenderSource,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::with_capacity(((grid_res + 1) * (grid_res + 1)) as usize);
        let mut inds = Vec::with_capacity((grid_res * grid_res * 6) as usize);
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
                    block_visual_id: block_id.max(0) as u32,
                    face_id: 0,
                    voxel_pos: [0, 0, 0],
                    variation_seed: block_id.max(0) as u32,
                    ao: color[0].max(color[1]).max(color[2]).clamp(0.0, 1.0),
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
                    block_visual_id: src_v.block_visual_id,
                    face_id: src_v.face_id,
                    voxel_pos: src_v.voxel_pos,
                    variation_seed: src_v.variation_seed,
                    ao: src_v.ao,
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
        data: &vv_world_runtime::PlanetData,
        blocks: &impl BlockRenderSource,
    ) -> (u32, [f32; 3], i32, i32) {
        let surface = data.terrain.get_height(face, u, v);

        for offset in 0..=8 {
            let layer = surface.saturating_sub(offset);
            let block = data.terrain.get_block(face, u, v, layer);

            let Some(render) = blocks.block_render(block) else {
                continue;
            };

            if matches!(
                render.render_mode,
                CompiledRenderMode::Transparent | CompiledRenderMode::Additive
            ) {
                continue;
            }

            let texture_id = Self::face_texture_id(render.texture_for_face(CompiledBlockFace::Top));

            let color = if texture_id >= 0 {
                [1.0, 1.0, 1.0]
            } else {
                render.color
            };

            return (layer, color, texture_id, block.raw() as i32);
        }

        let block = data.terrain.get_surface_block(face, u, v);

        let render = blocks.block_render(block);

        let texture_id = render
            .and_then(|render| render.texture_for_face(CompiledBlockFace::Top))
            .map(|id| id.raw() as i32)
            .unwrap_or(-1);

        let color = if texture_id >= 0 {
            [1.0, 1.0, 1.0]
        } else {
            render
                .map(|render| render.color)
                .unwrap_or([0.45, 0.70, 0.45])
        };

        (surface, color, texture_id, block.raw() as i32)
    }
}

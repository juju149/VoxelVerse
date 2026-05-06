use super::MeshGen;
use crate::content::TerrainPalette;
use crate::generation::CoordSystem;
use crate::rendering::Vertex;
use crate::voxel::LodKey;
use crate::world::PlanetData;

impl MeshGen {
    pub fn generate_lod_mesh(key: LodKey, data: &PlanetData) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();

        let grid_res = 64;
        let row_len = grid_res + 1;

        // calculate global pos for any grid index (even outside this chunk)
        // this allows us to "peek" into neighbor chunks for perfect normals.
        let get_sample_pos = |gx: i32, gy: i32| -> glam::Vec3 {
            let step_u = (gx as i64 * key.size as i64) / grid_res as i64;
            let step_v = (gy as i64 * key.size as i64) / grid_res as i64;

            // calculate absolute U/V
            let abs_u = (key.x as i64 + step_u).clamp(0, data.resolution as i64) as u32;
            let abs_v = (key.y as i64 + step_v).clamp(0, data.resolution as i64) as u32;

            let h = data.terrain.get_height(key.face, abs_u, abs_v);
            CoordSystem::get_vertex_pos(key.face, abs_u, abs_v, h, data.resolution)
        };

        // 1. Generate Vertices
        for vy in 0..=grid_res {
            for ux in 0..=grid_res {
                let pos = get_sample_pos(ux as i32, vy as i32);

                // seamless normal fix
                // instead of clamping to grid edges, we look -1 and +1 in global grid Space
                // this ensures the normal at the chunk edge matches the neighbor's normal perfectly

                let p_right = get_sample_pos(ux as i32 + 1, vy as i32);
                let p_left = get_sample_pos(ux as i32 - 1, vy as i32);
                let p_down = get_sample_pos(ux as i32, vy as i32 + 1);
                let p_up = get_sample_pos(ux as i32, vy as i32 - 1);

                // central Difference
                let tangent_u = p_right - p_left;
                let tangent_v = p_down - p_up;

                let mut normal = tangent_u.cross(tangent_v).normalize();
                if normal.dot(pos.normalize()) < 0.0 {
                    normal = -normal;
                }

                // --- COLORING ---
                let slope = normal.dot(pos.normalize()).abs();

                // recalculate h locally for core check
                let offset_u = (ux * key.size) / grid_res;
                let offset_v = (vy * key.size) / grid_res;
                let h = data.terrain.get_height(
                    key.face,
                    (key.x + offset_u).min(data.resolution),
                    (key.y + offset_v).min(data.resolution),
                );

                let is_core = data.has_core && h < data.profile.core_layers;

                let color = if is_core {
                    TerrainPalette::LOD_CORE
                } else {
                    // Look up the biome at this surface point and use its actual block colors.
                    // This makes deserts yellow, arctic ice white-blue, tundra grey, etc.
                    let bu = (key.x + offset_u).min(data.resolution.saturating_sub(1));
                    let bv = (key.y + offset_v).min(data.resolution.saturating_sub(1));
                    let biome_id = data.terrain.get_biome_id(key.face, bu, bv);
                    let biome = data.biomes.biome(biome_id);

                    if slope < 0.82 {
                        // Steep face → subsurface block color (stone, gravel, bare rock).
                        data.content.color(biome.subsurface_block)
                    } else {
                        // Gently sloped → surface block color (grass, snow, sand, ice…).
                        data.content.color(biome.surface_block)
                    }
                };

                verts.push(Vertex {
                    pos: pos.to_array(),
                    uv: [0.0, 0.0],
                    color,
                    normal: normal.to_array(),
                    tex_index: 0,
                });
            }
        }

        // generate indices
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

        // generate Skirts (hides physical gaps)
        let radius = data.profile.surface_radius;
        let chunk_phys_size = (key.size as f32 / data.resolution as f32) * radius;

        let skirt_depth = (chunk_phys_size * 0.15).clamp(4.0, 500.0);

        let mut add_skirt_edge = |coord_pairs: &[(u32, u32)], reverse: bool| {
            let base_idx = verts.len() as u32;
            for &(ux, vy) in coord_pairs {
                let src_idx = vy * row_len + ux;
                let src_v = verts[src_idx as usize];

                // bend skirt inwards slightly to avoid poking through other meshes
                let p = glam::Vec3::from_array(src_v.pos);
                let down = -p.normalize() * skirt_depth;

                verts.push(Vertex {
                    pos: (p + down).to_array(),
                    uv: [0.0, 0.0],
                    color: src_v.color,
                    normal: src_v.normal,
                    tex_index: 0,
                });
            }
            let len = coord_pairs.len() as u32;
            for i in 0..(len - 1) {
                let s1 = coord_pairs[i as usize].1 * row_len + coord_pairs[i as usize].0;
                let s2 =
                    coord_pairs[(i + 1) as usize].1 * row_len + coord_pairs[(i + 1) as usize].0;
                let k1 = base_idx + i;
                let k2 = base_idx + i + 1;

                // winding
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

        // define active edges positive logic
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
}

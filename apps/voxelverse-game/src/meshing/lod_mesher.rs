use super::{CpuMesh, CpuVertex, MeshGen};
use crate::content::TerrainPalette;
use crate::generation::CoordSystem;
use crate::voxel::LodKey;
use crate::world::PlanetData;

const LOD_GRID_RES: u32 = 64;

#[derive(Clone, Copy)]
struct LodSample {
    pos: glam::Vec3,
    u: u32,
    v: u32,
    height: u32,
}

impl MeshGen {
    pub fn generate_lod_mesh(key: LodKey, data: &PlanetData) -> CpuMesh {
        let row_len = LOD_GRID_RES + 1;
        let mut verts = Vec::with_capacity((row_len * row_len + row_len * 4) as usize);
        let mut inds =
            Vec::with_capacity(((LOD_GRID_RES * LOD_GRID_RES + LOD_GRID_RES * 4) * 6) as usize);

        let sample_side = (LOD_GRID_RES + 3) as usize;
        let samples = build_lod_samples(key, data, sample_side);

        let sample_at = |gx: u32, gy: u32| -> LodSample {
            samples[((gy + 1) as usize * sample_side) + (gx + 1) as usize]
        };
        let neighbor_at = |gx: i32, gy: i32| -> LodSample {
            samples[((gy + 1) as usize * sample_side) + (gx + 1) as usize]
        };

        for vy in 0..=LOD_GRID_RES {
            for ux in 0..=LOD_GRID_RES {
                let sample = sample_at(ux, vy);
                let pos = sample.pos;

                let tangent_u = neighbor_at(ux as i32 + 1, vy as i32).pos
                    - neighbor_at(ux as i32 - 1, vy as i32).pos;
                let tangent_v = neighbor_at(ux as i32, vy as i32 + 1).pos
                    - neighbor_at(ux as i32, vy as i32 - 1).pos;

                let radial = pos.normalize();
                let mut normal = tangent_u.cross(tangent_v).normalize();
                if normal.dot(radial) < 0.0 {
                    normal = -normal;
                }

                let slope = normal.dot(radial).abs();
                let color = lod_vertex_color(key, data, sample, slope);

                verts.push(CpuVertex {
                    pos: pos.to_array(),
                    uv: [0.0, 0.0],
                    color,
                    normal: normal.to_array(),
                    tex_index: 0,
                });
            }
        }

        for y in 0..LOD_GRID_RES {
            for x in 0..LOD_GRID_RES {
                let tl = y * row_len + x;
                let tr = tl + 1;
                let bl = (y + 1) * row_len + x;
                let br = bl + 1;

                inds.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
            }
        }

        let radius = data.profile.surface_radius;
        let chunk_phys_size = (key.size as f32 / data.resolution as f32) * radius;
        let skirt_depth = (chunk_phys_size * 0.15).clamp(4.0, 500.0);

        add_skirt_edge(&mut verts, &mut inds, row_len, 0, false, skirt_depth);
        add_skirt_edge(&mut verts, &mut inds, row_len, 1, true, skirt_depth);
        add_skirt_edge(&mut verts, &mut inds, row_len, 2, true, skirt_depth);
        add_skirt_edge(&mut verts, &mut inds, row_len, 3, false, skirt_depth);

        CpuMesh::new(verts, inds)
    }
}

fn build_lod_samples(key: LodKey, data: &PlanetData, sample_side: usize) -> Vec<LodSample> {
    let mut samples = Vec::with_capacity(sample_side * sample_side);
    for gy in -1..=(LOD_GRID_RES as i32 + 1) {
        for gx in -1..=(LOD_GRID_RES as i32 + 1) {
            let step_u = (gx as i64 * key.size as i64) / LOD_GRID_RES as i64;
            let step_v = (gy as i64 * key.size as i64) / LOD_GRID_RES as i64;
            let u = (key.x as i64 + step_u).clamp(0, data.resolution as i64) as u32;
            let v = (key.y as i64 + step_v).clamp(0, data.resolution as i64) as u32;
            let height = data.terrain.get_height(key.face, u, v);
            let pos = CoordSystem::get_vertex_pos(key.face, u, v, height, data.profile);
            samples.push(LodSample { pos, u, v, height });
        }
    }
    samples
}

fn lod_vertex_color(key: LodKey, data: &PlanetData, sample: LodSample, slope: f32) -> [f32; 3] {
    if data.has_core && sample.height < data.profile.core_layers {
        return TerrainPalette::LOD_CORE;
    }

    let u = sample.u.min(data.resolution.saturating_sub(1));
    let v = sample.v.min(data.resolution.saturating_sub(1));
    let biome = data
        .terrain
        .registry()
        .biome(data.terrain.get_biome_id(key.face, u, v) as usize);
    let (surface_block, subsurface_block) = data.terrain.lod_surface_blocks(key.face, u, v);
    let surface_color = data.content.color(surface_block);
    let subsurface_color = data.content.color(subsurface_block);

    let steepness = (1.0 - slope).clamp(0.0, 1.0);
    let height_norm = if data.profile.max_terrain_offset == 0 {
        0.0
    } else {
        ((sample.height as f32 - data.profile.surface_layer as f32)
            / data.profile.max_terrain_offset as f32)
            .clamp(0.0, 1.0)
    };

    let is_mountain = has_biome_tag(biome, "mountain");
    let is_frozen = has_biome_tag(biome, "frozen");
    let is_cold = has_biome_tag(biome, "cold");
    let is_dry = has_biome_tag(biome, "dry") || has_biome_tag(biome, "desert");

    // Gentle distant terrain should read as a biome patch, not only as a block
    // fallback color. The palette is authored in biome data and is therefore
    // the correct far-distance source for forests, meadows, cold slopes, etc.
    let biome_ground = if is_frozen {
        biome.color_tint.grass
    } else if is_dry {
        surface_color
    } else {
        mix(surface_color, biome.color_tint.grass, 0.55)
    };

    let mut color = mix(
        biome_ground,
        subsurface_color,
        smoothstep(0.16, 0.48, steepness),
    );

    if is_mountain {
        let rock = mix(subsurface_color, surface_color, 0.25);
        let snowline = if is_cold { 0.42 } else { 0.68 };
        let snow_amount = smoothstep(snowline, 0.95, height_norm) * (1.0 - steepness * 0.55);
        let rock_amount = smoothstep(0.28, 0.72, steepness);
        color = mix(color, rock, rock_amount);
        color = mix(color, surface_color, snow_amount);
    }

    // Tiny deterministic macro variation prevents planet-scale LOD tiles from
    // looking like flat paint while keeping transitions stable across rebuilds.
    let variation = 0.94 + hash01(key.face, u, v) * 0.12;
    scale_color(color, variation)
}

fn has_biome_tag(biome: &crate::content::CompiledProceduralBiome, needle: &str) -> bool {
    biome
        .vegetation_tags
        .iter()
        .chain(biome.fauna_tags.iter())
        .chain(std::iter::once(&biome.key))
        .any(|tag| tag.rsplit('/').next().is_some_and(|last| last == needle))
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0).max(0.0001)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn scale_color(color: [f32; 3], scale: f32) -> [f32; 3] {
    [
        (color[0] * scale).clamp(0.0, 1.0),
        (color[1] * scale).clamp(0.0, 1.0),
        (color[2] * scale).clamp(0.0, 1.0),
    ]
}

fn hash01(face: u8, u: u32, v: u32) -> f32 {
    let mut x = u
        .wrapping_mul(2_654_435_761)
        .wrapping_add(v.wrapping_mul(805_459_861))
        .wrapping_add((face as u32).wrapping_mul(97_531));
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

fn add_skirt_edge(
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    row_len: u32,
    edge: u8,
    reverse: bool,
    skirt_depth: f32,
) {
    let base_idx = verts.len() as u32;

    for i in 0..=LOD_GRID_RES {
        let (ux, vy) = edge_coord(edge, i);
        let src_v = verts[(vy * row_len + ux) as usize];
        let p = glam::Vec3::from_array(src_v.pos);
        let down = -p.normalize() * skirt_depth;

        verts.push(CpuVertex {
            pos: (p + down).to_array(),
            uv: [0.0, 0.0],
            color: src_v.color,
            normal: src_v.normal,
            tex_index: 0,
        });
    }

    for i in 0..LOD_GRID_RES {
        let (s1x, s1y) = edge_coord(edge, i);
        let (s2x, s2y) = edge_coord(edge, i + 1);
        let s1 = s1y * row_len + s1x;
        let s2 = s2y * row_len + s2x;
        let k1 = base_idx + i;
        let k2 = base_idx + i + 1;

        if reverse {
            inds.extend_from_slice(&[s1, k2, k1, s1, s2, k2]);
        } else {
            inds.extend_from_slice(&[s1, k1, k2, s1, k2, s2]);
        }
    }
}

fn edge_coord(edge: u8, i: u32) -> (u32, u32) {
    match edge {
        0 => (i, 0),
        1 => (i, LOD_GRID_RES),
        2 => (0, i),
        _ => (LOD_GRID_RES, i),
    }
}

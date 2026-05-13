//! Voxelized stair-step LOD mesher.
//!
//! Each LOD tile is divided into LOD_GRID_RES × LOD_GRID_RES "macro-voxels".
//! One macro-voxel covers `tile_size / LOD_GRID_RES` base voxels per side, so
//! the macro size doubles every LOD level (2, 4, 8, 16, 32… base voxels) — the
//! progression the engine needs to keep the cubic voxel aesthetic continuous
//! from the player all the way to the planet horizon.
//!
//! For each macro cell we emit:
//!   * a top quad at the cell's height (max of its four corner samples), and
//!   * up to four vertical side walls down to each neighbour's height (or a
//!     fixed skirt depth when the neighbour belongs to a different LOD tile).
//!
//! Vertex colours are baked from the same biome / surface block tables the
//! voxel mesher uses, so distant terrain reads as the same material seen up
//! close — no texture sampling is performed (the LOD mesh stays on the
//! `VERTEX_COLOR_MATERIAL_SENTINEL` material).

use super::{CpuMesh, CpuVertex, MeshGen, VERTEX_COLOR_MATERIAL_SENTINEL};
use vv_pack_compiler::TerrainPalette;
use vv_math::CoordSystem;
use vv_voxel::{LodKey, CHUNK_SIZE};
use vv_world::PlanetData;
use glam::Vec3;

/// Cells per side per LOD tile.  Equal to `CHUNK_SIZE` so the macro-voxel size
/// matches one base voxel chunk at the smallest LOD level and doubles cleanly
/// from there.
const LOD_GRID_RES: u32 = CHUNK_SIZE;

impl MeshGen {
    pub fn generate_lod_mesh(key: LodKey, data: &PlanetData) -> CpuMesh {
        let n = LOD_GRID_RES;
        // Base-voxel size of one macro cell.  `key.size` is the tile width in
        // base voxels; `step` divides it into `n` cells.  Clamped to at least 1
        // to keep things well-defined for tiles smaller than the grid (which
        // the quadtree never emits today but the guard costs nothing).
        let step = (key.size / n).max(1);
        let res = data.resolution;

        // Sample heights at the (n+1) × (n+1) cell corners.  Cells later read
        // four corners and take the max so the top sits on the upper envelope
        // of the heightfield — prevents the LOD from punching below real
        // terrain when ground rises steeply between samples.
        let corner_side = (n + 1) as usize;
        let mut h_corners = Vec::with_capacity(corner_side * corner_side);
        for cj in 0..=n {
            for ci in 0..=n {
                let u = (key.x + ci * step).min(res.saturating_sub(1));
                let v = (key.y + cj * step).min(res.saturating_sub(1));
                h_corners.push(data.terrain.get_height(key.face, u, v));
            }
        }
        let cell_h = |i: u32, j: u32| -> u32 {
            let c00 = h_corners[(j as usize) * corner_side + i as usize];
            let c10 = h_corners[(j as usize) * corner_side + (i + 1) as usize];
            let c01 = h_corners[((j + 1) as usize) * corner_side + i as usize];
            let c11 = h_corners[((j + 1) as usize) * corner_side + (i + 1) as usize];
            c00.max(c10).max(c01).max(c11)
        };

        let mut heights = vec![0u32; (n * n) as usize];
        for j in 0..n {
            for i in 0..n {
                heights[(j * n + i) as usize] = cell_h(i, j);
            }
        }

        // Skirt depth (layers) at tile boundary so neighbouring LOD tiles at
        // different resolutions never reveal a gap underneath.
        let radius = data.profile.surface_radius;
        let tile_phys = (key.size as f32 / res as f32) * radius;
        let skirt_layers = ((tile_phys * 0.10) / data.profile.layer_height.max(1e-3))
            .clamp(4.0, 800.0) as u32;

        let mut verts: Vec<CpuVertex> = Vec::with_capacity(((n * n) * 5 * 4) as usize / 4);
        let mut inds: Vec<u32> = Vec::with_capacity(((n * n) * 5 * 6) as usize / 4);

        for cj in 0..n {
            for ci in 0..n {
                let h = heights[(cj * n + ci) as usize];

                // Cell extent on the planet grid (in base voxels).  Clamped to
                // the planet resolution so edge cells fall back onto the same
                // boundary sample, matching what `cell_h` saw.
                let u0 = (key.x + ci * step).min(res);
                let u1 = (key.x + (ci + 1) * step).min(res);
                let v0 = (key.y + cj * step).min(res);
                let v1 = (key.y + (cj + 1) * step).min(res);

                // Mid-sample drives colour / biome lookup.  Clamped to a valid
                // grid index.
                let mid_u = (u0 + (u1 - u0) / 2).min(res.saturating_sub(1));
                let mid_v = (v0 + (v1 - v0) / 2).min(res.saturating_sub(1));

                let p_bl = CoordSystem::get_vertex_pos(key.face, u0, v0, h, data.profile);
                let p_br = CoordSystem::get_vertex_pos(key.face, u1, v0, h, data.profile);
                let p_tr = CoordSystem::get_vertex_pos(key.face, u1, v1, h, data.profile);
                let p_tl = CoordSystem::get_vertex_pos(key.face, u0, v1, h, data.profile);

                let cell_center = (p_bl + p_br + p_tr + p_tl) * 0.25;
                let radial = cell_center.normalize_or_zero();
                let slope = 1.0_f32; // top face is by construction radial-aligned at LOD scale
                let top_color = lod_surface_color(
                    key.face, mid_u, mid_v, h, slope, data,
                );
                let wall_color = lod_wall_color(key.face, mid_u, mid_v, h, data);

                push_quad(
                    &mut verts,
                    &mut inds,
                    [p_bl, p_br, p_tr, p_tl],
                    radial.to_array(),
                    top_color,
                );

                let mut emit_wall = |bl: Vec3, br: Vec3, tr: Vec3, tl: Vec3| {
                    let edge1 = br - bl;
                    let edge2 = tl - bl;
                    let mut nrm = edge1.cross(edge2).normalize_or_zero();
                    let outward_hint = ((bl + br + tr + tl) * 0.25) - cell_center;
                    if nrm.dot(outward_hint) < 0.0 {
                        nrm = -nrm;
                    }
                    push_quad(&mut verts, &mut inds, [bl, br, tr, tl], nrm.to_array(), wall_color);
                };

                // -U wall (at u = u0)
                let nh = if ci == 0 {
                    h.saturating_sub(skirt_layers)
                } else {
                    heights[(cj * n + ci - 1) as usize]
                };
                if nh < h {
                    let bl = CoordSystem::get_vertex_pos(key.face, u0, v0, nh, data.profile);
                    let br = CoordSystem::get_vertex_pos(key.face, u0, v1, nh, data.profile);
                    let tr = CoordSystem::get_vertex_pos(key.face, u0, v1, h, data.profile);
                    let tl = CoordSystem::get_vertex_pos(key.face, u0, v0, h, data.profile);
                    emit_wall(bl, br, tr, tl);
                }

                // +U wall (at u = u1)
                let nh = if ci == n - 1 {
                    h.saturating_sub(skirt_layers)
                } else {
                    heights[(cj * n + ci + 1) as usize]
                };
                if nh < h {
                    let bl = CoordSystem::get_vertex_pos(key.face, u1, v1, nh, data.profile);
                    let br = CoordSystem::get_vertex_pos(key.face, u1, v0, nh, data.profile);
                    let tr = CoordSystem::get_vertex_pos(key.face, u1, v0, h, data.profile);
                    let tl = CoordSystem::get_vertex_pos(key.face, u1, v1, h, data.profile);
                    emit_wall(bl, br, tr, tl);
                }

                // -V wall (at v = v0)
                let nh = if cj == 0 {
                    h.saturating_sub(skirt_layers)
                } else {
                    heights[((cj - 1) * n + ci) as usize]
                };
                if nh < h {
                    let bl = CoordSystem::get_vertex_pos(key.face, u1, v0, nh, data.profile);
                    let br = CoordSystem::get_vertex_pos(key.face, u0, v0, nh, data.profile);
                    let tr = CoordSystem::get_vertex_pos(key.face, u0, v0, h, data.profile);
                    let tl = CoordSystem::get_vertex_pos(key.face, u1, v0, h, data.profile);
                    emit_wall(bl, br, tr, tl);
                }

                // +V wall (at v = v1)
                let nh = if cj == n - 1 {
                    h.saturating_sub(skirt_layers)
                } else {
                    heights[((cj + 1) * n + ci) as usize]
                };
                if nh < h {
                    let bl = CoordSystem::get_vertex_pos(key.face, u0, v1, nh, data.profile);
                    let br = CoordSystem::get_vertex_pos(key.face, u1, v1, nh, data.profile);
                    let tr = CoordSystem::get_vertex_pos(key.face, u1, v1, h, data.profile);
                    let tl = CoordSystem::get_vertex_pos(key.face, u0, v1, h, data.profile);
                    emit_wall(bl, br, tr, tl);
                }
            }
        }

        CpuMesh::new(verts, inds)
    }
}

fn push_quad(
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    pos: [Vec3; 4],
    normal: [f32; 3],
    color: [f32; 3],
) {
    let base = verts.len() as u32;
    for p in pos {
        verts.push(CpuVertex {
            pos: p.to_array(),
            uv: [0.0, 0.0],
            color,
            normal,
            tex_index: VERTEX_COLOR_MATERIAL_SENTINEL,
        });
    }
    // Pipeline runs with `cull_mode: None`, so either winding lights correctly
    // (per-vertex normals carry the real orientation).  We use both windings to
    // make sure the quad is solid from either side.
    inds.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}

fn lod_surface_color(
    face: u8,
    u: u32,
    v: u32,
    height: u32,
    slope: f32,
    data: &PlanetData,
) -> [f32; 3] {
    if data.has_core && height < data.profile.core_layers {
        return TerrainPalette::LOD_CORE;
    }
    let res = data.resolution;
    let u = u.min(res.saturating_sub(1));
    let v = v.min(res.saturating_sub(1));
    let biome = data
        .terrain
        .registry()
        .biome(data.terrain.get_biome_id(face, u, v) as usize);
    let (surface_block, subsurface_block) = data.terrain.lod_surface_blocks(face, u, v);
    let surface_color = data.terrain_visuals.block_color(surface_block);
    let subsurface_color = data.terrain_visuals.block_color(subsurface_block);

    let steepness = (1.0 - slope).clamp(0.0, 1.0);
    let height_norm = if data.profile.max_terrain_offset == 0 {
        0.0
    } else {
        ((height as f32 - data.profile.surface_layer as f32)
            / data.profile.max_terrain_offset as f32)
            .clamp(0.0, 1.0)
    };

    let is_mountain = has_biome_tag(biome, "mountain");
    let is_frozen = has_biome_tag(biome, "frozen");
    let is_cold = has_biome_tag(biome, "cold");
    let is_dry = has_biome_tag(biome, "dry") || has_biome_tag(biome, "desert");

    let biome_ground = if is_frozen || is_dry {
        surface_color
    } else {
        mix(surface_color, biome.color_tint.grass, 0.22)
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

    let variation = 0.94 + hash01(face, u, v) * 0.12;
    scale_color(color, variation)
}

fn lod_wall_color(face: u8, u: u32, v: u32, height: u32, data: &PlanetData) -> [f32; 3] {
    if data.has_core && height < data.profile.core_layers {
        return TerrainPalette::LOD_CORE;
    }
    let res = data.resolution;
    let u = u.min(res.saturating_sub(1));
    let v = v.min(res.saturating_sub(1));
    let (_surface, subsurface) = data.terrain.lod_surface_blocks(face, u, v);
    let base = data.terrain_visuals.block_color(subsurface);
    // Walls are vertical → faked AO + slight darkening so steps read as cliffs
    // rather than blending into the top face.
    scale_color(base, 0.7)
}

fn has_biome_tag(biome: &vv_pack_compiler::CompiledProceduralBiome, needle: &str) -> bool {
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


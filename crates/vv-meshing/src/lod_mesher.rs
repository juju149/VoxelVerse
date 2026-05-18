//! Voxelized stair-step LOD mesher.
//!
//! Each LOD tile is divided into LOD_GRID_RES × LOD_GRID_RES "macro-voxels".
//! One macro-voxel covers `tile_size / LOD_GRID_RES` base voxels per side, so
//! the macro size doubles every LOD level (2, 4, 8, 16, 32... base voxels).
//!
//! The mesher receives a `LodMeshInput` with all heights and colors already
//! computed by the world layer.  No world references remain here.

use super::{CpuMesh, CpuVertex, MeshGen, VERTEX_COLOR_MATERIAL_SENTINEL};
use crate::lod_input::{LodCellColors, LodMeshInput};
use glam::Vec3;
use vv_math::CoordSystem;
use vv_voxel::CHUNK_SIZE;

const LOD_GRID_RES: u32 = CHUNK_SIZE;
/// Fraction of the top-face brightness applied to cliff walls.
/// A value well below 1.0 gives terrain clear silhouette depth at LOD ranges.
const LOD_WALL_SHADE_SCALE: f32 = 0.65;

impl MeshGen {
    pub fn generate_lod_mesh(input: &LodMeshInput) -> CpuMesh {
        let key = input.key;
        let n = LOD_GRID_RES;
        let step = (key.size / n).max(1);
        let grid_res = input.grid.resolution;

        let mut verts: Vec<CpuVertex> = Vec::with_capacity(((n * n) * 5 * 4) as usize / 4);
        let mut inds: Vec<u32> = Vec::with_capacity(((n * n) * 5 * 6) as usize / 4);

        let vp = |u: u32, v: u32, layer: u32| -> Vec3 {
            let u = u.min(grid_res.saturating_sub(1));
            let v = v.min(grid_res.saturating_sub(1));
            CoordSystem::get_vertex_pos(key.face, u, v, layer, input.grid)
        };

        for cj in 0..n {
            for ci in 0..n {
                let cell_idx = (cj * n + ci) as usize;
                let h = input.cell_heights[cell_idx];
                let LodCellColors {
                    top: top_color,
                    wall: wall_color,
                    ..
                } = input.cell_colors[cell_idx];

                let u0 = (key.x + ci * step).min(grid_res);
                let u1 = (key.x + (ci + 1) * step).min(grid_res);
                let v0 = (key.y + cj * step).min(grid_res);
                let v1 = (key.y + (cj + 1) * step).min(grid_res);

                let p_bl = vp(u0, v0, h);
                let p_br = vp(u1, v0, h);
                let p_tr = vp(u1, v1, h);
                let p_tl = vp(u0, v1, h);

                let cell_center = (p_bl + p_br + p_tr + p_tl) * 0.25;
                let radial = cell_center.normalize_or_zero();

                push_quad(
                    &mut verts,
                    &mut inds,
                    [p_bl, p_br, p_tr, p_tl],
                    radial.to_array(),
                    top_color,
                );

                let wall_c = scale_color(wall_color, LOD_WALL_SHADE_SCALE);

                let mut emit_wall = |bl: Vec3, br: Vec3, tr: Vec3, tl: Vec3| {
                    push_quad(
                        &mut verts,
                        &mut inds,
                        [bl, br, tr, tl],
                        radial.to_array(),
                        wall_c,
                    );
                };

                // -U wall
                let nh = if ci == 0 {
                    h.saturating_sub(input.skirt_layers)
                } else {
                    input.cell_heights[(cj * n + ci - 1) as usize]
                };
                if nh < h {
                    emit_wall(vp(u0, v0, nh), vp(u0, v1, nh), vp(u0, v1, h), vp(u0, v0, h));
                }

                // +U wall
                let nh = if ci == n - 1 {
                    h.saturating_sub(input.skirt_layers)
                } else {
                    input.cell_heights[(cj * n + ci + 1) as usize]
                };
                if nh < h {
                    emit_wall(vp(u1, v1, nh), vp(u1, v0, nh), vp(u1, v0, h), vp(u1, v1, h));
                }

                // -V wall
                let nh = if cj == 0 {
                    h.saturating_sub(input.skirt_layers)
                } else {
                    input.cell_heights[((cj - 1) * n + ci) as usize]
                };
                if nh < h {
                    emit_wall(vp(u1, v0, nh), vp(u0, v0, nh), vp(u0, v0, h), vp(u1, v0, h));
                }

                // +V wall
                let nh = if cj == n - 1 {
                    h.saturating_sub(input.skirt_layers)
                } else {
                    input.cell_heights[((cj + 1) * n + ci) as usize]
                };
                if nh < h {
                    emit_wall(vp(u0, v1, nh), vp(u1, v1, nh), vp(u1, v1, h), vp(u0, v1, h));
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
    inds.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}

fn scale_color(color: [f32; 3], scale: f32) -> [f32; 3] {
    [
        (color[0] * scale).clamp(0.0, 1.0),
        (color[1] * scale).clamp(0.0, 1.0),
        (color[2] * scale).clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::push_quad;
    use glam::Vec3;

    #[test]
    fn lod_quads_emit_one_front_facing_winding() {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        push_quad(
            &mut verts,
            &mut inds,
            [
                Vec3::new(0., 0., 0.),
                Vec3::new(1., 0., 0.),
                Vec3::new(1., 1., 0.),
                Vec3::new(0., 1., 0.),
            ],
            [0., 0., 1.],
            [1., 1., 1.],
        );
        assert_eq!(verts.len(), 4);
        assert_eq!(inds, [0, 1, 2, 2, 3, 0]);
    }
}

/// Scene builder: produces GPU-ready vertex/index buffers for viewer scenes.
/// Builds a standalone flat-world cube mesh without PlanetData.
/// Uses the same Vertex format as vv-mesh so the same shader pipeline works.
use glam::Vec3;
use vv_mesh::Vertex;
use vv_registry::BlockRenderSource;

use crate::args::Scene;

// Half-size of a voxel (0.5 m per voxel → each cube is 0.5 m wide)
const S: f32 = 0.5;

/// (face_id, normal, 4 corners as (pos_offset, uv))
/// face_id matches the game conventions: 0=top,1=bottom,2=north,3=south,4=east,5=west
const CUBE_FACES: &[(u32, [f32; 3], [([f32; 3], [f32; 2]); 4])] = &[
    // Top (Y+)  — CCW when viewed from above: normal = +Y
    (
        0,
        [0.0, 1.0, 0.0],
        [
            ([0.0, S, S], [0.0, 1.0]),
            ([S, S, S], [1.0, 1.0]),
            ([S, S, 0.0], [1.0, 0.0]),
            ([0.0, S, 0.0], [0.0, 0.0]),
        ],
    ),
    // Bottom (Y-)  — CCW when viewed from below: normal = -Y
    (
        1,
        [0.0, -1.0, 0.0],
        [
            ([0.0, 0.0, 0.0], [0.0, 0.0]),
            ([S, 0.0, 0.0], [1.0, 0.0]),
            ([S, 0.0, S], [1.0, 1.0]),
            ([0.0, 0.0, S], [0.0, 1.0]),
        ],
    ),
    // North (Z-)
    (
        2,
        [0.0, 0.0, -1.0],
        [
            ([S, 0.0, 0.0], [0.0, 0.0]),
            ([0.0, 0.0, 0.0], [1.0, 0.0]),
            ([0.0, S, 0.0], [1.0, 1.0]),
            ([S, S, 0.0], [0.0, 1.0]),
        ],
    ),
    // South (Z+)
    (
        3,
        [0.0, 0.0, 1.0],
        [
            ([0.0, 0.0, S], [0.0, 0.0]),
            ([S, 0.0, S], [1.0, 0.0]),
            ([S, S, S], [1.0, 1.0]),
            ([0.0, S, S], [0.0, 1.0]),
        ],
    ),
    // East (X+)
    (
        4,
        [1.0, 0.0, 0.0],
        [
            ([S, 0.0, S], [0.0, 0.0]),
            ([S, 0.0, 0.0], [1.0, 0.0]),
            ([S, S, 0.0], [1.0, 1.0]),
            ([S, S, S], [0.0, 1.0]),
        ],
    ),
    // West (X-)
    (
        5,
        [-1.0, 0.0, 0.0],
        [
            ([0.0, 0.0, 0.0], [0.0, 0.0]),
            ([0.0, 0.0, S], [1.0, 0.0]),
            ([0.0, S, S], [1.0, 1.0]),
            ([0.0, S, 0.0], [0.0, 1.0]),
        ],
    ),
];

/// AO computed from whether corner neighbours are present.
/// For a free-standing scene, we approximate AO using the neighbour mask.
fn ao_for_corner(neighbours_solid: u8) -> f32 {
    match neighbours_solid {
        0 => 1.0,
        1 => 0.84,
        2 => 0.72,
        _ => 0.55,
    }
}

/// Build a flat cube at integer grid position (gx, gz, gy).
/// gx/gz are horizontal positions, gy is vertical (y up).
/// We cull faces that are adjacent to another solid cube.
fn add_cube(
    gx: i32,
    gy: i32,
    gz: i32,
    block_id_raw: u32,
    block_visual_id: u32,
    render_color: [f32; 3],
    texture_id: i32,
    solid_mask: impl Fn(i32, i32, i32) -> bool,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
) {
    let origin = Vec3::new(gx as f32 * S, gy as f32 * S, gz as f32 * S);
    let variation_seed =
        (gx.wrapping_mul(73856093) ^ gy.wrapping_mul(19349663) ^ gz.wrapping_mul(83492791)) as u32;
    let voxel_pos = [gx, gz, gy];

    for &(face_id, normal, corners) in CUBE_FACES {
        let (nx, ny, nz) = match face_id {
            0 => (0, 1, 0),
            1 => (0, -1, 0),
            2 => (0, 0, -1),
            3 => (0, 0, 1),
            4 => (1, 0, 0),
            5 => (-1, 0, 0),
            _ => (0, 0, 0),
        };
        if solid_mask(gx + nx, gy + ny, gz + nz) {
            continue; // face is occluded
        }

        let base_idx = verts.len() as u32;
        for (off, uv) in corners {
            let pos = origin + Vec3::from(off);
            verts.push(Vertex {
                pos: pos.to_array(),
                color: render_color,
                normal,
                uv: uv,
                texture_id,
                block_id: block_id_raw as i32,
                block_visual_id,
                face_id,
                voxel_pos,
                variation_seed,
                ao: ao_for_corner(0),
            });
        }
        inds.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }
}

pub struct SceneMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    /// Number of blocks per axis (for camera framing)
    pub extent: u32,
}

/// Build a viewer scene from a list of block ids placed in a grid.
/// `scene_blocks` is a list of (grid_x, grid_y, grid_z, BlockId) tuples.
pub fn build_scene(
    scene_blocks: &[(i32, i32, i32, vv_registry::BlockId)],
    blocks: &impl BlockRenderSource,
) -> SceneMesh {
    let solid_positions: std::collections::HashSet<(i32, i32, i32)> =
        scene_blocks.iter().map(|&(x, y, z, _)| (x, y, z)).collect();

    let solid_mask = |x: i32, y: i32, z: i32| solid_positions.contains(&(x, y, z));

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for &(gx, gy, gz, block_id) in scene_blocks {
        let Some(render) = blocks.block_render(block_id) else {
            continue;
        };

        // Pick the main texture id (top face preferred)
        let texture_id = render
            .texture_for_face(vv_registry::CompiledBlockFace::Top)
            .map(|tid| tid.raw() as i32)
            .unwrap_or(-1);

        add_cube(
            gx,
            gy,
            gz,
            block_id.raw(),
            render.visual_id.raw(),
            render.color,
            texture_id,
            &solid_mask,
            &mut vertices,
            &mut indices,
        );
    }

    let max_coord = scene_blocks
        .iter()
        .flat_map(|&(x, y, z, _)| [x, y, z])
        .max()
        .map(|v| v as u32 + 1)
        .unwrap_or(1);

    SceneMesh {
        vertices,
        indices,
        extent: max_coord,
    }
}

/// Layout block positions for a given scene type.
/// All blocks have the same block_id.
pub fn layout_scene(
    scene: Scene,
    block_ids: &[(vv_registry::BlockId, String)],
) -> Vec<(i32, i32, i32, vv_registry::BlockId)> {
    use crate::args::Scene::*;

    if block_ids.is_empty() {
        return Vec::new();
    }

    match scene {
        Single => {
            let (id, _) = block_ids[0];
            vec![(0, 0, 0, id)]
        }
        Wall => {
            // 3×3 grid, all same block
            let (id, _) = block_ids[0];
            let mut result = Vec::new();
            for x in 0..3i32 {
                for z in 0..3i32 {
                    result.push((x, 0, z, id));
                }
            }
            result
        }
        Patch => {
            // 5×5 grid
            let (id, _) = block_ids[0];
            let mut result = Vec::new();
            for x in 0..5i32 {
                for z in 0..5i32 {
                    result.push((x, 0, z, id));
                }
            }
            result
        }
        Cube => {
            // 3×3×3 stack
            let (id, _) = block_ids[0];
            let mut result = Vec::new();
            for x in 0..3i32 {
                for y in 0..3i32 {
                    for z in 0..3i32 {
                        result.push((x, y, z, id));
                    }
                }
            }
            result
        }
        Stairs => {
            // 5-step ascending staircase (each column fills from y=0 to y=step).
            // Good for testing face lighting, edge separation, and shadow contrast.
            let (id, _) = block_ids[0];
            let mut result = Vec::new();
            for step in 0i32..5 {
                for fill_y in 0..=step {
                    result.push((step, fill_y, 0, id));
                }
            }
            result
        }
    }
}

/// Build a grid mesh for the floor plane.
pub fn build_grid(extent_world: f32) -> (Vec<Vertex>, Vec<u32>) {
    let half = (extent_world + 1.0).max(2.0);
    let step = S;
    let count = (half / step) as i32 + 2;
    let color = [0.3f32, 0.3, 0.3];

    let mut verts = Vec::new();
    let mut inds = Vec::new();
    let mut idx = 0u32;

    let emit_line = |x0: f32,
                     z0: f32,
                     x1: f32,
                     z1: f32,
                     verts: &mut Vec<Vertex>,
                     inds: &mut Vec<u32>,
                     idx: &mut u32| {
        let v = |p: [f32; 3]| Vertex {
            pos: p,
            color,
            normal: [0.0, 1.0, 0.0],
            uv: [0.0, 0.0],
            texture_id: -1,
            block_id: -1,
            block_visual_id: 0,
            face_id: 0,
            voxel_pos: [0, 0, 0],
            variation_seed: 0,
            ao: 1.0,
        };
        verts.push(v([x0, -0.001, z0]));
        verts.push(v([x1, -0.001, z1]));
        inds.push(*idx);
        inds.push(*idx + 1);
        *idx += 2;
    };

    let origin = -count as f32 * step;
    let end = count as f32 * step;

    let mut i = -count;
    while i <= count {
        let t = i as f32 * step;
        emit_line(t, origin, t, end, &mut verts, &mut inds, &mut idx);
        emit_line(origin, t, end, t, &mut verts, &mut inds, &mut idx);
        i += 1;
    }

    (verts, inds)
}

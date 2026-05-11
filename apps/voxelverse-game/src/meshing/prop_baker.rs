//! Prop geometry baker.
//!
//! Converts `PropStamp` instances (placed by `ProceduralPlanetTerrain::props_for_chunk`)
//! into tiny coloured-cube geometry appended directly to the terrain `CpuMesh`.
//! Props share the terrain render pipeline — no extra GPU pass.
//!
//! # Performance design
//! * **Zero allocations per stamp** — `VoxModel` stores a pre-baked visible-face
//!   list computed once at load time (`BakedFace`).  There is no `HashSet` or
//!   per-call neighbour lookup inside `bake_stamp`.
//! * **Sentinel material index 0xFFFF** — the shader detects this index and uses
//!   vertex colour directly (no texture fetch, no material buffer read).
//! * **sRGB colours in vertex buffer** — the shader applies gamma-expansion so
//!   prop colours are treated consistently with terrain vertex colours.

use super::{CpuMesh, CpuVertex, VERTEX_COLOR_MATERIAL_SENTINEL};
use crate::generation::{procedural::PropStamp, CoordSystem};
use crate::world::{PlanetProfile, VoxModel, VoxModelRegistry};

/// Maximum visible faces a single prop may contribute to a chunk mesh.
/// A grass blade has ~10-20, a mushroom ~50.  Hard-capping prevents a single
/// unusually-complex model from blowing up the budget.
const MAX_FACES_PER_STAMP: usize = 256;

/// Maximum total prop quads per chunk (all stamps combined).
/// Beyond this threshold additional stamps are skipped this frame.
/// At 4 vertices per quad × 512 quads = 2048 vertices worst case, easily
/// below the GPU vertex budget.
const MAX_QUADS_PER_CHUNK: usize = 2048;

/// Append prop geometry from `stamps` to `mesh`.
///
/// Stamps come from `ProceduralPlanetTerrain::props_for_chunk()`.
/// Only stamps whose `model_key` exists in `models` produce geometry.
pub fn bake_props(
    stamps: &[PropStamp],
    models: &VoxModelRegistry,
    profile: PlanetProfile,
    mesh: &mut CpuMesh,
) {
    if stamps.is_empty() {
        return;
    }
    let mut verts = std::mem::take(&mut mesh.vertices);
    let mut inds = std::mem::take(&mut mesh.indices);
    let mut idx = verts.len() as u32;
    let mut total_quads = 0usize;

    for stamp in stamps {
        if total_quads >= MAX_QUADS_PER_CHUNK {
            break;
        }
        let Some(model) = models.get(&stamp.model_key) else {
            continue;
        };
        let before = verts.len();
        bake_stamp(stamp, model, profile, &mut verts, &mut inds, &mut idx);
        total_quads += (verts.len() - before) / 4;
    }

    mesh.vertices = verts;
    mesh.indices = inds;
}

/// Bake one prop stamp into the mesh buffers.
/// Uses the model's pre-baked `BakedFace` list — zero allocations.
fn bake_stamp(
    stamp: &PropStamp,
    model: &VoxModel,
    profile: PlanetProfile,
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
) {
    if model.is_empty() || model.size_x == 0 || model.size_y == 0 || model.size_z == 0 {
        return;
    }

    // Scale: fit the model's XY footprint within one terrain-voxel cell.
    let inv_max_xy = 1.0_f32 / (model.size_x.max(model.size_y) as f32);
    let scale_z = inv_max_xy; // preserve aspect ratio

    let base_u = stamp.u as f32;
    let base_v = stamp.v as f32;
    let base_l = stamp.surface_layer as f32 + 1.0; // sit above the surface

    // Rotation helpers — quarter-turn around the radial (+Z in model space) axis.
    let cx = model.size_x as f32 * 0.5;
    let cy = model.size_y as f32 * 0.5;
    let rot = stamp.rotation & 3;

    // Precompute sin/cos for rotation.
    let (sin_r, cos_r) = match rot {
        0 => (0.0_f32, 1.0_f32),
        1 => (1.0, 0.0),
        2 => (0.0, -1.0),
        _ => (-1.0, 0.0),
    };

    // Inline rotation of model-(mx,my) around centre (cx,cy).
    let rotate = |mx: f32, my: f32| -> (f32, f32) {
        let dx = mx - cx;
        let dy = my - cy;
        (cx + cos_r * dx - sin_r * dy, cy + sin_r * dx + cos_r * dy)
    };

    for (faces_done, face) in model.faces.iter().enumerate() {
        if faces_done >= MAX_FACES_PER_STAMP {
            break;
        }

        // Project the 4 model-space corners to planet world-space.
        let mut corners = [glam::Vec3::ZERO; 4];
        for (i, &[mx, my, mz]) in face.corners.iter().enumerate() {
            let (rx, ry) = rotate(mx, my);
            let wu = base_u + rx * inv_max_xy;
            let wv = base_v + ry * inv_max_xy;
            let wl = base_l + mz * scale_z;
            corners[i] = CoordSystem::get_vertex_pos_f32(stamp.face, wu, wv, wl, profile);
        }

        // Compute face normal from cross product of two edges.
        let e0 = corners[1] - corners[0];
        let e1 = corners[3] - corners[0];
        let normal = e0.cross(e1).normalize_or_zero().to_array();

        let base_idx = *idx;
        let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        for (i, pos) in corners.iter().enumerate() {
            verts.push(CpuVertex {
                pos: pos.to_array(),
                uv: uvs[i],
                normal,
                // sRGB colour — shader applies pow(2.2) like terrain vertices.
                color: face.rgb,
                // Sentinel: shader skips texture lookup for this material index.
                tex_index: VERTEX_COLOR_MATERIAL_SENTINEL,
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
        *idx += 4;
    }
}

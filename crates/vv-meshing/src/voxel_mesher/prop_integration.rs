use std::sync::Arc;

use crate::cpu_mesh::{CpuMesh, CpuVertex};
use crate::VoxelMeshingConfig;
use crate::VERTEX_COLOR_MATERIAL_SENTINEL;
use vv_math::{CoordSystem, SphericalGrid};

/// Orientation of a prop relative to its anchor block.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PropSurfaceOrientation {
    #[default]
    Floor,
    Ceiling,
}

/// One pre-baked visible face of a prop model.
/// Corners are in model-local floating-point coordinates.
/// The mesher applies rotation + planet-space projection at bake time.
#[derive(Clone, Copy, Debug)]
pub struct BakedPropFace {
    /// 4 quad corners in model space (CCW winding).
    pub corners: [[f32; 3]; 4],
    /// Pre-darkened sRGB color.  Shader applies pow(2.2).
    pub rgb: [f32; 3],
}

/// Pre-baked prop model geometry.
///
/// The world layer builds this once per unique model when building
/// `ChunkMeshInput::prop_instances`.  The mesher never loads .vox files.
pub struct PropMeshModel {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    /// All visible faces, culled at load time.
    pub faces: Vec<BakedPropFace>,
}

impl PropMeshModel {
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty() || self.size_x == 0 || self.size_y == 0 || self.size_z == 0
    }
}

/// A single prop instance ready for the mesher.
pub struct PropMeshInstance {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    pub surface_layer: u32,
    /// Shared pre-baked model geometry.
    pub model: Arc<PropMeshModel>,
    /// Quarter-turn rotation around the radial axis (0–3).
    pub rotation: u8,
    pub orientation: PropSurfaceOrientation,
}

/// Append prop geometry from `instances` to `mesh`.
pub fn bake_prop_instances(
    instances: &[PropMeshInstance],
    grid: SphericalGrid,
    mesh: &mut CpuMesh,
    config: VoxelMeshingConfig,
) {
    if instances.is_empty() {
        return;
    }

    let mut verts = std::mem::take(&mut mesh.vertices);
    let mut inds = std::mem::take(&mut mesh.indices);
    let mut idx = verts.len() as u32;
    let mut total_quads = 0usize;

    for inst in instances {
        if total_quads >= config.max_prop_quads_per_chunk {
            break;
        }
        let before = verts.len();
        bake_one(inst, grid, config, &mut verts, &mut inds, &mut idx);
        total_quads += (verts.len() - before) / 4;
    }

    mesh.vertices = verts;
    mesh.indices = inds;
}

fn bake_one(
    inst: &PropMeshInstance,
    grid: SphericalGrid,
    config: VoxelMeshingConfig,
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
) {
    let model = &inst.model;
    if model.is_empty() {
        return;
    }

    let inv_max_xy = 1.0_f32 / (model.size_x.max(model.size_y) as f32);
    let scale_z = inv_max_xy;
    let base_u = inst.u as f32;
    let base_v = inst.v as f32;

    let (base_l, layer_dir) = match inst.orientation {
        PropSurfaceOrientation::Floor => (inst.surface_layer as f32 + 1.0, 1.0_f32),
        PropSurfaceOrientation::Ceiling => (inst.surface_layer as f32 - 1.0, -1.0_f32),
    };

    let cx = model.size_x as f32 * 0.5;
    let cy = model.size_y as f32 * 0.5;
    let rot = inst.rotation & 3;
    let (sin_r, cos_r) = match rot {
        0 => (0.0_f32, 1.0_f32),
        1 => (1.0, 0.0),
        2 => (0.0, -1.0),
        _ => (-1.0, 0.0),
    };
    let rotate = |mx: f32, my: f32| -> (f32, f32) {
        let dx = mx - cx;
        let dy = my - cy;
        (cx + cos_r * dx - sin_r * dy, cy + sin_r * dx + cos_r * dy)
    };

    for (faces_done, face) in model.faces.iter().enumerate() {
        if faces_done >= config.max_prop_faces_per_stamp {
            break;
        }

        let mut corners = [glam::Vec3::ZERO; 4];
        for (i, &[mx, my, mz]) in face.corners.iter().enumerate() {
            let (rx, ry) = rotate(mx, my);
            let wu = base_u + rx * inv_max_xy;
            let wv = base_v + ry * inv_max_xy;
            let wl = base_l + mz * scale_z * layer_dir;
            corners[i] = CoordSystem::get_vertex_pos_f32(inst.face, wu, wv, wl, grid);
        }

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
                color: face.rgb,
                tex_index: VERTEX_COLOR_MATERIAL_SENTINEL,
            });
        }
        inds.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx + 2,
            base_idx + 3,
            base_idx,
        ]);
        *idx += 4;
    }
}

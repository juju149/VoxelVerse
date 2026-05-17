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

use super::{CpuMesh, CpuVertex, VoxelMeshingConfig, VERTEX_COLOR_MATERIAL_SENTINEL};
use vv_math::CoordSystem;
use vv_world::{PlanetProfile, VoxModel, VoxModelRegistry};
use vv_worldgen::procedural::{PropOrientation, PropStamp};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PropBatchKey<'a> {
    pub model_key: &'a str,
    pub rotation: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct PropInstance {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    pub surface_layer: u32,
    pub orientation: PropOrientation,
}

#[derive(Clone, Debug)]
pub struct PropInstanceBatch<'a> {
    pub key: PropBatchKey<'a>,
    pub instances: Vec<PropInstance>,
}

struct PropBakeContext<'a> {
    model: &'a VoxModel,
    profile: PlanetProfile,
    config: VoxelMeshingConfig,
}

/// Group alive prop stamps by shared model/rotation.
/// The current renderer still bakes these batches into terrain meshes, but
/// this is the stable handoff shape for future GPU instancing.
pub fn collect_prop_instance_batches(stamps: &[PropStamp]) -> Vec<PropInstanceBatch<'_>> {
    let mut batches: Vec<PropInstanceBatch<'_>> = Vec::new();
    for stamp in stamps {
        let key = PropBatchKey {
            model_key: stamp.model_key.as_str(),
            rotation: stamp.rotation & 3,
        };
        let instance = PropInstance {
            face: stamp.face,
            u: stamp.u,
            v: stamp.v,
            surface_layer: stamp.surface_layer,
            orientation: stamp.orientation,
        };
        if let Some(batch) = batches.iter_mut().find(|batch| batch.key == key) {
            batch.instances.push(instance);
        } else {
            batches.push(PropInstanceBatch {
                key,
                instances: vec![instance],
            });
        }
    }
    batches
}

/// Append prop geometry from `stamps` to `mesh`.
///
/// Stamps come from `ProceduralPlanetTerrain::props_for_chunk()`.
/// Only stamps whose `model_key` exists in `models` produce geometry.
pub fn bake_props(
    stamps: &[PropStamp],
    models: &VoxModelRegistry,
    profile: PlanetProfile,
    mesh: &mut CpuMesh,
    config: VoxelMeshingConfig,
) {
    if stamps.is_empty() {
        return;
    }
    let mut verts = std::mem::take(&mut mesh.vertices);
    let mut inds = std::mem::take(&mut mesh.indices);
    let mut idx = verts.len() as u32;
    let mut total_quads = 0usize;

    for batch in collect_prop_instance_batches(stamps) {
        let Some(model) = models.get(batch.key.model_key) else {
            continue;
        };
        for instance in &batch.instances {
            if total_quads >= config.max_prop_quads_per_chunk {
                break;
            }
            let before = verts.len();
            bake_instance(
                instance,
                batch.key.rotation,
                PropBakeContext {
                    model,
                    profile,
                    config,
                },
                &mut verts,
                &mut inds,
                &mut idx,
            );
            total_quads += (verts.len() - before) / 4;
        }
    }

    mesh.vertices = verts;
    mesh.indices = inds;
}

/// Bake one prop stamp into the mesh buffers.
/// Uses the model's pre-baked `BakedFace` list — zero allocations.
fn bake_instance(
    instance: &PropInstance,
    rotation: u8,
    context: PropBakeContext<'_>,
    verts: &mut Vec<CpuVertex>,
    inds: &mut Vec<u32>,
    idx: &mut u32,
) {
    let model = context.model;
    if model.is_empty() || model.size_x == 0 || model.size_y == 0 || model.size_z == 0 {
        return;
    }

    // Scale: fit the model's XY footprint within one terrain-voxel cell.
    let inv_max_xy = 1.0_f32 / (model.size_x.max(model.size_y) as f32);
    let scale_z = inv_max_xy; // preserve aspect ratio

    let base_u = instance.u as f32;
    let base_v = instance.v as f32;
    // Floor: prop base sits one layer above the solid anchor.
    // Ceiling: prop base hangs one layer below the solid anchor (grows inward).
    let (base_l, layer_dir) = match instance.orientation {
        PropOrientation::Floor => (instance.surface_layer as f32 + 1.0, 1.0_f32),
        PropOrientation::Ceiling => (instance.surface_layer as f32 - 1.0, -1.0_f32),
    };

    // Rotation helpers — quarter-turn around the radial (+Z in model space) axis.
    let cx = model.size_x as f32 * 0.5;
    let cy = model.size_y as f32 * 0.5;
    let rot = rotation & 3;

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
        if faces_done >= context.config.max_prop_faces_per_stamp {
            break;
        }

        // Project the 4 model-space corners to planet world-space.
        let mut corners = [glam::Vec3::ZERO; 4];
        for (i, &[mx, my, mz]) in face.corners.iter().enumerate() {
            let (rx, ry) = rotate(mx, my);
            let wu = base_u + rx * inv_max_xy;
            let wv = base_v + ry * inv_max_xy;
            let wl = base_l + mz * scale_z * layer_dir;
            corners[i] =
                CoordSystem::get_vertex_pos_f32(instance.face, wu, wv, wl, context.profile);
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

#[cfg(test)]
mod tests {
    use super::collect_prop_instance_batches;
    use vv_worldgen::procedural::{PropOrientation, PropStamp};

    fn stamp(model_key: &str, rotation: u8, u: u32) -> PropStamp {
        PropStamp {
            face: 0,
            u,
            v: 2,
            surface_layer: 10,
            model_key: model_key.to_string(),
            rotation,
            orientation: PropOrientation::Floor,
        }
    }

    #[test]
    fn prop_instances_batch_by_model_and_rotation() {
        let stamps = vec![
            stamp("core:voxel/flower", 0, 1),
            stamp("core:voxel/flower", 0, 2),
            stamp("core:voxel/flower", 1, 3),
            stamp("core:voxel/rock", 0, 4),
        ];

        let batches = collect_prop_instance_batches(&stamps);

        assert_eq!(batches.len(), 3);
        assert!(batches
            .iter()
            .any(|batch| batch.key.model_key == "core:voxel/flower"
                && batch.key.rotation == 0
                && batch.instances.len() == 2));
    }
}

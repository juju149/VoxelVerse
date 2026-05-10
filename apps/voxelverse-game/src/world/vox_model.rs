//! MagicaVoxel .vox model loaded into a simple flat voxel list.
//!
//! This lives in `world/` so it can be stored inside `PlanetData` (which is
//! `Clone + Send`) and accessed from rayon mesh-worker threads without any
//! locking — the map is read-only after startup.

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ─── face direction constants ───────────────────────────────────────────────
// Used in BakedFace::dir to determine world-space brightness multiplier.
pub const FACE_TOP: u8 = 0; // +Z (outward, brightest)
pub const FACE_BTM: u8 = 1; // -Z (inward, darkest)
pub const FACE_PV: u8 = 2;  // +Y side
pub const FACE_NV: u8 = 3;  // -Y side
pub const FACE_PU: u8 = 4;  // +X side
pub const FACE_NU: u8 = 5;  // -X side

/// Vertex-colour brightness per face direction (sRGB, applied at bake time).
/// These encode a fake ambient-occlusion / diffuse cue that gives voxels depth
/// without requiring real-time per-voxel lighting.
pub const FACE_BRIGHTNESS: [f32; 6] = [
    1.00, // +Z top   (brightest, catches sky)
    0.38, // -Z bottom
    0.80, // +Y side
    0.80, // -Y side
    0.68, // +X side
    0.68, // -X side
];

/// One pre-baked visible face of a .vox model.
///
/// `corners[i]` = (model_x, model_y, model_z) of each quad corner in the
/// standard CCW winding used by the stamp baker.  Computed once at load;
/// re-used unchanged every time the model is stamped into a chunk mesh.
#[derive(Clone, Copy, Debug)]
pub struct BakedFace {
    /// 4 quad corners in model-space floating-point.
    /// The stamp baker applies rotation + scale and maps these to world space.
    pub corners: [[f32; 3]; 4],
    /// Pre-darkened sRGB colour (NOT linearized — the shader applies gamma).
    pub rgb: [f32; 3],
}

/// A loaded .vox model with pre-baked visible-face list.
///
/// After `load()`, every `BakedFace` represents exactly one quad that is
/// either on the model surface or adjacent to an empty cell.  Interior faces
/// are culled at load time, not at bake time.
#[derive(Clone, Debug)]
pub struct VoxModel {
    /// Pre-baked visible-face list. Zero allocs during chunk meshing.
    pub faces: Vec<BakedFace>,
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
}

impl VoxModel {
    /// Load the first model from a .vox file on disk.
    /// Returns `None` if the file is missing or malformed.
    pub fn load(path: &Path) -> Option<Self> {
        let path_str = path.to_str()?;
        let data = match dot_vox::load(path_str) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[prop] Failed to load {:?}: {}", path, e);
                return None;
            }
        };
        let model = data.models.into_iter().next()?;
        let palette = data.palette;

        let (sx, sy, sz) = (model.size.x, model.size.y, model.size.z);

        // Collect voxels with their palette colour.
        struct Raw { x: u8, y: u8, z: u8, r: f32, g: f32, b: f32 }
        let raw_voxels: Vec<Raw> = model.voxels.iter().map(|v| {
            let c = palette.get(v.i as usize).copied().unwrap_or(dot_vox::Color {
                r: 200, g: 200, b: 200, a: 255,
            });
            Raw {
                x: v.x, y: v.y, z: v.z,
                r: c.r as f32 / 255.0,
                g: c.g as f32 / 255.0,
                b: c.b as f32 / 255.0,
            }
        }).collect();

        // Neighbour lookup (built once, discarded after this fn).
        let filled: HashSet<(u8, u8, u8)> =
            raw_voxels.iter().map(|v| (v.x, v.y, v.z)).collect();
        let occ = |ix: i32, iy: i32, iz: i32| -> bool {
            if ix < 0 || iy < 0 || iz < 0 { return false; }
            filled.contains(&(ix as u8, iy as u8, iz as u8))
        };

        // Pre-bake all visible faces.
        let mut faces = Vec::with_capacity(raw_voxels.len() * 3);
        for v in &raw_voxels {
            let (x, y, z) = (v.x as f32, v.y as f32, v.z as f32);
            let (ix, iy, iz) = (v.x as i32, v.y as i32, v.z as i32);

            macro_rules! face {
                ($dir:expr, $c0:expr, $c1:expr, $c2:expr, $c3:expr) => {{
                    let b = FACE_BRIGHTNESS[$dir as usize];
                    faces.push(BakedFace {
                        corners: [$c0, $c1, $c2, $c3],
                        rgb: [v.r * b, v.g * b, v.b * b],
                    });
                }};
            }

            // +Z top
            if !occ(ix, iy, iz + 1) {
                face!(FACE_TOP,
                    [x,   y,   z+1.0],
                    [x+1.0, y,   z+1.0],
                    [x+1.0, y+1.0, z+1.0],
                    [x,   y+1.0, z+1.0]
                );
            }
            // -Z bottom
            if !occ(ix, iy, iz - 1) {
                face!(FACE_BTM,
                    [x,   y+1.0, z],
                    [x+1.0, y+1.0, z],
                    [x+1.0, y,   z],
                    [x,   y,   z]
                );
            }
            // +Y side (+V)
            if !occ(ix, iy + 1, iz) {
                face!(FACE_PV,
                    [x,   y+1.0, z],
                    [x+1.0, y+1.0, z],
                    [x+1.0, y+1.0, z+1.0],
                    [x,   y+1.0, z+1.0]
                );
            }
            // -Y side (-V)
            if !occ(ix, iy - 1, iz) {
                face!(FACE_NV,
                    [x+1.0, y,   z],
                    [x,   y,   z],
                    [x,   y,   z+1.0],
                    [x+1.0, y,   z+1.0]
                );
            }
            // +X side (+U)
            if !occ(ix + 1, iy, iz) {
                face!(FACE_PU,
                    [x+1.0, y,   z],
                    [x+1.0, y+1.0, z],
                    [x+1.0, y+1.0, z+1.0],
                    [x+1.0, y,   z+1.0]
                );
            }
            // -X side (-U)
            if !occ(ix - 1, iy, iz) {
                face!(FACE_NU,
                    [x,   y+1.0, z],
                    [x,   y,   z],
                    [x,   y,   z+1.0],
                    [x,   y+1.0, z+1.0]
                );
            }
        }

        faces.shrink_to_fit();
        Some(VoxModel { faces, size_x: sx, size_y: sy, size_z: sz })
    }

    /// True if this model has no visible geometry (fully surrounded or empty).
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}

/// Read-only registry of all loaded .vox models, keyed by content ref string.
///
/// Built once at startup; shared as `Arc<VoxModelRegistry>` across rayon workers.
#[derive(Clone, Debug, Default)]
pub struct VoxModelRegistry {
    pub models: HashMap<String, VoxModel>,
}

impl VoxModelRegistry {
    /// Load only the models listed in `needed_keys` from `asset_paths`.
    /// Skips assets not in `needed_keys` to avoid loading thousands of
    /// character / entity models that are never used by terrain props.
    pub fn load_all(
        pack_dir: &Path,
        asset_paths: &HashMap<String, String>,
        needed_keys: &HashSet<String>,
    ) -> Self {
        let mut models = HashMap::with_capacity(needed_keys.len());
        let mut missing = 0usize;
        for key in needed_keys {
            let Some(rel_path) = asset_paths.get(key) else {
                missing += 1;
                eprintln!("[prop] No asset path for model key '{key}'");
                continue;
            };
            let full_path: PathBuf = pack_dir.join(rel_path);
            if let Some(model) = VoxModel::load(&full_path) {
                models.insert(key.clone(), model);
            } else {
                missing += 1;
            }
        }
        println!(
            "[prop] Loaded {}/{} vox models ({} missing/empty)",
            models.len(),
            needed_keys.len(),
            missing
        );
        Self { models }
    }

    pub fn get(&self, key: &str) -> Option<&VoxModel> {
        self.models.get(key)
    }
}

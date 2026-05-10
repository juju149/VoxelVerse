//! Block model definitions: shared, reusable mesh + collision contracts.
//!
//! A block model defines *what shape* a block has — geometry, face layout for
//! materials, and collision volume. Multiple blocks can share the same model;
//! the compiler deduplicates them by content into `BlockModelId`.
//!
//! Face-layer convention is **local object axes**, not world directions, so
//! the same model can be reused across orientations (logs, planet curvature,
//! future block states):
//!
//! - `py` / `ny` — local +Y / -Y face
//! - `pz` / `nz` — local +Z / -Z face
//! - `px` / `nx` — local +X / -X face
//! - `end`       — both axial caps (cube_column)
//! - `side`      — the four lateral faces (cube_column)

use crate::ContentRef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockModelDef {
    pub format_version: u32,
    pub display_name: String,
    pub mesh: RawBlockMesh,
    pub collision: RawBlockCollisionShape,
}

/// The geometry kind of a block model. Each variant declares a closed set of
/// face-layer slot names that referencing blocks must populate exactly.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawBlockMesh {
    /// No geometry. Used by air-like blocks. Face layers: empty.
    None,

    /// A unit cube with six independent faces. The `face_layers` array
    /// names the slots — block materials must match these names exactly.
    /// Required length: 6. Canonical naming: `["py", "ny", "pz", "nz",
    /// "px", "nx"]` (local object axes).
    Cube {
        face_layers: Vec<String>,
        #[serde(default = "default_true")]
        ambient_occlusion: bool,
    },

    /// A vertical column with two distinct face groups: axial caps (`end`)
    /// and lateral faces (`side`). Default axis is local +Y; future
    /// `axis` block states will rotate the mapping. Required length: 2.
    /// Canonical naming: `["end", "side"]`.
    CubeColumn {
        face_layers: Vec<String>,
        #[serde(default = "default_true")]
        ambient_occlusion: bool,
    },
}

/// Collision volume of a block model. Sprint 0 keeps the four legacy kinds
/// for an isofunctional refactor; AABB / box-list arrive with slabs/stairs.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RawBlockCollisionShape {
    None,
    FullCube,
    SoftCube,
    LeafVolume,
}

impl RawBlockMesh {
    /// The face-layer slot names declared by this mesh, in canonical order.
    /// Block `materials` maps must contain *exactly* these keys — no more,
    /// no less.
    pub fn face_layers(&self) -> &[String] {
        match self {
            RawBlockMesh::None => &[],
            RawBlockMesh::Cube { face_layers, .. } => face_layers.as_slice(),
            RawBlockMesh::CubeColumn { face_layers, .. } => face_layers.as_slice(),
        }
    }

    /// Required length of the `face_layers` array for this mesh kind.
    /// Used by the compiler to enforce slot-count invariants.
    pub fn required_face_layer_count(&self) -> usize {
        match self {
            RawBlockMesh::None => 0,
            RawBlockMesh::Cube { .. } => 6,
            RawBlockMesh::CubeColumn { .. } => 2,
        }
    }

    /// A short stable tag used as part of `BlockModelId` hashing.
    pub fn kind_tag(&self) -> &'static str {
        match self {
            RawBlockMesh::None => "none",
            RawBlockMesh::Cube { .. } => "cube",
            RawBlockMesh::CubeColumn { .. } => "cube_column",
        }
    }

    /// Whether this mesh exposes ambient-occlusion as a model option.
    pub fn ambient_occlusion(&self) -> bool {
        match self {
            RawBlockMesh::Cube { ambient_occlusion, .. }
            | RawBlockMesh::CubeColumn { ambient_occlusion, .. } => *ambient_occlusion,
            RawBlockMesh::None => false,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Helper: convenient way for a block compiler to refer to a model by ID.
pub type BlockModelRef = ContentRef;

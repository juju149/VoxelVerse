/// Sentinel for geometry whose albedo is already baked into vertex color.
pub const MATERIAL_INDEX_MASK: u32 = 0x0000_FFFF;

const EDGE_MIN_U: u32 = 1 << 0;
const EDGE_MAX_U: u32 = 1 << 1;
const EDGE_MIN_V: u32 = 1 << 2;
const EDGE_MAX_V: u32 = 1 << 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FaceEdgeMask {
    pub min_u: bool,
    pub max_u: bool,
    pub min_v: bool,
    pub max_v: bool,
}

impl FaceEdgeMask {
    fn bits(self) -> u32 {
        (u32::from(self.min_u) * EDGE_MIN_U)
            | (u32::from(self.max_u) * EDGE_MAX_U)
            | (u32::from(self.min_v) * EDGE_MIN_V)
            | (u32::from(self.max_v) * EDGE_MAX_V)
    }
}

pub fn pack_material_edges(material_layer: u32, edges: FaceEdgeMask) -> u32 {
    debug_assert!(material_layer <= MATERIAL_INDEX_MASK);
    (material_layer & MATERIAL_INDEX_MASK) | (edges.bits() << 16)
}

/// How a voxel participates in blending and face-hiding.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VoxelMeshClass {
    #[default]
    None,
    Solid,
    Water,
}

/// How faces are generated for this voxel.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VoxelMeshKind {
    #[default]
    None,
    Cube,
    CubeColumn,
}

/// Texture layer indices per cube face.
#[derive(Clone, Copy, Debug, Default)]
pub struct VoxelVisualLayers {
    pub top: u32,
    pub bottom: u32,
    pub front: u32,
    pub back: u32,
    pub left: u32,
    pub right: u32,
}

/// Per-voxel visual data baked at content-compile time.
#[derive(Clone, Copy, Debug, Default)]
pub struct VoxelVisual {
    pub layers: VoxelVisualLayers,
    pub tint: [f32; 3],
}

/// All material data the mesher needs for one voxel type.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshMaterialEntry {
    pub is_renderable: bool,
    pub is_opaque_cube: bool,
    pub uses_greedy: bool,
    pub mesh_class: VoxelMeshClass,
    pub mesh_kind: VoxelMeshKind,
    pub visual: VoxelVisual,
    pub color: [f32; 3],
}

/// Lookup table indexed by `VoxelId::raw()`.
///
/// Built by the world layer from the compiled block registry before the job
/// is dispatched.  The mesher treats this as read-only.
pub struct MeshMaterialTable {
    entries: Vec<MeshMaterialEntry>,
}

impl MeshMaterialTable {
    pub fn new(entries: Vec<MeshMaterialEntry>) -> Self {
        Self { entries }
    }

    #[inline]
    fn get(&self, raw: u16) -> &MeshMaterialEntry {
        self.entries
            .get(raw as usize)
            .unwrap_or(self.entries.first().unwrap_or(&FALLBACK_ENTRY))
    }

    pub fn is_renderable(&self, id: vv_voxel::VoxelId) -> bool {
        self.get(id.raw()).is_renderable
    }

    pub fn is_opaque_cube(&self, id: vv_voxel::VoxelId) -> bool {
        self.get(id.raw()).is_opaque_cube
    }

    pub fn uses_greedy(&self, id: vv_voxel::VoxelId) -> bool {
        self.get(id.raw()).uses_greedy
    }

    pub fn mesh_class(&self, id: vv_voxel::VoxelId) -> VoxelMeshClass {
        self.get(id.raw()).mesh_class
    }

    pub fn mesh_kind(&self, id: vv_voxel::VoxelId) -> VoxelMeshKind {
        self.get(id.raw()).mesh_kind
    }

    pub fn visual(&self, id: vv_voxel::VoxelId) -> VoxelVisual {
        self.get(id.raw()).visual
    }

    pub fn color(&self, id: vv_voxel::VoxelId) -> [f32; 3] {
        self.get(id.raw()).color
    }

    pub fn hides_face_between(&self, current: vv_voxel::VoxelId, other: vv_voxel::VoxelId) -> bool {
        if self.is_opaque_cube(other) {
            return true;
        }
        current == other && self.mesh_class(current) == VoxelMeshClass::Water
    }
}

static FALLBACK_ENTRY: MeshMaterialEntry = MeshMaterialEntry {
    is_renderable: false,
    is_opaque_cube: false,
    uses_greedy: false,
    mesh_class: VoxelMeshClass::None,
    mesh_kind: VoxelMeshKind::None,
    visual: VoxelVisual {
        layers: VoxelVisualLayers {
            top: 0,
            bottom: 0,
            front: 0,
            back: 0,
            left: 0,
            right: 0,
        },
        tint: [1.0, 1.0, 1.0],
    },
    color: [1.0, 1.0, 1.0],
};

#[cfg(test)]
mod tests {
    use super::{pack_material_edges, FaceEdgeMask, MATERIAL_INDEX_MASK};

    #[test]
    fn material_layer_stays_in_low_bits() {
        let packed = pack_material_edges(
            42,
            FaceEdgeMask {
                min_u: true,
                max_u: false,
                min_v: false,
                max_v: false,
            },
        );
        assert_eq!(packed & MATERIAL_INDEX_MASK, 42);
    }

    #[test]
    fn edge_bits_land_in_high_word() {
        let packed = pack_material_edges(
            0,
            FaceEdgeMask {
                min_u: true,
                ..Default::default()
            },
        );
        assert_ne!(packed >> 16, 0);
        assert_eq!(packed & MATERIAL_INDEX_MASK, 0);
    }
}

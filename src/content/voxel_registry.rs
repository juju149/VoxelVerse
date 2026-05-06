use crate::voxel::VoxelId;

#[derive(Clone, Copy, Debug)]
pub struct VoxelDefinition {
    pub id: VoxelId,
    pub stable_key: &'static str,
    pub display_name: &'static str,
    pub solid: bool,
    pub color: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct VoxelRegistry {
    definitions: Vec<VoxelDefinition>,
    default_place_voxel: VoxelId,
}

impl VoxelRegistry {
    pub fn builtin() -> Self {
        let definitions = vec![
            VoxelDefinition {
                id: VoxelId::AIR,
                stable_key: "voxanet:air",
                display_name: "Air",
                solid: false,
                color: [0.0, 0.0, 0.0],
            },
            VoxelDefinition {
                id: VoxelId::CORE,
                stable_key: "voxanet:core",
                display_name: "Core Rock",
                solid: true,
                color: TerrainPalette::CORE,
            },
            VoxelDefinition {
                id: VoxelId::DIRT,
                stable_key: "voxanet:dirt",
                display_name: "Dirt",
                solid: true,
                color: TerrainPalette::DIRT,
            },
            VoxelDefinition {
                id: VoxelId::GRASS,
                stable_key: "voxanet:grass",
                display_name: "Grass",
                solid: true,
                color: TerrainPalette::GRASS,
            },
        ];

        Self::validate_definitions(&definitions);

        Self {
            definitions,
            default_place_voxel: VoxelId::DIRT,
        }
    }

    pub fn definition(&self, id: VoxelId) -> Option<&VoxelDefinition> {
        self.definitions
            .get(id.raw() as usize)
            .filter(|def| def.id == id)
            .or_else(|| self.definitions.iter().find(|def| def.id == id))
    }

    pub fn is_solid(&self, id: VoxelId) -> bool {
        self.definition(id).is_some_and(|def| def.solid)
    }

    pub fn color(&self, id: VoxelId) -> [f32; 3] {
        self.definition(id)
            .map(|def| def.color)
            .unwrap_or(TerrainPalette::DIRT)
    }

    pub fn default_place_voxel(&self) -> VoxelId {
        self.default_place_voxel
    }

    fn validate_definitions(definitions: &[VoxelDefinition]) {
        debug_assert!(definitions
            .iter()
            .all(|def| !def.stable_key.is_empty() && !def.display_name.is_empty()));
    }
}

use crate::content::TerrainPalette;

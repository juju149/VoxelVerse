use smallvec::SmallVec;

use crate::ContentKey;

use super::{BlockProceduralConfig, CompiledSurfaceProgram};

#[derive(Debug, Clone)]
pub struct CompiledBlockVisual {
    pub material_key: ContentKey,
    pub base_color: [f32; 4],
    pub palette: SmallVec<[[f32; 4]; 8]>,
    pub roughness: f32,
    pub metallic: f32,
    pub emission: Option<[f32; 4]>,
    pub alpha: f32,

    // Geometry authoring compiled from render.shape.
    pub bevel: f32,
    pub normal_strength: f32,

    pub variation: CompiledBlockVisualVariation,

    pub surface_program: CompiledSurfaceProgram,

    // Legacy bridge for shader layout. For now flat uses defaults.
    pub procedural: BlockProceduralConfig,

    pub faces: CompiledBlockFaceVisuals,

    // Empty for flat. Kept until procedural details are reintroduced cleanly.
    pub details: SmallVec<[CompiledBlockDetail; 8]>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledBlockVisualVariation {
    pub per_voxel_tint: f32,
    pub per_face_tint: f32,
    pub macro_noise_scale: f32,
    pub macro_noise_strength: f32,
    pub micro_noise_scale: f32,
    pub micro_noise_strength: f32,
    pub edge_darkening: f32,
    pub ao_influence: f32,
    pub biome_tint_strength: f32,
    pub wetness_response: f32,
    pub snow_response: f32,
    pub dust_response: f32,
}

#[derive(Debug, Clone, Default)]
pub struct CompiledBlockFaceVisuals {
    pub top: Option<CompiledBlockFaceVisual>,
    pub side: Option<CompiledBlockFaceVisual>,
    pub bottom: Option<CompiledBlockFaceVisual>,
    pub north: Option<CompiledBlockFaceVisual>,
    pub south: Option<CompiledBlockFaceVisual>,
    pub east: Option<CompiledBlockFaceVisual>,
    pub west: Option<CompiledBlockFaceVisual>,
}

#[derive(Debug, Clone)]
pub struct CompiledBlockFaceVisual {
    pub color_bias: [f32; 4],

    // Empty for flat.
    // Will later become typed detail/program masks instead of strings.
    pub detail_bias: SmallVec<[String; 4]>,
}

#[derive(Debug, Clone)]
pub struct CompiledBlockDetail {
    pub kind: String,
    pub density: f32,
    pub color: [f32; 4],
    pub min_size: f32,
    pub max_size: f32,
    pub slope_bias: f32,
}

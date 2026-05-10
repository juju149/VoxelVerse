mod block_family;
mod block_registry;
mod compiler;
mod content_index;
mod materials;
mod planet_registry;
mod procedural;
mod procedural_registry;
mod render_compiler;
mod render_registry;
mod texture_registry;

pub use block_family::{BlockStateValue, CompiledBlockFamily, MAX_VARIANTS_PER_FAMILY};
pub use block_registry::{
    BlockMaterialLayers, BlockModelId, BlockModelRegistry, BlockRegistry, CompiledBlock,
    CompiledBlockModel, CompiledBlockVisual, CompiledCollision, CompiledMesh, MaterialTextureSet,
};
pub use compiler::ContentCompiler;
pub use content_index::ContentIndex;
pub use materials::TerrainPalette;
pub use planet_registry::CompiledPlanet;
pub use procedural_registry::*;
pub use render_registry::{
    CompiledRenderContent, CompiledRenderGraph, CompiledRenderGraphPass, CompiledRenderProfile,
    CompiledRenderTechnique, CompiledShaderModule, MaterialFamilyId, RenderFeatureMask,
    RenderGraphId, RenderProfileId, RenderRegistry, RenderTechniqueId, ShaderContractId,
    ShaderModuleId, ShaderVariantId, ShaderVariantKey,
};
pub use texture_registry::{DecodedMaterialTextureSet, TextureRegistry};

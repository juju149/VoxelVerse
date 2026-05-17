//! Typed render pass and shader path declarations.
//!
//! The authored WGSL lives in `assets/packs/<namespace>/render/shaders`, but
//! the render graph, pass order and pipeline contracts are engine code. This
//! keeps the runtime deterministic and lets mods override shader source by
//! stable path without redefining bind groups or pipeline topology.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderPassId {
    ShadowDepth,
    Sky,
    Celestial,
    Clouds,
    TerrainOpaque,
    VolumetricFog,
    Precipitation,
    FinalComposite,
    Ui,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderPath {
    TerrainVertex,
    TerrainFragment,
    TerrainDepthVertex,
    SkyVertex,
    SkyFragment,
    CloudsVertex,
    CloudsFragment,
    VolumetricFogVertex,
    VolumetricFogFragment,
    PrecipitationVertex,
    PrecipitationFragment,
    CelestialVertex,
    CelestialFragment,
    WaterVertex,
    WaterFragment,
    FoliageVertex,
    FoliageFragment,
    FullscreenVertex,
    FinalCompositeFragment,
    FxaaFragment,
    BloomDownsampleFragment,
    BloomUpsampleFragment,
    UiVertex,
    UiFragment,
    DebugNormalsFragment,
    DebugDepthFragment,
    DebugLightingFragment,
}

impl ShaderPath {
    /// Shaders that have a compiled pipeline and are drawn every frame (or
    /// conditionally when their quality flag / atmosphere value is non-zero).
    /// These must compile without error before the renderer starts.
    pub const ACTIVE_IN_PASS: &'static [ShaderPath] = &[
        ShaderPath::TerrainVertex,
        ShaderPath::TerrainFragment,
        ShaderPath::TerrainDepthVertex,
        ShaderPath::SkyVertex,
        ShaderPath::SkyFragment,
        ShaderPath::CloudsVertex,
        ShaderPath::CloudsFragment,
        ShaderPath::VolumetricFogVertex,
        ShaderPath::VolumetricFogFragment,
        ShaderPath::PrecipitationVertex,
        ShaderPath::PrecipitationFragment,
        ShaderPath::CelestialVertex,
        ShaderPath::CelestialFragment,
        ShaderPath::FullscreenVertex,
        ShaderPath::FinalCompositeFragment,
        ShaderPath::UiVertex,
        ShaderPath::UiFragment,
    ];

    /// Shaders that are loaded and validated but whose pipelines are not yet
    /// wired into any render pass. Must be promoted to ACTIVE_IN_PASS before
    /// being used in production; their absence does not block startup.
    pub const OPTIONAL_FEATURE: &'static [ShaderPath] = &[
        ShaderPath::WaterVertex,
        ShaderPath::WaterFragment,
        ShaderPath::FoliageVertex,
        ShaderPath::FoliageFragment,
        ShaderPath::FxaaFragment,
        ShaderPath::BloomDownsampleFragment,
        ShaderPath::BloomUpsampleFragment,
    ];

    /// Shaders only active when a debug overlay is enabled at runtime.
    pub const DEBUG_ONLY: &'static [ShaderPath] = &[
        ShaderPath::DebugNormalsFragment,
        ShaderPath::DebugDepthFragment,
        ShaderPath::DebugLightingFragment,
    ];

    /// Union of all three categories. Used by ShaderLibrary::load to verify
    /// every shader file exists on disk at startup.
    pub const REQUIRED: &'static [ShaderPath] = &[
        ShaderPath::TerrainVertex,
        ShaderPath::TerrainFragment,
        ShaderPath::TerrainDepthVertex,
        ShaderPath::SkyVertex,
        ShaderPath::SkyFragment,
        ShaderPath::CloudsVertex,
        ShaderPath::CloudsFragment,
        ShaderPath::VolumetricFogVertex,
        ShaderPath::VolumetricFogFragment,
        ShaderPath::PrecipitationVertex,
        ShaderPath::PrecipitationFragment,
        ShaderPath::CelestialVertex,
        ShaderPath::CelestialFragment,
        ShaderPath::WaterVertex,
        ShaderPath::WaterFragment,
        ShaderPath::FoliageVertex,
        ShaderPath::FoliageFragment,
        ShaderPath::FullscreenVertex,
        ShaderPath::FinalCompositeFragment,
        ShaderPath::FxaaFragment,
        ShaderPath::BloomDownsampleFragment,
        ShaderPath::BloomUpsampleFragment,
        ShaderPath::UiVertex,
        ShaderPath::UiFragment,
        ShaderPath::DebugNormalsFragment,
        ShaderPath::DebugDepthFragment,
        ShaderPath::DebugLightingFragment,
    ];

    pub fn relative(self) -> &'static str {
        match self {
            ShaderPath::TerrainVertex => "passes/terrain/terrain.vert.wgsl",
            ShaderPath::TerrainFragment => "passes/terrain/terrain.frag.wgsl",
            ShaderPath::TerrainDepthVertex => "passes/terrain/terrain_depth.vert.wgsl",
            ShaderPath::SkyVertex => "passes/sky/sky.vert.wgsl",
            ShaderPath::SkyFragment => "passes/sky/sky.frag.wgsl",
            ShaderPath::CloudsVertex => "passes/clouds/clouds.vert.wgsl",
            ShaderPath::CloudsFragment => "passes/clouds/clouds.frag.wgsl",
            ShaderPath::VolumetricFogVertex => "passes/atmosphere/volumetric_fog.vert.wgsl",
            ShaderPath::VolumetricFogFragment => "passes/atmosphere/volumetric_fog.frag.wgsl",
            ShaderPath::PrecipitationVertex => "passes/precipitation/precipitation.vert.wgsl",
            ShaderPath::PrecipitationFragment => "passes/precipitation/precipitation.frag.wgsl",
            ShaderPath::CelestialVertex => "passes/celestial/celestial.vert.wgsl",
            ShaderPath::CelestialFragment => "passes/celestial/celestial.frag.wgsl",
            ShaderPath::WaterVertex => "passes/water/water.vert.wgsl",
            ShaderPath::WaterFragment => "passes/water/water.frag.wgsl",
            ShaderPath::FoliageVertex => "passes/foliage/foliage.vert.wgsl",
            ShaderPath::FoliageFragment => "passes/foliage/foliage.frag.wgsl",
            ShaderPath::FullscreenVertex => "passes/post/fullscreen.vert.wgsl",
            ShaderPath::FinalCompositeFragment => "passes/post/final_composite.frag.wgsl",
            ShaderPath::FxaaFragment => "passes/post/fxaa.frag.wgsl",
            ShaderPath::BloomDownsampleFragment => "passes/post/bloom_downsample.frag.wgsl",
            ShaderPath::BloomUpsampleFragment => "passes/post/bloom_upsample.frag.wgsl",
            ShaderPath::UiVertex => "passes/ui/ui.vert.wgsl",
            ShaderPath::UiFragment => "passes/ui/ui.frag.wgsl",
            ShaderPath::DebugNormalsFragment => "passes/debug/normals.frag.wgsl",
            ShaderPath::DebugDepthFragment => "passes/debug/depth.frag.wgsl",
            ShaderPath::DebugLightingFragment => "passes/debug/lighting.frag.wgsl",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ShaderPath;
    use std::path::Path;

    #[test]
    fn required_core_shaders_exist() {
        let core_pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        for shader in ShaderPath::REQUIRED {
            let path = core_pack
                .join("render")
                .join("shaders")
                .join(shader.relative());
            assert!(path.is_file(), "missing shader {}", path.display());
        }
    }

    /// Parse all ACTIVE_IN_PASS shaders with naga to catch WGSL errors at
    /// test-time rather than at runtime (wgpu device.create_shader_module panic).
    #[test]
    fn active_shaders_are_valid_wgsl() {
        let core_pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core");
        let stack = vec![vv_pack_compiler::shader::PackShaderRoot::new("core", core_pack)];
        let (lib, _report) = crate::shader::library::ShaderLibrary::load_stack(&stack)
            .expect("shader library loads");
        for &shader in ShaderPath::ACTIVE_IN_PASS {
            let source = lib.source(shader).expect("shader in library");
            naga::front::wgsl::parse_str(source)
                .unwrap_or_else(|e| panic!("WGSL parse failed in {}: {e}", shader.relative()));
        }
    }

    #[test]
    fn render_directory_has_no_ron_files() {
        let render_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs/core/render");
        let mut stack = vec![render_root];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("read render dir") {
                let path = entry.expect("dir entry").path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    assert_ne!(path.extension().and_then(|e| e.to_str()), Some("ron"));
                }
            }
        }
    }
}

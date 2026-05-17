//! Render-feature manifest schema.
//!
//! Lives under `defs/render/features/*.render_feature.ron` and
//! `defs/render/profiles/*.render_profile.ron`. The pack contract is:
//! WGSL code stays in `render/shaders/`, declarations stay in `defs/render/`.
//!
//! Every enum here is closed by design. A mod can author values from the
//! allow-lists below; anything else fails Pack Doctor before the renderer
//! ever sees the manifest. This is what lets `vv-render` keep ownership of
//! the engine topology while still allowing data-driven mod features.
//!
//! See section 4 of the shader architecture brief.

use serde::Deserialize;

// ── Top-level feature manifest ───────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRenderFeatureDef {
    pub format_version: u32,
    pub name: String,
    pub kind: RawRenderFeatureKind,
    pub slot: RawRenderFeatureSlot,
    /// Shader reference for the vertex stage. Format `<namespace>:shader/<rel>`,
    /// where `<rel>` is relative to the pack's `render/shaders/`.
    pub vertex: String,
    /// Optional fragment stage. Compute-only / vertex-only features may omit.
    #[serde(default)]
    pub fragment: Option<String>,
    pub entry_points: RawRenderFeatureEntryPoints,
    /// Bind groups the feature is allowed to consume. Closed allow-list.
    pub bind_groups: Vec<RawAllowedBindGroup>,
    pub target: RawRenderTargetKind,
    pub blend: RawBlendMode,
    pub depth: RawDepthMode,
    pub quality: RawRenderFeatureQuality,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRenderFeatureEntryPoints {
    pub vertex: String,
    #[serde(default)]
    pub fragment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRenderFeatureQuality {
    /// Lowest quality profile that may activate the feature.
    pub min_profile: RawRenderQualityClass,
    pub cost: RawRenderFeatureCost,
    /// When true, the runtime may silently drop the feature on a lower-tier
    /// profile rather than failing pipeline creation.
    #[serde(default)]
    pub can_disable_on_low: bool,
}

// ── Closed enums ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawRenderFeatureKind {
    TerrainMaterial,
    PostProcess,
    SkyLayer,
    WeatherLayer,
    WaterSurface,
    FoliageSurface,
    DebugView,
    UiEffect,
}

/// Pass-ordering slot inside the engine render schedule. Closed set: every
/// slot maps to a specific insertion point owned by `vv-render`.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawRenderFeatureSlot {
    AfterSkyBeforeTerrain,
    AfterSceneBeforePost,
    AfterPost,
    SkyLayerAfterGradient,
    WeatherLayerBeforeComposite,
    WaterSurface,
    FoliageSurface,
    TerrainOverlay,
    UiOverlay,
    DebugOverlay,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawAllowedBindGroup {
    Global,
    Local,
    MaterialAtlas,
    PostProcessInput,
    NoiseLut,
    Weather,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawRenderTargetKind {
    SceneHdr,
    SwapchainLdr,
    Depth,
    PostPingPong,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawDepthMode {
    None,
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawBlendMode {
    None,
    Alpha,
    Additive,
    Premultiplied,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RawRenderQualityClass {
    Low,
    High,
    Ultra,
    Cinematic,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawRenderFeatureCost {
    Cheap,
    Medium,
    Expensive,
}

// ── Render profile ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRenderProfileDef {
    pub format_version: u32,
    pub name: String,
    pub quality_class: RawRenderQualityClass,
    /// Identifiers of features (by manifest `name`) that this profile
    /// explicitly enables. Engine-managed passes are not listed here.
    #[serde(default)]
    pub enable_features: Vec<String>,
    /// Features the profile force-disables even if their `min_profile`
    /// would allow them. Useful for cinematic vs performance overrides.
    #[serde(default)]
    pub disable_features: Vec<String>,
}

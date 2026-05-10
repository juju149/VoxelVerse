use serde::{Deserialize, Deserializer, de};

/// Logical pack reference in `namespace:domain/path` form.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ContentRef(pub String);

impl<'de> Deserialize<'de> for ContentRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        validate_content_ref(&value).map_err(de::Error::custom)?;
        Ok(Self(value))
    }
}

pub type TextureRef = ContentRef;

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialTextureSet {
    pub albedo: TextureRef,
    pub normal: TextureRef,
    pub roughness: TextureRef,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RawBlockShape {
    None,
    #[default]
    Cube,
    CrossPlane,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RawRenderMode {
    Invisible,
    #[default]
    Opaque,
    AlphaTest,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockFaceMaterials {
    pub top: ContentRef,
    pub sides: ContentRef,
    pub bottom: ContentRef,
    #[serde(default)]
    pub front: Option<ContentRef>,
    #[serde(default)]
    pub back: Option<ContentRef>,
    #[serde(default)]
    pub left: Option<ContentRef>,
    #[serde(default)]
    pub right: Option<ContentRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawBlockMaterials {
    None,
    All(ContentRef),
    Faces(RawBlockFaceMaterials),
}

#[derive(Debug, Clone, Deserialize)]
pub enum RawMaterialTint {
    BiomeTint(String),
    Fixed([f32; 3]),
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawMaterialCategory {
    BlockSurface,
    Item,
    Prop,
    Creature,
    Ui,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawTextureSampling {
    PixelArtNearest,
    Linear,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawAuthoringDef {
    pub source: String,
    pub generated_by: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialDef {
    pub display_name: String,
    pub category: RawMaterialCategory,
    pub albedo: TextureRef,
    #[serde(default)]
    pub normal: Option<TextureRef>,
    #[serde(default)]
    pub roughness: Option<TextureRef>,
    #[serde(default)]
    pub tint: Option<RawMaterialTint>,
    pub render: RawRenderMode,
    pub sampling: RawTextureSampling,
    pub atlas: ContentRef,
    pub authoring: RawAuthoringDef,
}

fn validate_content_ref(value: &str) -> Result<(), String> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(format!("ref '{}' must use namespace:path form", value));
    };
    if namespace.is_empty() || path.is_empty() {
        return Err(format!("ref '{}' has empty namespace or path", value));
    }
    if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
        return Err(format!("ref '{}' must stay inside its pack", value));
    }
    Ok(())
}

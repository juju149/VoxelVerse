use serde::{de, Deserialize, Deserializer};

/// A logical reference to a texture resource.
/// Example: `"core:blocks/grass_top_albedo"` resolves to
/// `packs/core/textures/blocks/grass_top_albedo.png`.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct TextureRef(pub String);

impl<'de> Deserialize<'de> for TextureRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        validate_texture_ref(&value).map_err(de::Error::custom)?;
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMaterialTextureSet {
    pub albedo: TextureRef,
    pub normal: TextureRef,
    pub roughness: TextureRef,
}

/// Raw visual data for a block as written in pack files.
/// V0 supports a single PBR-lite material on the top face.
#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockVisual {
    pub top: RawMaterialTextureSet,
    #[serde(default = "default_tint")]
    pub tint: [f32; 3],
}

fn default_tint() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn validate_texture_ref(value: &str) -> Result<(), String> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(format!(
            "texture ref '{}' must use namespace:path form",
            value
        ));
    };
    if namespace.is_empty() || path.is_empty() {
        return Err(format!(
            "texture ref '{}' has empty namespace or path",
            value
        ));
    }
    if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
        return Err(format!(
            "texture ref '{}' must stay inside pack textures",
            value
        ));
    }
    Ok(())
}

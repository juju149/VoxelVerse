use crate::content::{BlockRegistry, MaterialTextureSet};
use png::{BitDepth, ColorType, Transformations};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct TextureImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct DecodedMaterialTextureSet {
    pub albedo: TextureImage,
    pub normal: TextureImage,
    pub roughness: TextureImage,
}

#[derive(Clone, Debug)]
pub struct TextureRegistry {
    tile_size: u32,
    materials: Vec<DecodedMaterialTextureSet>,
}

impl TextureRegistry {
    pub fn load(pack_root: &Path, blocks: &BlockRegistry) -> Result<Self, Vec<String>> {
        let mut errors = Vec::new();
        let mut materials = vec![Self::fallback_material(16)];
        let mut tile_size = 16;

        for material in blocks.material_sets() {
            match Self::load_material(pack_root, material) {
                Ok(decoded) => {
                    let size = decoded.albedo.width;
                    if decoded.albedo.width != decoded.albedo.height
                        || decoded.normal.width != size
                        || decoded.normal.height != size
                        || decoded.roughness.width != size
                        || decoded.roughness.height != size
                    {
                        errors.push(format!(
                            "Material textures must be square and same size: albedo='{}', normal='{}', roughness='{}'",
                            material.albedo, material.normal, material.roughness
                        ));
                    } else {
                        if materials.len() == 1 {
                            tile_size = size;
                            materials[0] = Self::fallback_material(tile_size);
                        } else if size != tile_size {
                            errors.push(format!(
                                "Material '{}' uses {}px tiles but registry already uses {}px tiles",
                                material.albedo, size, tile_size
                            ));
                        }
                        materials.push(decoded);
                    }
                }
                Err(err) => errors.push(err),
            }
        }

        if errors.is_empty() {
            Ok(Self {
                tile_size,
                materials,
            })
        } else {
            Err(errors)
        }
    }

    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    pub fn materials(&self) -> &[DecodedMaterialTextureSet] {
        &self.materials
    }

    fn load_material(
        pack_root: &Path,
        material: &MaterialTextureSet,
    ) -> Result<DecodedMaterialTextureSet, String> {
        Ok(DecodedMaterialTextureSet {
            albedo: Self::load_texture_ref(pack_root, &material.albedo)?,
            normal: Self::load_texture_ref(pack_root, &material.normal)?,
            roughness: Self::load_texture_ref(pack_root, &material.roughness)?,
        })
    }

    fn load_texture_ref(pack_root: &Path, texture_ref: &str) -> Result<TextureImage, String> {
        let path = Self::resolve_texture_path(pack_root, texture_ref)?;
        Self::decode_png(&path).map_err(|err| format!("{}: {}", path.display(), err))
    }

    fn resolve_texture_path(pack_root: &Path, texture_ref: &str) -> Result<PathBuf, String> {
        let (namespace, path) = texture_ref
            .split_once(':')
            .ok_or_else(|| format!("Texture ref '{}' must use namespace:path form", texture_ref))?;
        if path.contains("..") || path.starts_with('/') || path.starts_with('\\') {
            return Err(format!(
                "Texture ref '{}' must stay inside pack textures",
                texture_ref
            ));
        }
        Ok(pack_root
            .join(namespace)
            .join("textures")
            .join(path)
            .with_extension("png"))
    }

    fn decode_png(path: &Path) -> Result<TextureImage, String> {
        let file = File::open(path).map_err(|err| format!("missing or unreadable PNG ({err})"))?;
        let mut decoder = png::Decoder::new(BufReader::new(file));
        decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);

        let mut reader = decoder
            .read_info()
            .map_err(|err| format!("invalid PNG header ({err})"))?;
        let mut buffer = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut buffer)
            .map_err(|err| format!("invalid PNG data ({err})"))?;
        let bytes = &buffer[..info.buffer_size()];

        if info.bit_depth != BitDepth::Eight {
            return Err(format!(
                "PNG must decode to 8-bit channels, got {:?}",
                info.bit_depth
            ));
        }

        let rgba = match info.color_type {
            ColorType::Rgba => bytes.to_vec(),
            ColorType::Rgb => {
                let mut out = Vec::with_capacity((info.width * info.height * 4) as usize);
                for px in bytes.chunks_exact(3) {
                    out.extend_from_slice(&[px[0], px[1], px[2], 255]);
                }
                out
            }
            ColorType::Grayscale => {
                let mut out = Vec::with_capacity((info.width * info.height * 4) as usize);
                for &v in bytes {
                    out.extend_from_slice(&[v, v, v, 255]);
                }
                out
            }
            ColorType::GrayscaleAlpha => {
                let mut out = Vec::with_capacity((info.width * info.height * 4) as usize);
                for px in bytes.chunks_exact(2) {
                    out.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
                }
                out
            }
            ColorType::Indexed => {
                return Err("indexed PNG was not expanded by decoder".to_string());
            }
        };

        Ok(TextureImage {
            width: info.width,
            height: info.height,
            rgba,
        })
    }

    fn fallback_material(tile_size: u32) -> DecodedMaterialTextureSet {
        let texels = (tile_size * tile_size) as usize;
        DecodedMaterialTextureSet {
            albedo: TextureImage {
                width: tile_size,
                height: tile_size,
                rgba: [255, 255, 255, 255].repeat(texels),
            },
            normal: TextureImage {
                width: tile_size,
                height: tile_size,
                rgba: [128, 128, 255, 255].repeat(texels),
            },
            roughness: TextureImage {
                width: tile_size,
                height: tile_size,
                rgba: [210, 210, 210, 255].repeat(texels),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TextureRegistry;
    use crate::content::{compile::ContentCompiler, pack::PackLoader};
    use std::path::Path;

    #[test]
    fn core_pack_texture_registry_loads_grass_top_material() {
        let pack = PackLoader::load_from_dir(Path::new("packs/core")).expect("core pack");
        let blocks = ContentCompiler::compile_blocks(pack.blocks).expect("blocks");
        let textures = TextureRegistry::load(Path::new("packs"), &blocks).expect("textures");

        assert_eq!(textures.materials().len(), 10);
        assert_eq!(textures.tile_size(), 16);
    }

    #[test]
    fn missing_texture_ref_reports_path() {
        let err =
            TextureRegistry::load_texture_ref(Path::new("packs"), "core:blocks/does_not_exist")
                .expect_err("missing texture must error");

        assert!(err.contains("does_not_exist.png"));
        assert!(err.contains("missing or unreadable PNG"));
    }
}

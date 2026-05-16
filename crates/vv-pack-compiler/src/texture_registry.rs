use crate::{BlockRegistry, MaterialTextureSet};
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
        let material_sets = blocks.material_sets();

        if material_sets.is_empty() {
            let tile_size = 16;
            return Ok(Self {
                tile_size,
                materials: vec![Self::fallback_material(tile_size)],
            });
        }

        // First pass: decode every material, replacing missing/invalid ones with
        // None (they will become the fallback white tile).  We never panic on
        // missing textures — content errors are warned about and the slot is
        // kept alive so that all BlockMaterialLayers indices remain valid.
        let mut decoded: Vec<Option<DecodedMaterialTextureSet>> =
            Vec::with_capacity(material_sets.len());
        let mut warn_count = 0usize;

        for material in material_sets {
            match Self::load_material(pack_root, material) {
                Ok(mat) => {
                    let size = mat.albedo.width;
                    let ok = mat.albedo.height == size
                        && mat.normal.width == size
                        && mat.normal.height == size
                        && mat.roughness.width == size
                        && mat.roughness.height == size;
                    if ok {
                        decoded.push(Some(mat));
                    } else {
                        eprintln!(
                            "[texture] size mismatch for '{}', using fallback",
                            material.albedo
                        );
                        decoded.push(None);
                        warn_count += 1;
                    }
                }
                Err(err) => {
                    eprintln!("[texture] missing: {}", err);
                    decoded.push(None);
                    warn_count += 1;
                }
            }
        }

        if warn_count > 0 {
            eprintln!(
                "[texture] {} material(s) missing or invalid — falling back to white",
                warn_count
            );
        }

        let tile_size = decoded
            .iter()
            .flatten()
            .map(|m| m.albedo.width)
            .max()
            .unwrap_or(16);

        let fallback = Self::fallback_material(tile_size);
        // Index 0 is always the fallback; indices 1..N match material_sets[0..N-1].
        let mut materials = Vec::with_capacity(decoded.len() + 1);
        materials.push(fallback.clone());
        for slot in decoded {
            let mat = match slot {
                Some(m) => Self::resize_material_nearest(m, tile_size),
                None => fallback.clone(),
            };
            materials.push(mat);
        }

        Ok(Self {
            tile_size,
            materials,
        })
    }

    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    pub fn materials(&self) -> &[DecodedMaterialTextureSet] {
        &self.materials
    }

    pub fn average_albedo_color(&self, layer: u32) -> [f32; 3] {
        let Some(material) = self.materials.get(layer as usize) else {
            return [1.0, 1.0, 1.0];
        };
        average_srgb_image_linear(&material.albedo)
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
        let path = path.strip_prefix("texture/").unwrap_or(path);
        Ok(pack_root
            .join(namespace)
            .join("media")
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

    fn resize_material_nearest(
        material: DecodedMaterialTextureSet,
        tile_size: u32,
    ) -> DecodedMaterialTextureSet {
        DecodedMaterialTextureSet {
            albedo: Self::resize_texture_nearest(material.albedo, tile_size),
            normal: Self::resize_texture_nearest(material.normal, tile_size),
            roughness: Self::resize_texture_nearest(material.roughness, tile_size),
        }
    }

    fn resize_texture_nearest(image: TextureImage, tile_size: u32) -> TextureImage {
        if image.width == tile_size && image.height == tile_size {
            return image;
        }

        let mut rgba = vec![0; (tile_size * tile_size * 4) as usize];
        for y in 0..tile_size {
            let src_y = y * image.height / tile_size;
            for x in 0..tile_size {
                let src_x = x * image.width / tile_size;
                let src = ((src_y * image.width + src_x) * 4) as usize;
                let dst = ((y * tile_size + x) * 4) as usize;
                rgba[dst..dst + 4].copy_from_slice(&image.rgba[src..src + 4]);
            }
        }

        TextureImage {
            width: tile_size,
            height: tile_size,
            rgba,
        }
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

fn average_srgb_image_linear(image: &TextureImage) -> [f32; 3] {
    let mut sum = [0.0_f32; 3];
    let mut weight_sum = 0.0_f32;
    for px in image.rgba.chunks_exact(4) {
        let alpha = px[3] as f32 / 255.0;
        if alpha <= 0.0 {
            continue;
        }
        sum[0] += srgb_u8_to_linear(px[0]) * alpha;
        sum[1] += srgb_u8_to_linear(px[1]) * alpha;
        sum[2] += srgb_u8_to_linear(px[2]) * alpha;
        weight_sum += alpha;
    }

    if weight_sum <= 0.0 {
        [1.0, 1.0, 1.0]
    } else {
        [
            sum[0] / weight_sum,
            sum[1] / weight_sum,
            sum[2] / weight_sum,
        ]
    }
}

fn srgb_u8_to_linear(value: u8) -> f32 {
    let c = value as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::TextureRegistry;
    use std::path::Path;
    use vv_pack_loader::PackLoader;

    #[test]
    fn core_pack_texture_registry_loads_grass_top_material() {
        let pack_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs");
        let pack = PackLoader::load_from_dir(&pack_root.join("core")).expect("core pack");
        let objects = crate::object_compiler::compile_objects(pack.objects).expect("objects");
        let textures = TextureRegistry::load(&pack_root, &objects.blocks).expect("textures");

        assert!(textures.materials().len() > 10);
        assert!(textures.tile_size() >= 128);
    }

    #[test]
    fn average_albedo_color_reads_decoded_material_texture() {
        let pack_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs");
        let pack = PackLoader::load_from_dir(&pack_root.join("core")).expect("core pack");
        let objects = crate::object_compiler::compile_objects(pack.objects).expect("objects");
        let textures = TextureRegistry::load(&pack_root, &objects.blocks).expect("textures");

        let grass = objects
            .blocks
            .lookup_default("core:object/terrain/grass")
            .expect("grass block");
        let grass_top = objects.blocks.visual(grass).layers.top;
        let color = textures.average_albedo_color(grass_top);

        assert!(color[1] > color[0], "grass should stay visibly green");
        assert!(color[1] > color[2], "grass should stay visibly green");
    }

    #[test]
    fn missing_texture_ref_reports_path() {
        let err = TextureRegistry::load_texture_ref(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/packs"),
            "core:blocks/does_not_exist",
        )
        .expect_err("missing texture must error");

        assert!(err.contains("does_not_exist.png"));
        assert!(err.contains("missing or unreadable PNG"));
    }
}

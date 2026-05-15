use vv_pack_compiler::{DecodedMaterialTextureSet, TextureRegistry};

pub(crate) struct TextureAtlas {
    pub albedo_view: wgpu::TextureView,
    pub normal_view: wgpu::TextureView,
    pub roughness_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureAtlas {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        registry: &TextureRegistry,
    ) -> Self {
        let tile_size = registry.tile_size();
        let layer_count = registry.materials().len() as u32;

        let albedo = Self::create_array_texture(
            device,
            queue,
            "Material Albedo",
            wgpu::TextureFormat::Rgba8UnormSrgb,
            tile_size,
            registry.materials(),
            |m| &m.albedo.rgba,
        );
        let normal = Self::create_array_texture(
            device,
            queue,
            "Material Normal",
            wgpu::TextureFormat::Rgba8Unorm,
            tile_size,
            registry.materials(),
            |m| &m.normal.rgba,
        );
        let roughness = Self::create_array_texture(
            device,
            queue,
            "Material Roughness",
            wgpu::TextureFormat::Rgba8Unorm,
            tile_size,
            registry.materials(),
            |m| &m.roughness.rgba,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Material Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        debug_assert_eq!(layer_count, registry.materials().len() as u32);
        Self {
            albedo_view: albedo,
            normal_view: normal,
            roughness_view: roughness,
            sampler,
        }
    }

    fn create_array_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        format: wgpu::TextureFormat,
        tile_size: u32,
        materials: &[DecodedMaterialTextureSet],
        select: fn(&DecodedMaterialTextureSet) -> &[u8],
    ) -> wgpu::TextureView {
        let mip_level_count = Self::mip_level_count(tile_size);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: materials.len() as u32,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (layer, material) in materials.iter().enumerate() {
            let mut mip_data = select(material).to_vec();
            let mut mip_size = tile_size;

            for mip_level in 0..mip_level_count {
                let (upload_data, bytes_per_row) = Self::pad_rgba_rows(&mip_data, mip_size);
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &upload_data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: Some(mip_size),
                    },
                    wgpu::Extent3d {
                        width: mip_size,
                        height: mip_size,
                        depth_or_array_layers: 1,
                    },
                );

                if mip_level + 1 < mip_level_count {
                    mip_data = Self::downsample_rgba8(&mip_data, mip_size, mip_size);
                    mip_size = (mip_size / 2).max(1);
                }
            }
        }

        texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }

    fn mip_level_count(size: u32) -> u32 {
        let mut levels = 1;
        let mut size = size.max(1);
        while size > 1 {
            size = (size / 2).max(1);
            levels += 1;
        }
        levels
    }

    fn downsample_rgba8(src: &[u8], width: u32, height: u32) -> Vec<u8> {
        debug_assert_eq!(src.len(), (width * height * 4) as usize);
        let dst_width = (width / 2).max(1);
        let dst_height = (height / 2).max(1);
        let mut dst = vec![0; (dst_width * dst_height * 4) as usize];

        for y in 0..dst_height {
            for x in 0..dst_width {
                let mut sum = [0u32; 4];
                let mut samples = 0u32;
                let src_x = x * 2;
                let src_y = y * 2;

                for oy in 0..2 {
                    for ox in 0..2 {
                        let px = (src_x + ox).min(width - 1);
                        let py = (src_y + oy).min(height - 1);
                        let src_index = ((py * width + px) * 4) as usize;
                        for channel in 0..4 {
                            sum[channel] += u32::from(src[src_index + channel]);
                        }
                        samples += 1;
                    }
                }

                let dst_index = ((y * dst_width + x) * 4) as usize;
                for channel in 0..4 {
                    dst[dst_index + channel] = (sum[channel] / samples) as u8;
                }
            }
        }

        dst
    }

    fn pad_rgba_rows(src: &[u8], size: u32) -> (Vec<u8>, u32) {
        let row_bytes = size * 4;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes = row_bytes.div_ceil(alignment) * alignment;

        if padded_row_bytes == row_bytes {
            return (src.to_vec(), row_bytes);
        }

        let mut padded = vec![0; (padded_row_bytes * size) as usize];
        for row in 0..size as usize {
            let src_start = row * row_bytes as usize;
            let dst_start = row * padded_row_bytes as usize;
            padded[dst_start..dst_start + row_bytes as usize]
                .copy_from_slice(&src[src_start..src_start + row_bytes as usize]);
        }

        (padded, padded_row_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::TextureAtlas;

    #[test]
    fn mip_chain_reaches_one_pixel() {
        assert_eq!(TextureAtlas::mip_level_count(1), 1);
        assert_eq!(TextureAtlas::mip_level_count(2), 2);
        assert_eq!(TextureAtlas::mip_level_count(256), 9);
    }

    #[test]
    fn downsample_rgba8_averages_four_texels() {
        let src = [
            0, 10, 20, 255, 10, 20, 30, 255, 20, 30, 40, 255, 30, 40, 50, 255,
        ];

        let dst = TextureAtlas::downsample_rgba8(&src, 2, 2);

        assert_eq!(dst, vec![15, 25, 35, 255]);
    }

    #[test]
    fn padded_rows_match_wgpu_alignment() {
        let src = vec![7; 32 * 32 * 4];
        let (padded, bytes_per_row) = TextureAtlas::pad_rgba_rows(&src, 32);

        assert_eq!(bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        assert_eq!(padded.len(), (bytes_per_row * 32) as usize);
        assert_eq!(&padded[..128], &src[..128]);
    }
}

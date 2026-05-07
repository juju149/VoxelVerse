use crate::content::{DecodedMaterialTextureSet, TextureRegistry};

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
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
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
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: materials.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (layer, material) in materials.iter().enumerate() {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                select(material),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(tile_size * 4),
                    rows_per_image: Some(tile_size),
                },
                wgpu::Extent3d {
                    width: tile_size,
                    height: tile_size,
                    depth_or_array_layers: 1,
                },
            );
        }

        texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }
}

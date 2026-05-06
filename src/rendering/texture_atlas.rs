use crate::content::BlockRegistry;

/// GPU texture array where each layer is a 16×16 tile for one block type.
/// All tiles are currently solid white — vertex color carries the block color.
/// When real PNG textures exist in packs/core/textures/blocks/, swap the
/// upload loop to load them instead.
pub(crate) struct TextureAtlas {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureAtlas {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        registry: &BlockRegistry,
    ) -> Self {
        let tile_size = 16u32;
        let num_layers = (registry.block_count() as u32).max(1);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Block Atlas"),
            size: wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: num_layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // All tiles are white so vertex color is the sole color source.
        // Future: load block.png from packs/{ns}/textures/blocks/{stem}.png.
        let tile_pixels = (tile_size * tile_size) as usize;
        let white_tile = vec![255u8; tile_pixels * 4];

        for layer in 0..num_layers {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &white_tile,
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

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self { view, sampler }
    }
}

use std::{fs, path::Path};

use image::{imageops::FilterType, GenericImageView, RgbaImage};
use wgpu::util::DeviceExt;

use vv_registry::{ContentKey, TextureRegistry};

const TILE_SIZE: u32 = 32;

pub(crate) struct BlockTextureAtlas {
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) rect_buffer: wgpu::Buffer,
    _texture: wgpu::Texture,
}

impl BlockTextureAtlas {
    pub(crate) fn build(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets_root: &Path,
        textures: &TextureRegistry,
    ) -> Self {
        let tile_count = textures.len().max(1) as u32;
        let columns = (tile_count as f32).sqrt().ceil() as u32;
        let rows = tile_count.div_ceil(columns);
        let width = columns * TILE_SIZE;
        let height = rows * TILE_SIZE;

        let mut atlas = RgbaImage::from_pixel(width, height, image::Rgba([255, 255, 255, 255]));
        let mut rects = vec![[0.0f32, 0.0, 1.0, 1.0]; tile_count as usize];

        for (index, key) in textures.keys().iter().enumerate() {
            let index = index as u32;
            let tile_x = (index % columns) * TILE_SIZE;
            let tile_y = (index / columns) * TILE_SIZE;
            let tile = load_block_texture(assets_root, key).unwrap_or_else(white_tile);
            image::imageops::overlay(&mut atlas, &tile, tile_x as i64, tile_y as i64);

            let inset_u = 0.5 / width as f32;
            let inset_v = 0.5 / height as f32;
            rects[index as usize] = [
                tile_x as f32 / width as f32 + inset_u,
                tile_y as f32 / height as f32 + inset_v,
                (tile_x + TILE_SIZE) as f32 / width as f32 - inset_u,
                (tile_y + TILE_SIZE) as f32 / height as f32 - inset_v,
            ];
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Block Texture Atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            atlas.as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Block Texture Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let rect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block Texture Atlas Rects"),
            contents: bytemuck::cast_slice(&rects),
            usage: wgpu::BufferUsages::STORAGE,
        });

        Self {
            view,
            sampler,
            rect_buffer,
            _texture: texture,
        }
    }
}

fn load_block_texture(assets_root: &Path, key: &ContentKey) -> Option<RgbaImage> {
    let path = find_block_texture_path(assets_root, key)?;
    let image = image::open(path).ok()?;
    let rgba = image.to_rgba8();
    if image.dimensions() == (TILE_SIZE, TILE_SIZE) {
        Some(rgba)
    } else {
        Some(image::imageops::resize(
            &rgba,
            TILE_SIZE,
            TILE_SIZE,
            FilterType::Nearest,
        ))
    }
}

fn find_block_texture_path(assets_root: &Path, key: &ContentKey) -> Option<std::path::PathBuf> {
    let direct = assets_root
        .join("packs")
        .join(key.namespace())
        .join("resources")
        .join("textures")
        .join("blocks")
        .join(format!("{}.png", key.name()));
    if direct.is_file() {
        return Some(direct);
    }

    let packs_root = assets_root.join("packs");
    let entries = fs::read_dir(&packs_root).ok()?;
    for entry in entries.flatten() {
        let pack_root = entry.path();
        if !pack_root.is_dir() {
            continue;
        }
        let manifest = fs::read_to_string(pack_root.join("pack.ron")).unwrap_or_default();
        if !manifest.contains(&format!("namespace: \"{}\"", key.namespace())) {
            continue;
        }
        let path = pack_root
            .join("resources")
            .join("textures")
            .join("blocks")
            .join(format!("{}.png", key.name()));
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

fn white_tile() -> RgbaImage {
    RgbaImage::from_pixel(TILE_SIZE, TILE_SIZE, image::Rgba([255, 255, 255, 255]))
}

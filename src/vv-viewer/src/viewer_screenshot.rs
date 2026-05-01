/// Screenshot capture: renders the current scene to an offscreen texture
/// and reads back the pixels synchronously using wgpu's MAP_READ buffer.
use std::path::PathBuf;
use vv_registry::ContentKey;

pub fn screenshot_path(key: &ContentKey, scene_label: &str, mode_label: &str) -> PathBuf {
    let safe = format!("{}_{}_{}", key.namespace(), key.name(), scene_label)
        .replace([':', ' ', '/', '\\'], "_");
    PathBuf::from("target/viewer-screenshots").join(format!("{safe}_{mode_label}.png"))
}

/// Render the scene to an offscreen RGBA8 texture and read the pixels back.
/// The pipeline must target `wgpu::TextureFormat::Rgba8UnormSrgb`.
/// Returns the image on success.
#[allow(clippy::too_many_arguments)]
pub fn take_screenshot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    global_bind: &wgpu::BindGroup,
    scene_local_bind: &wgpu::BindGroup,
    scene_vbuf: &wgpu::Buffer,
    scene_ibuf: &wgpu::Buffer,
    scene_index_count: u32,
    width: u32,
    height: u32,
) -> Option<image::RgbaImage> {
    use wgpu::*;

    let format = TextureFormat::Rgba8UnormSrgb;

    let render_tex = device.create_texture(&TextureDescriptor {
        label: Some("screenshot render tex"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let render_view = render_tex.create_view(&TextureViewDescriptor::default());

    let depth_tex = device.create_texture(&TextureDescriptor {
        label: Some("screenshot depth tex"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_tex.create_view(&TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("screenshot pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &render_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.12,
                        g: 0.12,
                        b: 0.14,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        if scene_index_count > 0 {
            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, global_bind, &[]);
            rpass.set_bind_group(1, scene_local_bind, &[]);
            rpass.set_vertex_buffer(0, scene_vbuf.slice(..));
            rpass.set_index_buffer(scene_ibuf.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..scene_index_count, 0, 0..1);
        }
    }

    // wgpu requires row bytes aligned to 256.
    let bytes_per_row = (width * 4 + 255) & !255;
    let buf_size = (bytes_per_row * height) as u64;
    let staging = device.create_buffer(&BufferDescriptor {
        label: Some("screenshot staging"),
        size: buf_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        ImageCopyTexture {
            texture: &render_tex,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        ImageCopyBuffer {
            buffer: &staging,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    // Map and read back synchronously.
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(Maintain::Wait);
    rx.recv().ok()?.ok()?;

    let data = slice.get_mapped_range();
    let mut img = image::RgbaImage::new(width, height);
    for row in 0..height {
        let row_start = (row * bytes_per_row) as usize;
        let src = &data[row_start..row_start + (width * 4) as usize];
        for col in 0..width {
            let i = (col * 4) as usize;
            img.put_pixel(
                col,
                row,
                image::Rgba([src[i], src[i + 1], src[i + 2], src[i + 3]]),
            );
        }
    }
    drop(data);
    staging.unmap();
    Some(img)
}

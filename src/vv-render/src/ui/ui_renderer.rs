use vv_ui::UiFrame;

use super::{create_ui_pipeline, UiGpuBuffers, UiMeshBuilder, UiTextItem};

#[derive(Debug)]
pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    buffers: UiGpuBuffers,
    index_count: u32,
    text_items: Vec<UiTextItem>,
}

impl UiRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            pipeline: create_ui_pipeline(device, format),
            buffers: UiGpuBuffers::new(device),
            index_count: 0,
            text_items: Vec::new(),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &UiFrame) {
        let mesh = UiMeshBuilder::build(frame);
        self.index_count = mesh.indices.len() as u32;
        self.text_items = mesh.text_items.clone();
        self.buffers.upload(device, queue, &mesh);
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn text_items(&self) -> &[UiTextItem] {
        &self.text_items
    }

    pub fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        if self.index_count == 0 {
            return;
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.buffers.vertex_buffer().slice(..));
        pass.set_index_buffer(
            self.buffers.index_buffer().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

use wgpu::util::DeviceExt;

use super::{UiMesh, UiVertex};

#[derive(Debug)]
pub struct UiGpuBuffers {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
}

impl UiGpuBuffers {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_capacity = 4;
        let index_capacity = 6;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("VoxelVerse UI Vertex Buffer"),
            size: (vertex_capacity * std::mem::size_of::<UiVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("VoxelVerse UI Index Buffer"),
            size: (index_capacity * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            vertex_capacity,
            index_capacity,
        }
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &UiMesh) {
        self.ensure_vertex_capacity(device, mesh.vertices.len());
        self.ensure_index_capacity(device, mesh.indices.len());

        if !mesh.vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertices));
        }

        if !mesh.indices.is_empty() {
            queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&mesh.indices));
        }
    }

    fn ensure_vertex_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.vertex_capacity {
            return;
        }

        let next = next_capacity(self.vertex_capacity, needed);
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VoxelVerse UI Vertex Buffer"),
            contents: &vec![0u8; next * std::mem::size_of::<UiVertex>()],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        self.vertex_capacity = next;
    }

    fn ensure_index_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.index_capacity {
            return;
        }

        let next = next_capacity(self.index_capacity, needed);
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VoxelVerse UI Index Buffer"),
            contents: &vec![0u8; next * std::mem::size_of::<u32>()],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        self.index_capacity = next;
    }
}

fn next_capacity(current: usize, needed: usize) -> usize {
    let mut next = current.max(4);
    while next < needed {
        next *= 2;
    }
    next
}

use bytemuck::{Pod, Zeroable};

/// GPU vertex layout — 48 bytes.
/// Attribute locations match shader.wgsl exactly.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],      // offset  0, attr 0
    pub uv: [f32; 2],       // offset 12, attr 1
    pub normal: [f32; 3],   // offset 20, attr 2
    pub color: [f32; 3],    // offset 32, attr 3 — tint / AO
    pub tex_index: u32,     // offset 44, attr 4 — atlas layer
}

pub struct ChunkMesh {
    pub v_buf: wgpu::Buffer,
    pub i_buf: wgpu::Buffer,
    pub num_inds: u32,
    pub num_verts: usize,
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub center: glam::Vec3,
    pub radius: f32,
}

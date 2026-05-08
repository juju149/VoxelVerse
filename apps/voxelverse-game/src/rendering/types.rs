use crate::meshing::CpuVertex;
use bytemuck::{Pod, Zeroable};

/// GPU vertex layout — 48 bytes.
/// Attribute locations match shader.wgsl exactly.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],    // offset  0, attr 0
    pub uv: [f32; 2],     // offset 12, attr 1
    pub normal: [f32; 3], // offset 20, attr 2
    pub color: [f32; 3],  // offset 32, attr 3 — tint / AO
    pub tex_index: u32,   // offset 44, attr 4 — packed material + render flags
}

/// Compile-time guard: diagnostics/render_stats.rs hardcodes VERTEX_BYTES = 48.
/// If the Vertex layout ever changes, this will produce a build error.
const _: () = assert!(
    std::mem::size_of::<Vertex>() == 48,
    "Vertex size changed — update VERTEX_BYTES in diagnostics/render_stats.rs"
);

impl From<CpuVertex> for Vertex {
    #[inline]
    fn from(v: CpuVertex) -> Self {
        Self {
            pos: v.pos,
            uv: v.uv,
            normal: v.normal,
            color: v.color,
            tex_index: v.tex_index,
        }
    }
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

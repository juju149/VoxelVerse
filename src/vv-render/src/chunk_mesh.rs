use glam::Vec3;

/// GPU buffers for a single rendered chunk or LOD tile.
pub struct ChunkMesh {
    pub v_buf: wgpu::Buffer,
    pub i_buf: wgpu::Buffer,
    pub num_inds: u32,
    pub num_verts: usize,
    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    /// Bounding sphere centre (world space).
    pub center: Vec3,
    /// Bounding sphere radius.
    pub radius: f32,
}

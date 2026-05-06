use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
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

pub struct Frustum {
    planes: [glam::Vec4; 6],
}

impl Frustum {
    pub fn from_matrix(m: glam::Mat4) -> Self {
        let r0 = m.row(0);
        let r1 = m.row(1);
        let r2 = m.row(2);
        let r3 = m.row(3);

        let mut planes = [
            r3 + r0, // Left
            r3 - r0, // Right
            r3 + r1, // Bottom
            r3 - r1, // Top
            r3 + r2, // Near
            r3 - r2, // Far
        ];

        for plane in &mut planes {
            let len = glam::Vec3::new(plane.x, plane.y, plane.z).length();
            *plane /= len;
        }

        Self { planes }
    }

    pub fn intersects_sphere(&self, center: glam::Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let dist = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;

            if dist < -radius {
                return false;
            }
        }
        true
    }
}

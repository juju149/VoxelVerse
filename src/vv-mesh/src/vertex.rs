use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub texture_id: i32,
    pub block_id: i32,
    pub block_visual_id: u32,
    pub face_id: u32,
    pub voxel_pos: [i32; 3],
    pub variation_seed: u32,
    pub ao: f32,
}

impl Vertex {
    pub fn untextured(pos: [f32; 3], color: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            pos,
            color,
            normal,
            uv: [0.0, 0.0],
            texture_id: -1,
            block_id: -1,
            block_visual_id: 0,
            face_id: 0,
            voxel_pos: [0, 0, 0],
            variation_seed: 0,
            ao: 1.0,
        }
    }
}

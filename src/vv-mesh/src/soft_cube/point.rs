use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SoftCubePoint {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
}

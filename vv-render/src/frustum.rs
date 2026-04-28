use glam::{Vec3, Vec4, Mat4};

/// Six-plane view frustum used for GPU-side visibility culling.
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    /// Build a frustum from a combined view-projection matrix.
    pub fn from_matrix(m: Mat4) -> Self {
        let r0 = m.row(0);
        let r1 = m.row(1);
        let r2 = m.row(2);
        let r3 = m.row(3);
        let mut planes = [
            r3 + r0, // left
            r3 - r0, // right
            r3 + r1, // bottom
            r3 - r1, // top
            r3 + r2, // near
            r3 - r2, // far
        ];
        for p in &mut planes {
            let len = Vec3::new(p.x, p.y, p.z).length();
            *p /= len;
        }
        Self { planes }
    }

    /// Returns `true` if a sphere (centre + radius) intersects or is inside the frustum.
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let dist = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            if dist < -radius { return false; }
        }
        true
    }
}

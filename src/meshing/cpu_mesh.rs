/// CPU-side vertex produced by the meshing stage.
///
/// No GPU types, no wgpu dependency.  The renderer converts this to its own
/// `rendering::Vertex` (which has identical field layout) before uploading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpuVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    /// Material layer. 0 = neutral fallback.
    pub tex_index: u32,
}

impl CpuVertex {
    #[allow(dead_code)] // used in tests; mesh builders use struct literal syntax
    #[inline]
    pub fn new(pos: [f32; 3], uv: [f32; 2], normal: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            pos,
            uv,
            normal,
            color,
            tex_index: 0,
        }
    }
}

/// Axis-aligned bounding box of a mesh in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshBounds {
    pub center: [f32; 3],
    pub radius: f32,
}

impl MeshBounds {
    /// Compute bounds from a vertex list. Returns a zero-size bound at the origin for empty slices.
    pub fn from_vertices(verts: &[CpuVertex]) -> Self {
        if verts.is_empty() {
            return Self {
                center: [0.0; 3],
                radius: 0.0,
            };
        }

        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];

        for v in verts {
            for i in 0..3 {
                min[i] = min[i].min(v.pos[i]);
                max[i] = max[i].max(v.pos[i]);
            }
        }

        let cx = (min[0] + max[0]) * 0.5;
        let cy = (min[1] + max[1]) * 0.5;
        let cz = (min[2] + max[2]) * 0.5;

        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dz = max[2] - min[2];

        Self {
            center: [cx, cy, cz],
            radius: (dx * dx + dy * dy + dz * dz).sqrt() * 0.5,
        }
    }
}

/// A CPU-side mesh ready to be uploaded to the GPU.
/// Produced by `MeshGen`; consumed by the renderer upload path.
#[derive(Clone, Debug, Default)]
pub struct CpuMesh {
    pub vertices: Vec<CpuVertex>,
    pub indices: Vec<u32>,
    pub bounds: MeshBounds,
}

impl CpuMesh {
    #[allow(dead_code)] // used by tests and future callers
    pub fn new(vertices: Vec<CpuVertex>, indices: Vec<u32>) -> Self {
        let bounds = MeshBounds::from_vertices(&vertices);
        Self {
            vertices,
            indices,
            bounds,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

impl Default for MeshBounds {
    fn default() -> Self {
        Self {
            center: [0.0; 3],
            radius: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(pos: [f32; 3]) -> CpuVertex {
        CpuVertex::new(pos, [0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0])
    }

    #[test]
    fn bounds_empty_mesh() {
        let b = MeshBounds::from_vertices(&[]);
        assert_eq!(b.center, [0.0; 3]);
        assert_eq!(b.radius, 0.0);
    }

    #[test]
    fn bounds_unit_cube() {
        let verts = vec![
            v([0.0, 0.0, 0.0]),
            v([1.0, 0.0, 0.0]),
            v([0.0, 1.0, 0.0]),
            v([1.0, 1.0, 0.0]),
            v([0.0, 0.0, 1.0]),
            v([1.0, 0.0, 1.0]),
            v([0.0, 1.0, 1.0]),
            v([1.0, 1.0, 1.0]),
        ];
        let b = MeshBounds::from_vertices(&verts);
        assert_eq!(b.center, [0.5, 0.5, 0.5]);
        // diagonal of unit cube = sqrt(3) ≈ 1.732, half = 0.866
        let expected_r = (3.0_f32).sqrt() * 0.5;
        assert!((b.radius - expected_r).abs() < 1e-5);
    }

    #[test]
    fn cpu_mesh_is_empty_for_default() {
        let m = CpuMesh::default();
        assert!(m.is_empty());
    }

    #[test]
    fn indices_multiple_of_three_for_triangles() {
        // A single quad = 2 triangles = 6 indices
        let verts = vec![
            v([0.0, 0.0, 0.0]),
            v([1.0, 0.0, 0.0]),
            v([1.0, 1.0, 0.0]),
            v([0.0, 1.0, 0.0]),
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        let mesh = CpuMesh::new(verts, indices);
        assert_eq!(mesh.indices.len() % 3, 0);
    }
}

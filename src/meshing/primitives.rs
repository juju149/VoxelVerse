use super::{CpuMesh, CpuVertex, MeshGen};
use crate::content::TerrainPalette;
use glam::Vec3;

impl MeshGen {
    pub fn generate_cylinder(radius: f32, height: f32, segments: u32) -> CpuMesh {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let color = TerrainPalette::PLAYER;

        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * radius;
            let z = theta.sin() * radius;
            let normal = Vec3::new(x, 0.0, z).normalize().to_array();

            verts.push(CpuVertex {
                pos: [x, 0.0, z],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            });

            verts.push(CpuVertex {
                pos: [x, height, z],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            });
        }

        for i in 0..segments {
            let bottom1 = i * 2;
            let top1 = bottom1 + 1;
            let bottom2 = bottom1 + 2;
            let top2 = bottom1 + 3;

            inds.push(bottom1);
            inds.push(top1);
            inds.push(bottom2);
            inds.push(bottom2);
            inds.push(top1);
            inds.push(top2);
        }

        let center_idx = verts.len() as u32;
        verts.push(CpuVertex {
            pos: [0.0, height, 0.0],
            uv: [0.0, 0.0],
            color,
            normal: [0.0, 1.0, 0.0],
            tex_index: 0,
        });
        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * radius;
            let z = theta.sin() * radius;
            verts.push(CpuVertex {
                pos: [x, height, z],
                uv: [0.0, 0.0],
                color,
                normal: [0.0, 1.0, 0.0],
                tex_index: 0,
            });
        }
        for i in 0..segments {
            inds.push(center_idx);
            inds.push(center_idx + 1 + i);
            inds.push(center_idx + 1 + i + 1);
        }

        CpuMesh::new(verts, inds)
    }

    /// Generates a simple 2D crosshair for the center of the screen.
    pub fn generate_crosshair() -> CpuMesh {
        let s = 0.02; // size relative to screen (2%)
        let color = TerrainPalette::UI_WHITE;
        let normal = [0.0, 0.0, 1.0];

        let verts = vec![
            CpuVertex {
                pos: [-s, 0.0, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            CpuVertex {
                pos: [s, 0.0, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            CpuVertex {
                pos: [0.0, -s, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
            CpuVertex {
                pos: [0.0, s, 0.0],
                uv: [0.0, 0.0],
                color,
                normal,
                tex_index: 0,
            },
        ];
        let inds = vec![0, 1, 2, 3];
        CpuMesh::new(verts, inds)
    }
}

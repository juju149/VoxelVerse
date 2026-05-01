use glam::Vec3;

use crate::{MeshGen, Vertex};

impl MeshGen {
    pub fn generate_cylinder(radius: f32, height: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let color = [0.0, 0.5, 1.0];

        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * radius;
            let z = theta.sin() * radius;
            let n = Vec3::new(x, 0.0, z).normalize().to_array();

            verts.push(Vertex::untextured([x, 0.0, z], color, n));
            verts.push(Vertex::untextured([x, height, z], color, n));
        }

        for i in 0..segments {
            let b1 = i * 2;
            let t1 = b1 + 1;
            let b2 = b1 + 2;
            let t2 = b1 + 3;

            inds.extend_from_slice(&[b1, t1, b2, b2, t1, t2]);
        }

        let center_index = verts.len() as u32;

        verts.push(Vertex::untextured(
            [0.0, height, 0.0],
            color,
            [0.0, 1.0, 0.0],
        ));

        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;

            verts.push(Vertex::untextured(
                [theta.cos() * radius, height, theta.sin() * radius],
                color,
                [0.0, 1.0, 0.0],
            ));
        }

        for i in 0..segments {
            inds.push(center_index);
            inds.push(center_index + 1 + i);
            inds.push(center_index + 2 + i);
        }

        (verts, inds)
    }

    pub fn generate_sphere_guide(radius: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let color = [1.0, 1.0, 1.0];

        for y in 0..=segments {
            for x in 0..=segments {
                let xs = x as f32 / segments as f32;
                let ys = y as f32 / segments as f32;

                let xp = (xs * std::f32::consts::TAU).cos() * (ys * std::f32::consts::PI).sin();
                let yp = (ys * std::f32::consts::PI).cos();
                let zp = (xs * std::f32::consts::TAU).sin() * (ys * std::f32::consts::PI).sin();

                verts.push(Vertex::untextured(
                    [xp * radius, yp * radius, zp * radius],
                    color,
                    [xp, yp, zp],
                ));
            }
        }

        for y in 0..segments {
            for x in 0..segments {
                let i = y * (segments + 1) + x;

                inds.extend_from_slice(&[
                    i,
                    i + segments + 1,
                    i + segments + 2,
                    i + segments + 2,
                    i + 1,
                    i,
                ]);
            }
        }

        (verts, inds)
    }

    pub fn generate_crosshair() -> (Vec<Vertex>, Vec<u32>) {
        let s = 0.02f32;
        let color = [1.0, 1.0, 1.0];
        let normal = [0.0, 0.0, 1.0];

        let verts = vec![
            Vertex::untextured([-s, 0.0, 0.0], color, normal),
            Vertex::untextured([s, 0.0, 0.0], color, normal),
            Vertex::untextured([0.0, -s, 0.0], color, normal),
            Vertex::untextured([0.0, s, 0.0], color, normal),
        ];

        (verts, vec![0, 1, 2, 3])
    }
}

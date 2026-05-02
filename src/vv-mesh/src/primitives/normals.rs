use glam::Vec3;

use crate::MeshGen;

impl MeshGen {
    #[inline]
    pub(crate) fn face_normal(pos: [Vec3; 4]) -> Vec3 {
        (pos[1] - pos[0]).cross(pos[2] - pos[0]).normalize()
    }

    pub(crate) fn rounded_corner_normals(
        base: Vec3,
        adjacent_faces: [[(bool, Vec3); 2]; 4],
        strength: f32,
    ) -> [Vec3; 4] {
        if strength <= 0.0 {
            return [base; 4];
        }

        let t = strength.clamp(0.0, 1.0);
        let mut normals = [base; 4];

        for (normal, adjacent) in normals.iter_mut().zip(adjacent_faces) {
            let mut target = Vec3::ZERO;
            let mut count = 0u32;

            for (visible, face_normal) in adjacent {
                if visible {
                    target += face_normal;
                    count += 1;
                }
            }

            if count == 0 {
                continue;
            }

            let len = target.length();
            if len < 1e-8 {
                continue;
            }

            *normal = slerp_normal(base, target / len, t);
        }

        normals
    }
}

#[inline]
pub(super) fn slerp_normal(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let dot = a.dot(b).clamp(-1.0, 1.0);
    let theta = dot.acos();

    if theta < 1e-6 {
        return a;
    }

    let sin_theta = theta.sin();
    (a * ((1.0 - t) * theta).sin() + b * (t * theta).sin()) / sin_theta
}

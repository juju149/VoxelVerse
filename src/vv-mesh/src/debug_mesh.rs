use glam::Vec3;

use vv_core::BlockId;
use vv_planet::CoordSystem;
use vv_world_runtime::PlanetData;

use crate::{MeshGen, Vertex};

impl MeshGen {
    pub fn generate_collision_debug(
        player_pos: Vec3,
        planet: &PlanetData,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();

        let res = planet.resolution;
        let color = [1.0, 0.0, 0.0];
        let normal = [0.0, 1.0, 0.0];
        let range = 2i32;
        let mut idx = 0u32;

        let Some((center_id, _)) = CoordSystem::get_local_coords(player_pos, planet.geometry)
        else {
            return (verts, inds);
        };

        let su = (center_id.u as i32 - range).max(0);
        let eu = (center_id.u as i32 + range).min(res as i32 - 1);
        let sv = (center_id.v as i32 - range).max(0);
        let ev = (center_id.v as i32 + range).min(res as i32 - 1);
        let sl = (center_id.layer as i32 - range).max(0);
        let el = (center_id.layer as i32 + range).min(res as i32 - 1);

        for layer in sl..=el {
            for v in sv..=ev {
                for u in su..=eu {
                    let id = BlockId {
                        face: center_id.face,
                        layer: layer as u32,
                        u: u as u32,
                        v: v as u32,
                    };

                    if !planet.exists(id) {
                        continue;
                    }

                    let gp = |uu, vv, ll| {
                        CoordSystem::get_vertex_pos(
                            id.face,
                            id.u + uu,
                            id.v + vv,
                            id.layer + ll,
                            planet.geometry,
                        )
                    };

                    let c000 = gp(0, 0, 0);
                    let c100 = gp(1, 0, 0);
                    let c010 = gp(0, 1, 0);
                    let c110 = gp(1, 1, 0);
                    let c001 = gp(0, 0, 1);
                    let c101 = gp(1, 0, 1);
                    let c011 = gp(0, 1, 1);
                    let c111 = gp(1, 1, 1);

                    let center = (c000 + c100 + c010 + c110 + c001 + c101 + c011 + c111) * 0.125;
                    let shrink = 0.90f32;

                    let vi = |p: Vec3| {
                        Vertex::untextured(
                            (center + (p - center) * shrink).to_array(),
                            color,
                            normal,
                        )
                    };

                    let corners = [
                        vi(c000),
                        vi(c100),
                        vi(c110),
                        vi(c010),
                        vi(c001),
                        vi(c101),
                        vi(c111),
                        vi(c011),
                    ];

                    for corner in &corners {
                        verts.push(*corner);
                    }

                    let base = idx;

                    let lines = [
                        (0, 1),
                        (1, 2),
                        (2, 3),
                        (3, 0),
                        (4, 5),
                        (5, 6),
                        (6, 7),
                        (7, 4),
                        (0, 4),
                        (1, 5),
                        (2, 6),
                        (3, 7),
                    ];

                    for (start, end) in lines {
                        inds.push(base + start);
                        inds.push(base + end);
                    }

                    idx += 8;
                }
            }
        }

        (verts, inds)
    }
}

use super::MeshGen;
use crate::content::TerrainPalette;
use crate::generation::CoordSystem;
use crate::rendering::Vertex;
use crate::voxel::VoxelCoord;
use crate::world::PlanetData;
use glam::Vec3;

impl MeshGen {
    pub fn generate_collision_debug(
        player_pos: Vec3,
        planet: &PlanetData,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut verts = Vec::new();
        let mut inds = Vec::new();
        let res = planet.resolution;
        let color = TerrainPalette::COLLISION_DEBUG;
        let normal = [0.0, 1.0, 0.0];

        // check a 3x3x3 area around the player
        let range = 2;

        if let Some((center_id, _)) = CoordSystem::get_local_coords(player_pos, res) {
            let start_u = (center_id.u as i32 - range).max(0);
            let end_u = (center_id.u as i32 + range).min(res as i32 - 1);
            let start_v = (center_id.v as i32 - range).max(0);
            let end_v = (center_id.v as i32 + range).min(res as i32 - 1);
            let start_l = (center_id.layer as i32 - range).max(0);
            let end_l = (center_id.layer as i32 + range).min(res as i32 - 1);

            let mut idx = 0;

            for l in start_l..=end_l {
                for v in start_v..=end_v {
                    for u in start_u..=end_u {
                        let id = VoxelCoord {
                            face: center_id.face,
                            layer: l as u32,
                            u: u as u32,
                            v: v as u32,
                        };

                        let block_pos =
                            CoordSystem::get_block_center(id.face, id.u, id.v, id.layer, res);

                        if crate::physics::Physics::is_solid(block_pos, planet) {
                            // visualize the "Core" of the block that triggers collision
                            let get_p = |uu, vv, ll| {
                                CoordSystem::get_vertex_pos(
                                    id.face,
                                    id.u + uu,
                                    id.v + vv,
                                    id.layer + ll,
                                    res,
                                )
                            };

                            // get corners of the voxel
                            let c000 = get_p(0, 0, 0);
                            let c100 = get_p(1, 0, 0);
                            let c010 = get_p(0, 1, 0);
                            let c110 = get_p(1, 1, 0);
                            let c001 = get_p(0, 0, 1);
                            let c101 = get_p(1, 0, 1);
                            let c011 = get_p(0, 1, 1);
                            let c111 = get_p(1, 1, 1);

                            // shrink corners towards center by margin (visualize the "shave")
                            let center =
                                (c000 + c100 + c010 + c110 + c001 + c101 + c011 + c111) * 0.125;
                            let shrink = 0.90; // Exaggerate the shrink slightly so we can see it inside the block

                            let v = |p: Vec3| Vertex {
                                pos: (center + (p - center) * shrink).to_array(),
                                color,
                                normal,
                            };

                            let corners = [
                                v(c000),
                                v(c100),
                                v(c110),
                                v(c010), // Bottom
                                v(c001),
                                v(c101),
                                v(c111),
                                v(c011), // Top
                            ];

                            // add vertices
                            for c in &corners {
                                verts.push(*c);
                            }

                            // add line indices (Cube wireframe)
                            let base = idx;
                            let lines = [
                                (0, 1),
                                (1, 2),
                                (2, 3),
                                (3, 0), // Bottom ring
                                (4, 5),
                                (5, 6),
                                (6, 7),
                                (7, 4), // Top ring
                                (0, 4),
                                (1, 5),
                                (2, 6),
                                (3, 7), // Pillars
                            ];

                            for (s, e) in lines {
                                inds.push(base + s);
                                inds.push(base + e);
                            }
                            idx += 8;
                        }
                    }
                }
            }
        }
        (verts, inds)
    }
}

//! Builds `ChunkMeshInput` and `LodMeshInput` from a `PlanetSnapshot`.
//!
//! This is the single place that knows about both the world data model and the
//! mesher input types.  The mesher itself receives only the baked structs and
//! never touches `PlanetSnapshot`, `BlockRegistry`, or `ProceduralPlanetTerrain`.

use std::sync::Arc;

use vv_math::SphericalGrid;
use vv_meshing::{
    BakedPropFace, ChunkBorderSamples, ChunkMeshInput, ChunkVoxelView, LodCellColors, LodMeshInput,
    MeshMaterialEntry, MeshMaterialTable, PropMeshInstance, PropMeshModel, PropSurfaceOrientation,
    VoxelMeshClass, VoxelMeshKind, VoxelMeshingConfig, VoxelVisual, VoxelVisualLayers,
};
use vv_pack_compiler::{CompiledMesh, CompiledMeshClass};
use vv_voxel::{LodKey, SurfaceChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};

use crate::PlanetSnapshot;

impl PlanetSnapshot {
    // -----------------------------------------------------------------------
    // Voxel chunk
    // -----------------------------------------------------------------------

    /// Build a self-contained input for `MeshGen::build_chunk`.
    ///
    /// All world knowledge is resolved here; the mesher receives only pure
    /// geometry data.
    pub fn prepare_chunk_mesh_input(
        &self,
        key: SurfaceChunkKey,
        config: VoxelMeshingConfig,
    ) -> ChunkMeshInput {
        let mut voxels = ChunkVoxelView::new(key.face, self.resolution);
        let border = build_border_samples(self, key);

        let u_start = key.u_idx * CHUNK_SIZE;
        let v_start = key.v_idx * CHUNK_SIZE;
        let u_end = (u_start + CHUNK_SIZE).min(self.resolution);
        let v_end = (v_start + CHUNK_SIZE).min(self.resolution);

        // Terrain surface + cliff fill + water.
        for u in u_start..u_end {
            for v in v_start..v_end {
                let h = border.surface_height(u, v);
                if h == 0 {
                    continue;
                }
                // Surface block.
                let surface_id = self.generated_voxel(VoxelCoord {
                    face: key.face,
                    layer: h,
                    u,
                    v,
                });
                voxels.insert(h, u, v, surface_id);

                // Cliff fill (layers below surface down to deepest neighbor).
                let min_h = min_neighbor_height(&border, u, v, self.resolution);
                if min_h < h {
                    let bottom = min_h.max(h.saturating_sub(config.cliff_fill_depth));
                    for l in (bottom + 1)..h {
                        let id = self.generated_voxel(VoxelCoord {
                            face: key.face,
                            layer: l,
                            u,
                            v,
                        });
                        voxels.insert(l, u, v, id);
                    }
                }

                // Water at sea level.
                let sea = self.terrain.sea_level_layer();
                if let Some(water) = self.terrain.water_block() {
                    if h < sea {
                        voxels.insert(sea, u, v, water);
                    }
                }
            }
        }

        // Feature voxels (trees, structures) baked by worldgen.
        let features = self.bake_chunk_features(key, 1);
        for (coord, id) in &features.blocks {
            voxels.insert(coord.layer, coord.u, coord.v, *id);
            // Insert 6-connected neighbors so face culling has correct data.
            insert_neighbors(&mut voxels, coord.layer, coord.u, coord.v, self, key.face);
        }

        // Player overrides (modified voxels) for this chunk and 4 neighbors.
        for (coord, id) in self.modified_voxels_in_chunk_column(key) {
            voxels.insert(coord.layer, coord.u, coord.v, id);
            insert_neighbors(&mut voxels, coord.layer, coord.u, coord.v, self, key.face);
        }

        // Prop instances.
        let prop_instances = build_prop_instances(self, key, config);

        ChunkMeshInput {
            key,
            voxels,
            border_samples: border,
            material_table: Arc::clone(&self.material_table),
            prop_instances,
        }
    }

    // -----------------------------------------------------------------------
    // LOD tile
    // -----------------------------------------------------------------------

    /// Build a self-contained input for `MeshGen::generate_lod_mesh`.
    pub fn prepare_lod_mesh_input(&self, key: LodKey) -> LodMeshInput {
        let n = CHUNK_SIZE;
        let step = (key.size / n).max(1);
        let grid = SphericalGrid::new(
            self.profile.resolution,
            self.profile.inner_radius,
            self.profile.layer_height,
        );
        let sea = self.terrain.sea_level_layer();
        let water_block = self.terrain.water_block();

        let total = (n * n) as usize;
        let mut corner_heights = Vec::with_capacity(((n + 1) * (n + 1)) as usize);
        let mut cell_heights = Vec::with_capacity(total);
        let mut cell_colors = Vec::with_capacity(total);

        // Corner heights (one extra row/column for wall emission).
        for cj in 0..=(n as usize) {
            for ci in 0..=(n as usize) {
                let u = (key.x + ci as u32 * step).min(self.resolution.saturating_sub(1));
                let v = (key.y + cj as u32 * step).min(self.resolution.saturating_sub(1));
                corner_heights.push(self.terrain.terrain_surface_layer(key.face, u, v));
            }
        }

        for cj in 0..n {
            for ci in 0..n {
                let u = (key.x + ci * step + step / 2).min(self.resolution.saturating_sub(1));
                let v = (key.y + cj * step + step / 2).min(self.resolution.saturating_sub(1));
                let terrain_h = self.terrain.terrain_surface_layer(key.face, u, v);
                let has_water = water_block.is_some() && terrain_h < sea;
                let display_h = if has_water { sea } else { terrain_h };
                cell_heights.push(display_h);

                let (top_id, wall_id) = self.terrain.lod_surface_blocks(key.face, u, v);
                let top_color = self.terrain_visuals.block_color(if has_water {
                    water_block.unwrap_or(top_id)
                } else {
                    top_id
                });
                let wall_color = self.terrain_visuals.block_color(wall_id);
                cell_colors.push(LodCellColors {
                    top: top_color,
                    wall: wall_color,
                    is_water: has_water,
                });
            }
        }

        LodMeshInput {
            key,
            grid,
            corner_heights,
            cell_heights,
            cell_colors,
            sea_level: sea,
            skirt_layers: 4,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(crate) fn build_material_table(content: &vv_pack_compiler::BlockRegistry) -> MeshMaterialTable {
    let count = content.block_count();
    let mut entries = Vec::with_capacity(count);
    for raw in 0..count {
        let id = VoxelId::new(raw as u16);
        entries.push(block_to_material_entry(content, id));
    }
    MeshMaterialTable::new(entries)
}

fn block_to_material_entry(
    content: &vv_pack_compiler::BlockRegistry,
    id: VoxelId,
) -> MeshMaterialEntry {
    if id == VoxelId::AIR {
        return MeshMaterialEntry::default();
    }
    let Some(block) = content.block(id) else {
        return MeshMaterialEntry::default();
    };
    let model = content.model_of(id);

    let mesh_kind = match &model.mesh {
        CompiledMesh::None => VoxelMeshKind::None,
        CompiledMesh::Cube { .. } => VoxelMeshKind::Cube,
        CompiledMesh::CubeColumn { .. } => VoxelMeshKind::CubeColumn,
    };
    let mesh_class = match block.mesh_class {
        CompiledMeshClass::Water => VoxelMeshClass::Water,
        CompiledMeshClass::Invisible => VoxelMeshClass::None,
        _ => VoxelMeshClass::Solid,
    };
    let layers_raw = block.visual.layers;
    let visual = VoxelVisual {
        layers: VoxelVisualLayers {
            top: layers_raw.top,
            bottom: layers_raw.bottom,
            front: layers_raw.front,
            back: layers_raw.back,
            left: layers_raw.left,
            right: layers_raw.right,
        },
        tint: block.visual.tint,
    };

    MeshMaterialEntry {
        is_renderable: mesh_kind != VoxelMeshKind::None,
        is_opaque_cube: content.is_opaque_cube(id),
        uses_greedy: content.uses_greedy_opaque_meshing(id),
        mesh_class,
        mesh_kind,
        visual,
        color: block.color,
    }
}

fn build_border_samples(snapshot: &PlanetSnapshot, key: SurfaceChunkKey) -> ChunkBorderSamples {
    let u_start = key.u_idx * CHUNK_SIZE;
    let v_start = key.v_idx * CHUNK_SIZE;
    let width = CHUNK_SIZE + 2;
    let ubs = u_start.saturating_sub(1);
    let vbs = v_start.saturating_sub(1);
    let sea = snapshot.terrain.sea_level_layer();
    let water = snapshot.terrain.water_block().unwrap_or(VoxelId::AIR);

    let mut heights = Vec::with_capacity((width * width) as usize);
    for ui in 0..width {
        for vi in 0..width {
            let u = (ubs + ui).min(snapshot.resolution.saturating_sub(1));
            let v = (vbs + vi).min(snapshot.resolution.saturating_sub(1));
            heights.push(snapshot.terrain.terrain_surface_layer(key.face, u, v));
        }
    }

    ChunkBorderSamples::new(
        key.face,
        ubs,
        vbs,
        width,
        snapshot.resolution,
        sea,
        water,
        snapshot.profile,
        heights,
    )
}

fn min_neighbor_height(border: &ChunkBorderSamples, u: u32, v: u32, resolution: u32) -> u32 {
    let mut m = border.surface_height(u, v);
    if u > 0 {
        m = m.min(border.surface_height(u - 1, v));
    }
    if u + 1 < resolution {
        m = m.min(border.surface_height(u + 1, v));
    }
    if v > 0 {
        m = m.min(border.surface_height(u, v - 1));
    }
    if v + 1 < resolution {
        m = m.min(border.surface_height(u, v + 1));
    }
    m
}

fn insert_neighbors(
    voxels: &mut ChunkVoxelView,
    layer: u32,
    u: u32,
    v: u32,
    snapshot: &PlanetSnapshot,
    face: u8,
) {
    let res = snapshot.resolution;
    let mut try_insert = |l: u32, nu: u32, nv: u32| {
        if nu < res && nv < res {
            let id = snapshot.get_voxel(VoxelCoord {
                face,
                layer: l,
                u: nu,
                v: nv,
            });
            voxels.insert(l, nu, nv, id);
        }
    };
    if layer + 1 < res {
        try_insert(layer + 1, u, v);
    }
    if layer > 0 {
        try_insert(layer - 1, u, v);
    }
    if u > 0 {
        try_insert(layer, u - 1, v);
    }
    if u + 1 < res {
        try_insert(layer, u + 1, v);
    }
    if v > 0 {
        try_insert(layer, u, v - 1);
    }
    if v + 1 < res {
        try_insert(layer, u, v + 1);
    }
}

fn build_prop_instances(
    snapshot: &PlanetSnapshot,
    key: SurfaceChunkKey,
    config: VoxelMeshingConfig,
) -> Vec<PropMeshInstance> {
    if snapshot.prop_models.models.is_empty() {
        return Vec::new();
    }
    if !vv_meshing::MeshGen::should_bake_props_for_chunk(key, snapshot.player_surface_key, config) {
        return Vec::new();
    }

    let stamps = snapshot.terrain.props_for_chunk(key);
    let mut instances = Vec::new();

    for stamp in stamps {
        // Skip props on broken (player-destroyed) columns.
        if !snapshot.broken_props.is_alive(key.face, stamp.u, stamp.v) {
            continue;
        }
        let model = match snapshot.prop_models.get(&stamp.model_key) {
            Some(m) => m,
            None => continue,
        };
        let prop_model = Arc::new(vox_to_prop_model(model));
        if prop_model.is_empty() {
            continue;
        }
        let orientation = match stamp.orientation {
            vv_worldgen::PropOrientation::Floor => PropSurfaceOrientation::Floor,
            vv_worldgen::PropOrientation::Ceiling => PropSurfaceOrientation::Ceiling,
        };
        instances.push(PropMeshInstance {
            face: stamp.face,
            u: stamp.u,
            v: stamp.v,
            surface_layer: stamp.surface_layer,
            model: prop_model,
            rotation: stamp.rotation,
            orientation,
        });
    }

    instances
}

fn vox_to_prop_model(model: &crate::VoxModel) -> PropMeshModel {
    let mut faces = Vec::new();
    for face in &model.faces {
        faces.push(BakedPropFace {
            corners: face.corners,
            rgb: face.rgb,
        });
    }
    PropMeshModel {
        size_x: model.size_x,
        size_y: model.size_y,
        size_z: model.size_z,
        faces,
    }
}

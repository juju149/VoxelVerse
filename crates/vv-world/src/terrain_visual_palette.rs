use vv_pack_compiler::{BlockRegistry, TextureRegistry};
use vv_voxel::VoxelId;

#[derive(Clone, Debug)]
pub struct TerrainVisualPalette {
    block_colors: Vec<[f32; 3]>,
}

impl TerrainVisualPalette {
    pub fn from_textures(blocks: &BlockRegistry, textures: &TextureRegistry) -> Self {
        let mut block_colors = Vec::with_capacity(blocks.block_count());
        for raw in 0..blocks.block_count() {
            let id = VoxelId::new(raw as u16);
            let color = blocks
                .block(id)
                .map(|_| block_material_color(blocks, textures, id))
                .unwrap_or([1.0, 1.0, 1.0]);
            block_colors.push(color);
        }
        Self { block_colors }
    }

    pub fn fallback_from_blocks(blocks: &BlockRegistry) -> Self {
        let mut block_colors = Vec::with_capacity(blocks.block_count());
        for raw in 0..blocks.block_count() {
            let id = VoxelId::new(raw as u16);
            let color = blocks
                .block(id)
                .map(|block| block.color)
                .unwrap_or([1.0, 1.0, 1.0]);
            block_colors.push(color);
        }
        Self { block_colors }
    }

    pub fn block_color(&self, id: VoxelId) -> [f32; 3] {
        self.block_colors
            .get(id.raw() as usize)
            .copied()
            .unwrap_or([1.0, 1.0, 1.0])
    }
}

fn block_material_color(
    blocks: &BlockRegistry,
    textures: &TextureRegistry,
    id: VoxelId,
) -> [f32; 3] {
    let visual = blocks.visual(id);
    let layer = first_surface_layer(visual.layers.top, visual.layers.front, visual.layers.right);
    let mut color = if layer == 0 {
        blocks.color(id)
    } else {
        textures.average_albedo_color(layer)
    };
    color[0] *= visual.tint[0];
    color[1] *= visual.tint[1];
    color[2] *= visual.tint[2];
    color
}

fn first_surface_layer(top: u32, front: u32, right: u32) -> u32 {
    [top, front, right]
        .into_iter()
        .find(|layer| *layer != 0)
        .unwrap_or(0)
}


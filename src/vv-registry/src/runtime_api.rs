use std::sync::Arc;

use crate::{
    BiomeId, BiomeRegistry, BlockId, BlockRegistry, BlockVisualId, BlockVisualRegistry,
    CompiledBiome, CompiledBlockMining, CompiledBlockPhysics, CompiledBlockRender,
    CompiledClimateCurves, CompiledClimateTags, CompiledContent, CompiledDrops, CompiledFlora,
    CompiledOre, CompiledPlanetType, CompiledWorldSettings, ContentKey, FloraId, FloraRegistry,
    OreId, OreRegistry, PlanetTypeId, PlanetTypeRegistry, RuntimeBlockVisual, TagId,
};

#[derive(Debug, Clone)]
pub struct BlockContent {
    blocks: Arc<BlockRegistry>,
    visuals: Arc<BlockVisualRegistry>,
    palettes: Arc<Vec<[f32; 4]>>,
}

impl BlockContent {
    pub fn new(
        blocks: BlockRegistry,
        visuals: BlockVisualRegistry,
        palettes: Vec<[f32; 4]>,
    ) -> Self {
        Self {
            blocks: Arc::new(blocks),
            visuals: Arc::new(visuals),
            palettes: Arc::new(palettes),
        }
    }

    pub fn as_view(&self) -> BlockContentView<'_> {
        BlockContentView::new(&self.blocks, &self.visuals, &self.palettes)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockRuntimeView<'a> {
    pub id: BlockId,
    pub key: &'a ContentKey,
    pub stack_max: u8,
    pub tags: &'a [TagId],
    pub mining: &'a CompiledBlockMining,
    pub physics: &'a CompiledBlockPhysics,
    pub drops: &'a CompiledDrops,
}

#[derive(Debug, Clone, Copy)]
pub struct PlanetTypeView<'a> {
    pub id: PlanetTypeId,
    pub key: &'a ContentKey,
    pub data: &'a CompiledPlanetType,
}

#[derive(Debug, Clone, Copy)]
pub struct BiomeView<'a> {
    pub id: BiomeId,
    pub key: &'a ContentKey,
    pub data: &'a CompiledBiome,
}

#[derive(Debug, Clone, Copy)]
pub struct FloraView<'a> {
    pub id: FloraId,
    pub key: &'a ContentKey,
    pub data: &'a CompiledFlora,
}

#[derive(Debug, Clone, Copy)]
pub struct OreView<'a> {
    pub id: OreId,
    pub key: &'a ContentKey,
    pub data: &'a CompiledOre,
}

pub trait BlockRuntimeSource {
    fn block_runtime(&self, id: BlockId) -> Option<BlockRuntimeView<'_>>;
    fn block_key(&self, id: BlockId) -> Option<&ContentKey>;
}

pub trait BlockRenderSource {
    fn block_render(&self, id: BlockId) -> Option<&CompiledBlockRender>;
    fn block_visual(&self, id: BlockVisualId) -> Option<&RuntimeBlockVisual>;
    fn block_visual_palette(&self) -> &[[f32; 4]];
}

pub trait WorldSettingsSource {
    fn world_settings(&self) -> &CompiledWorldSettings;
}

pub trait PlanetTypeSource {
    fn default_planet_type(&self) -> Option<PlanetTypeId>;
    fn planet_type(&self, id: PlanetTypeId) -> Option<PlanetTypeView<'_>>;
}

pub trait BiomeSource {
    fn biome(&self, id: BiomeId) -> Option<BiomeView<'_>>;
}

pub trait FloraSource {
    fn flora(&self, id: FloraId) -> Option<FloraView<'_>>;
}

pub trait OreSource {
    fn ore(&self, id: OreId) -> Option<OreView<'_>>;
}

pub trait WorldgenSettingsSource {
    fn climate_tags(&self) -> &CompiledClimateTags;
    fn climate_curves(&self) -> &CompiledClimateCurves;
}

#[derive(Debug, Clone, Copy)]
pub struct BlockContentView<'a> {
    blocks: &'a BlockRegistry,
    visuals: &'a BlockVisualRegistry,
    palettes: &'a [[f32; 4]],
}

impl<'a> BlockContentView<'a> {
    pub(crate) fn new(
        blocks: &'a BlockRegistry,
        visuals: &'a BlockVisualRegistry,
        palettes: &'a [[f32; 4]],
    ) -> Self {
        Self {
            blocks,
            visuals,
            palettes,
        }
    }
}

impl BlockRuntimeSource for BlockContentView<'_> {
    fn block_runtime(&self, id: BlockId) -> Option<BlockRuntimeView<'_>> {
        let block = self.blocks.get(id)?;
        Some(BlockRuntimeView {
            id,
            key: self.blocks.key(id)?,
            stack_max: block.stack_max,
            tags: &block.tags,
            mining: &block.mining,
            physics: &block.physics,
            drops: &block.drops,
        })
    }

    fn block_key(&self, id: BlockId) -> Option<&ContentKey> {
        self.blocks.key(id)
    }
}

impl BlockRenderSource for BlockContentView<'_> {
    fn block_render(&self, id: BlockId) -> Option<&CompiledBlockRender> {
        self.blocks.get(id).map(|block| &block.render)
    }

    fn block_visual(&self, id: BlockVisualId) -> Option<&RuntimeBlockVisual> {
        self.visuals.get(id)
    }

    fn block_visual_palette(&self) -> &[[f32; 4]] {
        self.palettes
    }
}

impl BlockRuntimeSource for BlockContent {
    fn block_runtime(&self, id: BlockId) -> Option<BlockRuntimeView<'_>> {
        let block = self.blocks.get(id)?;
        Some(BlockRuntimeView {
            id,
            key: self.blocks.key(id)?,
            stack_max: block.stack_max,
            tags: &block.tags,
            mining: &block.mining,
            physics: &block.physics,
            drops: &block.drops,
        })
    }

    fn block_key(&self, id: BlockId) -> Option<&ContentKey> {
        self.blocks.key(id)
    }
}

impl BlockRenderSource for BlockContent {
    fn block_render(&self, id: BlockId) -> Option<&CompiledBlockRender> {
        self.blocks.get(id).map(|block| &block.render)
    }

    fn block_visual(&self, id: BlockVisualId) -> Option<&RuntimeBlockVisual> {
        self.visuals.get(id)
    }

    fn block_visual_palette(&self) -> &[[f32; 4]] {
        &self.palettes
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldContentView<'a> {
    settings: &'a CompiledWorldSettings,
}

impl<'a> WorldContentView<'a> {
    pub(crate) fn new(settings: &'a CompiledWorldSettings) -> Self {
        Self { settings }
    }
}

impl WorldSettingsSource for WorldContentView<'_> {
    fn world_settings(&self) -> &CompiledWorldSettings {
        self.settings
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldgenContentView<'a> {
    planet_types: &'a PlanetTypeRegistry,
    biomes: &'a BiomeRegistry,
    flora: &'a FloraRegistry,
    ores: &'a OreRegistry,
    default_planet_type: Option<PlanetTypeId>,
    climate_tags: &'a CompiledClimateTags,
    climate_curves: &'a CompiledClimateCurves,
}

impl<'a> WorldgenContentView<'a> {
    pub(crate) fn new(content: &'a CompiledContent) -> Self {
        Self {
            planet_types: &content.planet_types,
            biomes: &content.biomes,
            flora: &content.flora,
            ores: &content.ores,
            default_planet_type: content.default_planet_type,
            climate_tags: &content.climate_tags,
            climate_curves: &content.climate_curves,
        }
    }

    pub fn planet_types(&self) -> impl Iterator<Item = PlanetTypeView<'_>> {
        self.planet_types
            .keys()
            .iter()
            .zip(self.planet_types.entries())
            .enumerate()
            .map(|(index, (key, data))| PlanetTypeView {
                id: PlanetTypeId::new(index as u32),
                key,
                data,
            })
    }

    pub fn biomes(&self) -> impl Iterator<Item = BiomeView<'_>> {
        self.biomes
            .keys()
            .iter()
            .zip(self.biomes.entries())
            .enumerate()
            .map(|(index, (key, data))| BiomeView {
                id: BiomeId::new(index as u32),
                key,
                data,
            })
    }

    pub fn flora(&self) -> impl Iterator<Item = FloraView<'_>> {
        self.flora
            .keys()
            .iter()
            .zip(self.flora.entries())
            .enumerate()
            .map(|(index, (key, data))| FloraView {
                id: FloraId::new(index as u32),
                key,
                data,
            })
    }

    pub fn ores(&self) -> impl Iterator<Item = OreView<'_>> {
        self.ores
            .keys()
            .iter()
            .zip(self.ores.entries())
            .enumerate()
            .map(|(index, (key, data))| OreView {
                id: OreId::new(index as u32),
                key,
                data,
            })
    }
}

impl PlanetTypeSource for WorldgenContentView<'_> {
    fn default_planet_type(&self) -> Option<PlanetTypeId> {
        self.default_planet_type
    }

    fn planet_type(&self, id: PlanetTypeId) -> Option<PlanetTypeView<'_>> {
        Some(PlanetTypeView {
            id,
            key: self.planet_types.key(id)?,
            data: self.planet_types.get(id)?,
        })
    }
}

impl BiomeSource for WorldgenContentView<'_> {
    fn biome(&self, id: BiomeId) -> Option<BiomeView<'_>> {
        Some(BiomeView {
            id,
            key: self.biomes.key(id)?,
            data: self.biomes.get(id)?,
        })
    }
}

impl FloraSource for WorldgenContentView<'_> {
    fn flora(&self, id: FloraId) -> Option<FloraView<'_>> {
        Some(FloraView {
            id,
            key: self.flora.key(id)?,
            data: self.flora.get(id)?,
        })
    }
}

impl OreSource for WorldgenContentView<'_> {
    fn ore(&self, id: OreId) -> Option<OreView<'_>> {
        Some(OreView {
            id,
            key: self.ores.key(id)?,
            data: self.ores.get(id)?,
        })
    }
}

impl WorldgenSettingsSource for WorldgenContentView<'_> {
    fn climate_tags(&self) -> &CompiledClimateTags {
        self.climate_tags
    }

    fn climate_curves(&self) -> &CompiledClimateCurves {
        self.climate_curves
    }
}

impl CompiledContent {
    pub fn to_block_content(&self) -> BlockContent {
        BlockContent::new(
            self.blocks.clone(),
            self.block_visuals.clone(),
            self.block_visual_palettes.clone(),
        )
    }

    pub fn into_block_content(self) -> BlockContent {
        BlockContent::new(self.blocks, self.block_visuals, self.block_visual_palettes)
    }

    pub fn block_content(&self) -> BlockContentView<'_> {
        BlockContentView::new(
            &self.blocks,
            &self.block_visuals,
            &self.block_visual_palettes,
        )
    }

    pub fn world_content(&self) -> WorldContentView<'_> {
        WorldContentView::new(&self.world)
    }

    pub fn worldgen_content(&self) -> WorldgenContentView<'_> {
        WorldgenContentView::new(self)
    }
}

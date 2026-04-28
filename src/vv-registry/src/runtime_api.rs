use crate::{
    BiomeId, BiomeRegistry, BlockId, BlockRegistry, CompiledBiome, CompiledBlockMining,
    CompiledBlockPhysics, CompiledBlockRender, CompiledClimateCurves, CompiledClimateTags,
    CompiledContent, CompiledDrops, CompiledPlanetType, CompiledWorldSettings, ContentKey,
    PlanetTypeId, PlanetTypeRegistry, TagId,
};

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

pub trait BlockRuntimeSource {
    fn block_runtime(&self, id: BlockId) -> Option<BlockRuntimeView<'_>>;
    fn block_key(&self, id: BlockId) -> Option<&ContentKey>;
}

pub trait BlockRenderSource {
    fn block_render(&self, id: BlockId) -> Option<&CompiledBlockRender>;
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

pub trait WorldgenSettingsSource {
    fn climate_tags(&self) -> &CompiledClimateTags;
    fn climate_curves(&self) -> &CompiledClimateCurves;
}

#[derive(Debug, Clone, Copy)]
pub struct BlockContentView<'a> {
    blocks: &'a BlockRegistry,
}

impl<'a> BlockContentView<'a> {
    pub(crate) fn new(blocks: &'a BlockRegistry) -> Self {
        Self { blocks }
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
    default_planet_type: Option<PlanetTypeId>,
    climate_tags: &'a CompiledClimateTags,
    climate_curves: &'a CompiledClimateCurves,
}

impl<'a> WorldgenContentView<'a> {
    pub(crate) fn new(content: &'a CompiledContent) -> Self {
        Self {
            planet_types: &content.planet_types,
            biomes: &content.biomes,
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

impl WorldgenSettingsSource for WorldgenContentView<'_> {
    fn climate_tags(&self) -> &CompiledClimateTags {
        self.climate_tags
    }

    fn climate_curves(&self) -> &CompiledClimateCurves {
        self.climate_curves
    }
}

impl CompiledContent {
    pub fn block_content(&self) -> BlockContentView<'_> {
        BlockContentView::new(&self.blocks)
    }

    pub fn world_content(&self) -> WorldContentView<'_> {
        WorldContentView::new(&self.world)
    }

    pub fn worldgen_content(&self) -> WorldgenContentView<'_> {
        WorldgenContentView::new(self)
    }
}

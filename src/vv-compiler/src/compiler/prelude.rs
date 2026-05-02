pub(super) use std::{collections::HashMap, path::Path, str::FromStr};

pub(super) use smallvec::SmallVec;

pub(super) use vv_pack::{load_packs_from_assets, PackLoadOrder, RawDocument};
pub(super) use vv_registry::*;

pub(super) use vv_schema::{
    block::{
        BlockDef, BlockGeometryProfile, BlockMaterialKind, BlockShape, BlockSurfaceProgramDef,
        MaterialPhase, RawBlockEnvironmentResponseDef, RawBlockFaceVisual, RawBlockVisualVariation,
        RenderMode, TintMode,
    },
    common::tool::ToolKind,
    common::{
        BlockRef, EntityRef, HexColor, ItemRef, LootTableRef, PlaceableRef, ResourceRef, TagRef,
    },
    item::ItemKind,
    loot::{DropSpec, LootTableDef},
    recipe::{RecipeDef, RecipeIngredient, RecipePattern},
    tag::{TagContentKind, TagDef},
    worldgen::{
        biome::BiomeDef,
        fauna::FaunaDef,
        flora::{FloraDef, FloraFeature, TreeArchetypeKind},
        ore::OreDef,
        planet::PlanetTypeDef,
        structure::StructureDef,
        weather::WeatherDef,
    },
};

pub(super) use crate::{
    diagnostics::{CompileDiagnostic, CompileError, CompileResult, ReferenceKind},
    identity::derive_key,
    reference_index::ReferenceIndex,
};

use super::prelude::*;

pub(super) fn compiled_ideal_range(range: vv_schema::common::IdealRange) -> CompiledIdealRange {
    CompiledIdealRange {
        min: range.min,
        ideal_min: range.ideal_min,
        ideal_max: range.ideal_max,
        max: range.max,
    }
}

pub(super) fn compiled_tool_kind(kind: ToolKind) -> CompiledToolKind {
    match kind {
        ToolKind::Hand => CompiledToolKind::Hand,
        ToolKind::Pickaxe => CompiledToolKind::Pickaxe,
        ToolKind::Axe => CompiledToolKind::Axe,
        ToolKind::Shovel => CompiledToolKind::Shovel,
        ToolKind::Sword => CompiledToolKind::Sword,
        ToolKind::Shears => CompiledToolKind::Shears,
        ToolKind::Hoe => CompiledToolKind::Hoe,
    }
}

pub(super) fn compiled_tint_mode(mode: &TintMode) -> CompiledTintMode {
    match mode {
        TintMode::None => CompiledTintMode::None,
        TintMode::GrassColor => CompiledTintMode::GrassColor,
        TintMode::FoliageColor => CompiledTintMode::FoliageColor,
        TintMode::WaterColor => CompiledTintMode::WaterColor,
    }
}

pub(super) fn compiled_render_mode(mode: &RenderMode) -> CompiledRenderMode {
    match mode {
        RenderMode::Opaque => CompiledRenderMode::Opaque,
        RenderMode::Cutout => CompiledRenderMode::Cutout,
        RenderMode::Transparent => CompiledRenderMode::Transparent,
        RenderMode::Additive => CompiledRenderMode::Additive,
    }
}

pub(super) fn compiled_block_shape(shape: &BlockShape) -> CompiledBlockShape {
    match shape {
        BlockShape::Cube => CompiledBlockShape::Cube,
        BlockShape::Cross => CompiledBlockShape::Cross,
        BlockShape::Fluid => CompiledBlockShape::Fluid,
        BlockShape::Custom { model } => CompiledBlockShape::Custom {
            model: model.0.clone(),
        },
    }
}

pub(super) fn geometry_profile_id(profile: &BlockGeometryProfile) -> u32 {
    match profile {
        BlockGeometryProfile::HardCube => 0,
        BlockGeometryProfile::SoftCube => 1,
        BlockGeometryProfile::PillowCube => 2,
        BlockGeometryProfile::ChunkyCube => 3,
        BlockGeometryProfile::NaturalRock => 4,
        BlockGeometryProfile::LeafMass => 5,
        BlockGeometryProfile::Crystal => 6,
        BlockGeometryProfile::LiquidSoft => 7,
    }
}

pub(super) fn surface_program_id(program: &BlockSurfaceProgramDef) -> u32 {
    match program {
        BlockSurfaceProgramDef::Flat => RUNTIME_SURFACE_PROGRAM_FLAT,
        BlockSurfaceProgramDef::Patterned(_) => RUNTIME_SURFACE_PROGRAM_PATTERNED,
    }
}

pub(super) fn parse_hex_color(value: &str) -> Option<[f32; 4]> {
    let hex = value.strip_prefix('#')?;
    let parse = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();

    match hex.len() {
        6 => Some([
            parse(0..2)? as f32 / 255.0,
            parse(2..4)? as f32 / 255.0,
            parse(4..6)? as f32 / 255.0,
            1.0,
        ]),
        8 => Some([
            parse(0..2)? as f32 / 255.0,
            parse(2..4)? as f32 / 255.0,
            parse(4..6)? as f32 / 255.0,
            parse(6..8)? as f32 / 255.0,
        ]),
        _ => None,
    }
}

pub(super) fn is_known_material_kind(name: &str) -> bool {
    matches!(
        name,
        "generic"
            | "stone"
            | "dirt"
            | "grass"
            | "sand"
            | "snow"
            | "wood"
            | "leaves"
            | "metal"
            | "glass"
            | "ice"
            | "liquid"
            | "emissive"
    )
}

pub(super) fn material_ref_from_kind(kind: &BlockMaterialKind) -> String {
    match kind {
        BlockMaterialKind::Generic => "voxelverse:generic".to_owned(),
        BlockMaterialKind::Stone => "voxelverse:stone".to_owned(),
        BlockMaterialKind::Dirt => "voxelverse:dirt".to_owned(),
        BlockMaterialKind::Grass => "voxelverse:grass".to_owned(),
        BlockMaterialKind::Sand => "voxelverse:sand".to_owned(),
        BlockMaterialKind::Snow => "voxelverse:snow".to_owned(),
        BlockMaterialKind::Wood => "voxelverse:wood".to_owned(),
        BlockMaterialKind::Leaves => "voxelverse:leaves".to_owned(),
        BlockMaterialKind::Metal => "voxelverse:metal".to_owned(),
        BlockMaterialKind::Glass => "voxelverse:glass".to_owned(),
        BlockMaterialKind::Ice => "voxelverse:ice".to_owned(),
        BlockMaterialKind::Liquid => "voxelverse:liquid".to_owned(),
        BlockMaterialKind::Emissive => "voxelverse:emissive".to_owned(),
        BlockMaterialKind::Custom { material } => material.0.clone(),
    }
}

pub(super) fn stable_hash32(value: &str) -> u32 {
    let mut hash = 0x811c_9dc5u32;

    for byte in value.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }

    hash
}

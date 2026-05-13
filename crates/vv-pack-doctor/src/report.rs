//! Report types produced by Pack Doctor.
//!
//! The report is intentionally plain: every field maps directly to a JSON
//! key. The HTML view also reads this same structure, so the JSON schema is
//! the contract.

use std::path::{Path, PathBuf};

use crate::parse::ParseError;
use crate::scan::PackScan;

#[derive(Debug, Clone)]
pub struct Report {
    pub pack_root: PathBuf,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub unused: Unused,
    pub missing: Missing,
    pub progression: Progression,
    pub planet: PlanetReport,
    pub summary: Summary,
    pub health_score: u32,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub check: &'static str,
    pub message: String,
    pub path: Option<String>,
    pub id: Option<String>,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn new(check: &'static str, message: impl Into<String>) -> Self {
        Self {
            check,
            message: message.into(),
            path: None,
            id: None,
            field: None,
            suggestion: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
    pub fn with_suggestion(mut self, s: impl Into<String>) -> Self {
        self.suggestion = Some(s.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct Unused {
    pub textures: Vec<String>,
    pub materials: Vec<String>,
    pub items: Vec<String>,
    pub blocks: Vec<String>,
    pub loot_tables: Vec<String>,
    pub voxels: Vec<String>,
    pub shaders: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Missing {
    pub block_items: Vec<String>,
    pub loot_tables: Vec<String>,
    pub textures: Vec<String>,
    pub voxels: Vec<String>,
    pub shaders: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Progression {
    pub basic_loop_reachable: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PlanetReport {
    pub counts: PlanetCounts,
    pub planet_profiles: Vec<PlanetProfileSummary>,
    pub biome_sets: Vec<BiomeSetSummary>,
    pub biomes: Vec<BiomeSummary>,
    pub features: Vec<FeatureSummary>,
    pub ores: Vec<OreSummary>,
    pub caves: Vec<CaveSummary>,
    pub render_profiles: Vec<RenderProfileSummary>,
    pub budget_notes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PlanetCounts {
    pub planet_profiles: usize,
    pub biome_sets: usize,
    pub biomes: usize,
    pub vegetation_rules: usize,
    pub prop_scatters: usize,
    pub ore_rules: usize,
    pub cave_rules: usize,
    pub render_profiles: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PlanetProfileSummary {
    pub id: String,
    pub display_name: String,
    pub near_voxel_lod_radius: u32,
    pub far_surface_lod_radius: u32,
    pub upload_budget_chunks_per_frame: u32,
    pub region_cell_voxels: u32,
    pub feature_budget_per_chunk: u32,
    pub vegetation_refs: usize,
    pub prop_scatter_refs: usize,
    pub ore_refs: usize,
    pub cave_refs: usize,
}

#[derive(Debug, Clone, Default)]
pub struct BiomeSetSummary {
    pub id: String,
    pub display_name: String,
    pub selectors: usize,
    pub blend_radius: f32,
}

#[derive(Debug, Clone, Default)]
pub struct BiomeSummary {
    pub id: String,
    pub display_name: String,
    pub surface_top: String,
    pub surface_under: String,
    pub amplitude: f32,
    pub flatness: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FeatureSummary {
    pub id: String,
    pub kind: String,
    pub density: f32,
    pub min_spacing_voxels: f32,
    pub variant_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct OreSummary {
    pub id: String,
    pub block: String,
    pub density: f32,
    pub depth_voxels: (u32, u32),
}

#[derive(Debug, Clone, Default)]
pub struct CaveSummary {
    pub id: String,
    pub fields: usize,
    pub depth_voxels: (u32, u32),
    pub tunnel_radius: (f32, f32),
    pub chamber_radius: (f32, f32),
}

#[derive(Debug, Clone, Default)]
pub struct RenderProfileSummary {
    pub id: String,
    pub label: String,
    pub quality_class: String,
    pub fog: bool,
    pub water: String,
    pub enabled_features: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub blocks: usize,
    pub items: usize,
    pub materials: usize,
    pub textures: usize,
    pub recipes: usize,
    pub loot_tables: usize,
    pub voxels: usize,
    pub shader_modules: usize,
    pub techniques: usize,
    pub world_files: usize,
    pub errors: usize,
    pub warnings: usize,
}

impl Report {
    pub fn new(pack_root: &Path) -> Self {
        Self {
            pack_root: pack_root.to_path_buf(),
            errors: Vec::new(),
            warnings: Vec::new(),
            unused: Unused::default(),
            missing: Missing::default(),
            progression: Progression::default(),
            planet: PlanetReport::default(),
            summary: Summary::default(),
            health_score: 0,
        }
    }

    pub fn error(&mut self, d: Diagnostic) {
        self.errors.push(d);
    }

    pub fn warn(&mut self, d: Diagnostic) {
        self.warnings.push(d);
    }

    pub fn error_simple(&mut self, check: &'static str, message: impl Into<String>) {
        self.errors.push(Diagnostic::new(check, message));
    }
    pub fn warn_simple(&mut self, check: &'static str, message: impl Into<String>) {
        self.warnings.push(Diagnostic::new(check, message));
    }

    pub fn add_parse_error(&mut self, e: &ParseError) {
        let location = if e.line > 0 {
            format!("{}:{}:{}", e.rel_path, e.line, e.column)
        } else {
            e.rel_path.clone()
        };
        let mut d = Diagnostic::new("parse", format!("{}: {}", location, e.message))
            .with_path(e.rel_path.clone());
        if let Some(s) = &e.suggestion {
            d = d.with_suggestion(s.clone());
        }
        self.errors.push(d);
    }

    pub fn finalize(&mut self, scan: &PackScan) {
        let obj_blocks = scan
            .objects
            .iter()
            .filter(|o| o.def.block.is_some())
            .count();
        let obj_items = scan.objects.len();
        let obj_recipes = scan
            .objects
            .iter()
            .map(|o| o.def.recipes.len())
            .sum::<usize>();
        self.summary.blocks = obj_blocks;
        self.summary.items = obj_items;
        self.summary.materials = 0;
        self.summary.textures = scan.texture_files.len();
        self.summary.recipes = obj_recipes;
        self.summary.loot_tables = scan.objects.iter().filter(|o| o.def.loot.is_some()).count();
        self.summary.voxels = scan.voxel_files.len();
        self.summary.shader_modules = scan.render.shader_modules.len();
        self.summary.techniques = scan.render.techniques.len();
        self.summary.world_files = scan.world_files.len();
        self.summary.errors = self.errors.len();
        self.summary.warnings = self.warnings.len();
        self.health_score = self.compute_health_score();
    }

    fn compute_health_score(&self) -> u32 {
        let penalty = self.summary.errors as i32 * 10 + self.summary.warnings as i32;
        let score = 100i32 - penalty;
        score.clamp(0, 100) as u32
    }

    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

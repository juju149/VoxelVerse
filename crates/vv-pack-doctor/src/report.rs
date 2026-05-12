//! Report types produced by Pack Doctor.
//!
//! The report is intentionally plain: every field maps directly to a JSON
//! key. The HTML view also reads this same structure, so the JSON schema is
//! the contract.

use std::path::{Path, PathBuf};

use crate::scan::PackScan;

#[derive(Debug, Clone)]
pub struct Report {
    pub pack_root: PathBuf,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub unused: Unused,
    pub missing: Missing,
    pub progression: Progression,
    pub summary: Summary,
    pub health_score: u32,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub check: &'static str,
    pub message: String,
    pub path: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Unused {
    pub textures: Vec<String>,
    pub materials: Vec<String>,
    pub items: Vec<String>,
    pub blocks: Vec<String>,
    pub loot_tables: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Missing {
    pub block_items: Vec<String>,
    pub loot_tables: Vec<String>,
    pub textures: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Progression {
    pub basic_loop_reachable: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub blocks: usize,
    pub items: usize,
    pub materials: usize,
    pub textures: usize,
    pub recipes: usize,
    pub loot_tables: usize,
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
            summary: Summary::default(),
            health_score: 0,
        }
    }

    pub fn error(&mut self, check: &'static str, message: impl Into<String>) {
        self.errors.push(Diagnostic {
            check,
            message: message.into(),
            path: None,
            id: None,
        });
    }

    pub fn error_path(
        &mut self,
        check: &'static str,
        message: impl Into<String>,
        path: impl Into<String>,
    ) {
        self.errors.push(Diagnostic {
            check,
            message: message.into(),
            path: Some(path.into()),
            id: None,
        });
    }

    pub fn error_id(
        &mut self,
        check: &'static str,
        message: impl Into<String>,
        id: impl Into<String>,
    ) {
        self.errors.push(Diagnostic {
            check,
            message: message.into(),
            path: None,
            id: Some(id.into()),
        });
    }

    pub fn warn(&mut self, check: &'static str, message: impl Into<String>) {
        self.warnings.push(Diagnostic {
            check,
            message: message.into(),
            path: None,
            id: None,
        });
    }

    pub fn warn_id(
        &mut self,
        check: &'static str,
        message: impl Into<String>,
        id: impl Into<String>,
    ) {
        self.warnings.push(Diagnostic {
            check,
            message: message.into(),
            path: None,
            id: Some(id.into()),
        });
    }

    pub fn warn_path(
        &mut self,
        check: &'static str,
        message: impl Into<String>,
        path: impl Into<String>,
    ) {
        self.warnings.push(Diagnostic {
            check,
            message: message.into(),
            path: Some(path.into()),
            id: None,
        });
    }

    pub fn finalize(&mut self, scan: &PackScan) {
        let obj_blocks = scan.objects.iter().filter(|(_, d)| d.block.is_some()).count();
        let obj_items = scan.objects.len();
        let obj_recipes = scan.objects.iter().filter(|(_, d)| d.recipe.is_some()).count();
        self.summary.blocks = obj_blocks;
        self.summary.items = obj_items;
        self.summary.materials = 0;
        self.summary.textures = scan.texture_files.len();
        self.summary.recipes = obj_recipes;
        self.summary.loot_tables = 0;
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

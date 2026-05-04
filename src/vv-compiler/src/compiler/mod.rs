mod prelude;

mod blocks;
mod content;
mod driver;
mod helpers;
mod loot;
mod model;
mod programs;
mod recipes;
mod refs;
mod tags;
mod validation;
mod values;
mod worldgen;

use prelude::*;

pub fn compile_packs(load_order: &PackLoadOrder) -> CompileResult<CompiledContent> {
    let mut compiler = ContentCompiler::default();
    compiler.compile(load_order)
}

pub fn compile_assets_root(assets_root: &Path) -> CompileResult<CompiledContent> {
    let load_order = load_packs_from_assets(assets_root).map_err(|err| {
        CompileError::new(vec![CompileDiagnostic::InvalidReference {
            owner: "pack_loader".to_owned(),
            path: assets_root.to_path_buf(),
            reference: assets_root.display().to_string(),
            expected: ReferenceKind::Pack,
            reason: err.to_string(),
        }])
    })?;

    compile_packs(&load_order)
}

#[derive(Default)]
pub(super) struct ContentCompiler {
    pub(super) diagnostics: Vec<CompileDiagnostic>,
}

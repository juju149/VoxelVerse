use std::{error::Error, fmt, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    Block,
    Item,
    Entity,
    Placeable,
    LootTable,
    Tag,
    Texture,
    PlanetType,
    Pack,
}

impl fmt::Display for ReferenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ReferenceKind::Block => "block",
            ReferenceKind::Item => "item",
            ReferenceKind::Entity => "entity",
            ReferenceKind::Placeable => "placeable",
            ReferenceKind::LootTable => "loot_table",
            ReferenceKind::Tag => "tag",
            ReferenceKind::Texture => "texture",
            ReferenceKind::PlanetType => "planet_type",
            ReferenceKind::Pack => "pack",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone)]
pub enum CompileDiagnostic {
    InvalidIdentity {
        path: PathBuf,
        reason: String,
    },
    DuplicateResource {
        key: String,
        first_path: PathBuf,
        second_path: PathBuf,
    },
    MissingReference {
        owner: String,
        path: PathBuf,
        reference: String,
        expected: ReferenceKind,
    },
    InvalidReference {
        owner: String,
        path: PathBuf,
        reference: String,
        expected: ReferenceKind,
        reason: String,
    },
}

impl fmt::Display for CompileDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileDiagnostic::InvalidIdentity { path, reason } => {
                write!(f, "invalid identity for `{}`: {reason}", path.display())
            }
            CompileDiagnostic::DuplicateResource {
                key,
                first_path,
                second_path,
            } => write!(
                f,
                "duplicate resource `{key}` at `{}` and `{}`",
                first_path.display(),
                second_path.display()
            ),
            CompileDiagnostic::MissingReference {
                owner,
                path,
                reference,
                expected,
            } => write!(
                f,
                "`{owner}` in `{}` references missing {expected} `{reference}`",
                path.display()
            ),
            CompileDiagnostic::InvalidReference {
                owner,
                path,
                reference,
                expected,
                reason,
            } => write!(
                f,
                "`{owner}` in `{}` has invalid {expected} reference `{reference}`: {reason}",
                path.display()
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileError {
    diagnostics: Vec<CompileDiagnostic>,
}

pub type CompileResult<T> = Result<T, CompileError>;

impl CompileError {
    pub fn new(diagnostics: Vec<CompileDiagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[CompileDiagnostic] {
        &self.diagnostics
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} content compile diagnostics", self.diagnostics.len())
    }
}

impl Error for CompileError {}

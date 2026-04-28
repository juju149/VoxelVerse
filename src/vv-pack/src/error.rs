use std::{error::Error, fmt, path::PathBuf};

#[derive(Debug)]
pub enum PackLoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Ron {
        path: PathBuf,
        source: ron::error::SpannedError,
    },
    MissingManifest {
        pack_dir: PathBuf,
    },
    InvalidPackDirectory {
        path: PathBuf,
    },
}

pub type PackLoadResult<T> = Result<T, PackLoadError>;

impl fmt::Display for PackLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackLoadError::Io { path, source } => {
                write!(f, "failed to read `{}`: {source}", path.display())
            }
            PackLoadError::Ron { path, source } => {
                write!(f, "failed to parse RON `{}`: {source}", path.display())
            }
            PackLoadError::MissingManifest { pack_dir } => {
                write!(f, "pack `{}` is missing pack.ron", pack_dir.display())
            }
            PackLoadError::InvalidPackDirectory { path } => {
                write!(f, "`{}` is not a pack directory", path.display())
            }
        }
    }
}

impl Error for PackLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PackLoadError::Io { source, .. } => Some(source),
            PackLoadError::Ron { source, .. } => Some(source),
            PackLoadError::MissingManifest { .. } | PackLoadError::InvalidPackDirectory { .. } => {
                None
            }
        }
    }
}

use std::path::{Path, PathBuf};

use vv_registry::ContentKey;

use crate::diagnostics::CompileDiagnostic;

pub fn derive_key(
    namespace: &str,
    relative_path: &Path,
    family_root: &str,
) -> Result<ContentKey, CompileDiagnostic> {
    let family = Path::new(family_root);
    let without_family = relative_path
        .strip_prefix(family)
        .map_err(|_| invalid(relative_path, "path is outside expected family root"))?;
    let without_extension = without_family
        .with_extension("")
        .components()
        .map(|component| component.as_os_str().to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>()
        .join("/");

    ContentKey::new(namespace, without_extension).map_err(|err| {
        CompileDiagnostic::InvalidIdentity {
            path: relative_path.to_path_buf(),
            reason: err.to_string(),
        }
    })
}

fn invalid(path: &Path, reason: impl Into<String>) -> CompileDiagnostic {
    CompileDiagnostic::InvalidIdentity {
        path: PathBuf::from(path),
        reason: reason.into(),
    }
}

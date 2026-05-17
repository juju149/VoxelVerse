use std::fs;
use std::path::Path;

#[test]
fn render_crate_does_not_depend_on_gameplay() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();
    assert!(
        !cargo_toml.contains("vv-gameplay"),
        "vv-render must not declare vv-gameplay as a dependency"
    );

    let src_dir = manifest_dir.join("src");
    for path in rust_files(&src_dir) {
        let source = fs::read_to_string(&path).unwrap();
        assert!(
            !source.contains("vv_gameplay"),
            "{} imports vv_gameplay",
            path.display()
        );
    }
}

fn rust_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

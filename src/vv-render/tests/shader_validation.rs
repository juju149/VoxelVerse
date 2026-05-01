#[test]
fn main_shader_is_valid_wgsl() {
    let source = include_str!("../src/shader.wgsl");
    let module = naga::front::wgsl::parse_str(source).expect("shader.wgsl should parse");
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .expect("shader.wgsl should validate");

    let entry_points = module
        .entry_points
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>();
    assert!(entry_points.contains(&"vs_main"));
    assert!(entry_points.contains(&"fs_main"));
    assert!(entry_points.contains(&"vs_sky"));
    assert!(entry_points.contains(&"fs_sky"));
}

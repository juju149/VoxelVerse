use std::path::PathBuf;

use vv_pack::load_packs_from_assets;

#[test]
fn loads_voxelverse_core_raw_documents() {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let load_order = load_packs_from_assets(&assets).expect("voxelverse_core should load");
    let pack = load_order
        .packs()
        .iter()
        .find(|pack| pack.content.manifest.namespace == "voxelverse")
        .expect("voxelverse pack should be discovered");

    assert_eq!(pack.content.blocks.len(), 17);
    assert_eq!(pack.content.items.len(), 16);
    assert_eq!(pack.content.recipes.len(), 8);
    assert_eq!(pack.content.loot_tables.len(), 0);
    assert_eq!(pack.content.biomes.len(), 4);
    assert_eq!(pack.content.flora.len(), 6);
    assert_eq!(pack.content.ores.len(), 3);
    assert_eq!(pack.content.structures.len(), 1);
    assert_eq!(pack.content.weather.len(), 1);
    assert!(!pack.content.tags.is_empty());
}

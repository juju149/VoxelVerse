# Modding V1

Start with [`PACK_V1.md`](PACK_V1.md). It is the authoritative contract.

Minimal workflow:

1. Create or edit one file under `assets/packs/<namespace>/defs/`.
2. Use path-derived ids; do not write duplicate ids inside files.
3. Use full references such as `core:object/terrain/stone`.
4. Use strict tags such as `#core:tag/material/wood`.
5. Give every visible item `category`, `visible_in_inventory`, and `inventory_icon`.
6. Reference voxel model manifests, never raw `.vox` paths.
7. Run Pack Doctor.

Validation:

```powershell
cargo run -p vv-pack-doctor -- assets/packs/core
cargo test -p vv-content-schema -p vv-pack-loader -p vv-pack-compiler -p vv-pack-doctor
```

Core V1 is valid only at `0 errors`, `0 warnings`, score `100/100`.

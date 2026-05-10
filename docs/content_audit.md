# VoxelVerse Content Audit

Date: 2026-05-10

Scope: `assets/packs/core`, with focus on the historical `voxel/` bank.
This is the pre-migration audit used to build the applied migration map.

## Current Pack Shape

Current top-level pack layout:

| Path | Role today | Runtime status |
| --- | --- | --- |
| `assets/packs/core/blocks/` | Raw block RON files | Loaded by `vv-pack-loader` |
| `assets/packs/core/worldgen/` | Raw procedural RON files | Loaded by `vv-pack-loader` |
| `assets/packs/core/textures/` | PNG block textures | Loaded by `TextureRegistry` |
| `assets/packs/core/voxel/` | Imported `.vox` bank and legacy manifests | Not loaded by current Rust runtime |
| `assets/packs/core/items/` | Present, not wired in inspected loader | Not loaded by current Rust runtime |
| `assets/packs/core/generated/` | Generated output area | Not a raw content source |
| `assets/packs/core/pack.toml` | Pack metadata | Present |

Observed file counts:

| Extension | Count |
| --- | ---: |
| `.vox` | 4504 |
| `.ron` | 133 |
| `.png` | 54 |
| `.md` | 1 |
| `.toml` | 1 |

## Historical `voxel/` Categories

| Directory | `.vox` count | Subdirectories | Problem |
| --- | ---: | ---: | --- |
| `armor/` | 315 | 48 | Equipment visuals mixed with item identity |
| `figure/` | 359 | 34 | Humanoid body visuals, not gameplay entities |
| `glider/` | 18 | 0 | Equipment category outside equipment tree |
| `item/` | 159 | 20 | Resources, food, crafting, consumables mixed |
| `lantern/` | 11 | 0 | Equipable light, separate from props/lights |
| `npc/` | 1886 | 449 | Animals, monsters, aquatic, flying, humanoids, bosses mixed |
| `object/` | 29 | 4 | Props, interactables and effects mixed |
| `sprite/` | 945 | 71 | Vegetation, props, containers, crops, resources mixed |
| `weapon/` | 779 | 117 | Weapons, tools, shields, projectiles mixed |

Top-level non-directory `.vox`:

| File | Proposed status |
| --- | --- |
| `char_template.vox` | `media/voxel/debug/char_template.vox` |
| `not_found.vox` | `media/voxel/debug/not_found.vox` |
| `particle.vox` | `media/voxel/effects/particle.vox` |

## Manifest Files

The current `voxel/` root contains legacy manifests such as:

| Manifest family | Meaning inferred | Proposed status |
| --- | --- | --- |
| `*_central_manifest.ron` | Modular body central parts | Quarantine in `legacy_imports/manifests/` until converted to `defs/skeletons/` and `defs/entities/` |
| `*_lateral_manifest.ron` | Modular body lateral parts | Quarantine in `legacy_imports/manifests/` until converted to `defs/skeletons/` and `defs/entities/` |
| `*_armor_*_manifest.ron` | Armor part placement | Quarantine, then convert to item/equipment visuals |
| `biped_weapon_manifest.ron` | Weapon attachment/visual map | Quarantine, then convert to item visuals |
| `item_drop_manifest.ron` | Item id to voxel visual map | Quarantine, then convert to `ItemDef.visual` |
| `object_manifest.ron` | Object visual map | Quarantine, then convert to `PropDef`/`EntityDef` |
| `sprite_manifest.ron` | Terrain sprite map | Quarantine, then convert to `VegetationDef`/`PropDef` |
| `tool_trail_manifest.ron` | Effect/trail visuals | Quarantine, then convert to effect definitions |

These manifests reference historical logical paths like `npc.duck.male.head`,
`voxel.sprite.flowers.sunflower_1`, and `common.items.*`. They are useful
evidence, but they are not the target registry format for VoxelVerse.

## Code References Found

Inspected files:

| File | Relevant behavior |
| --- | --- |
| `crates/vv-pack-loader/src/loader.rs` | Loads `.ron` from `blocks/` and `worldgen/*`; derives ids from pack namespace and filename stem |
| `crates/vv-pack-compiler/src/compiler.rs` | Compiles block defs into `BlockRegistry`; block visuals inline texture refs |
| `crates/vv-pack-compiler/src/texture_registry.rs` | Resolves texture refs to `assets/packs/<namespace>/textures/<path>.png` |
| `crates/vv-content-schema/src/block.rs` | Current `RawBlockDef` schema |
| `crates/vv-content-schema/src/visual.rs` | Current inline material texture set schema |
| `apps/voxelverse-game/src/app/content_bootstrap.rs` | Loads `assets/packs/core` through the loader/compiler |

Important current constraints:

- Moving `blocks/`, `worldgen/`, or `textures/` now would break runtime tests unless the loader is changed in the same step.
- Moving `voxel/` does not currently break inspected runtime code, but can break future conversion scripts or legacy manifests if references are not preserved in a map.
- Current block visuals reference texture PNGs directly through inline `RawMaterialTextureSet`; the target architecture should introduce `defs/materials/` before moving block texture refs to material ids.

## Referenced vs Orphaned Assets

Current Rust runtime:

- Does not load `.vox` files directly.
- Does not read `voxel/*_manifest.ron`.
- Does not compile item, entity, prop, vegetation model definitions yet.

Legacy manifests:

- Reference many `.vox` paths indirectly through historical dotted paths.
- Use historical domains like `voxel.sprite.*`, `voxel.item.*`, `voxel.weapon.*`, and `npc.*`.
- Need a conversion pass before any deletion decision.

Conclusion:

- No `.vox` should be deleted during this migration.
- `.vox` files not referenced by Rust runtime are not automatically unused.
- Top-level debug/admin candidates should be moved to target `debug/` or marked `needs_manual_review`, not deleted.

## Duplicate Candidates

Hash scan found duplicate `.vox` bytes. Examples:

| Hash group | Files | Current decision |
| --- | --- | --- |
| `5141D701...` | `crawler_moss/male/leg_bcr.vox`, `leg_fcr.vox`, `leg_fr.vox` | Keep; mirrored limb parts can be intentional |
| `48584E14...` | `emberfly/male/leg_bcr.vox`, `leg_fcr.vox`, `leg_fr.vox` | Keep; mirrored limb parts can be intentional |
| `5E9F0B35...` | `crawler_sand/male/leg_bcr.vox`, `leg_fcr.vox`, `leg_fr.vox` | Keep; mirrored limb parts can be intentional |
| `991F55BD...` | `sprite/crystal/ice_crystal_3.vox`, `ice_crystal_5.vox` | Delete candidate only after visual review |

No duplicate `.vox` deletion is approved by this audit.

## Debug/Admin Candidates

Observed candidates:

| Path | Proposed action |
| --- | --- |
| `voxel/char_template.vox` | Move to `media/voxel/debug/` |
| `voxel/not_found.vox` | Move to `media/voxel/debug/` |
| `voxel/particle.vox` | Move to `media/voxel/effects/` |
| `voxel/armor/tabard_admin.vox` | Move to `media/voxel/debug/admin/` or `equipment/armor/needs_review/` |
| `voxel/armor/misc/back/admin.vox` | Move to `media/voxel/debug/admin/` |
| `voxel/armor/misc/bag/admin_black_hole.vox` | Move to `media/voxel/debug/admin/` |
| `voxel/weapon/debug_wand*.vox` | Move to `media/voxel/debug/tools/` |

These are not deletion candidates yet because legacy manifests reference admin/debug entries.

## Migration Risks

| Risk | Mitigation |
| --- | --- |
| Manifest paths use historical dotted ids | Quarantine manifests and generate old -> new mapping CSV |
| `npc/` contains many biological categories | Conservative category rules; uncertain files go to `needs_review` |
| Filename normalization can collide | Script detects destination collisions and refuses overwrite |
| Current loader expects `blocks/`, `worldgen/`, `textures/` | Do not move these until loader/schema migration is implemented |
| Empty target dirs are not tracked by Git | Migration script creates dirs at runtime |
| `.ron` parser not available in PowerShell | Optional Rust content tests remain the strict RON parser |

## Proposed Directory Mapping

| Old path | New path | Action | Confidence |
| --- | --- | --- | --- |
| `voxel/*_manifest.ron` | `legacy_imports/manifests/` | quarantine | high |
| `voxel/armor/**` | `media/voxel/equipment/armor/**` | move | high |
| `voxel/glider/**` | `media/voxel/equipment/gliders/**` | move | high |
| `voxel/lantern/**` | `media/voxel/equipment/accessories/lanterns/**` | move | medium |
| `voxel/weapon/projectile/**` | `media/voxel/projectiles/**` | move | high |
| `voxel/weapon/shield/**` | `media/voxel/equipment/shields/**` | move | high |
| `voxel/weapon/tool/**` | `media/voxel/equipment/tools/**` | move | high |
| `voxel/weapon/**` | `media/voxel/equipment/weapons/**` | move | medium |
| `voxel/item/food/**` | `media/voxel/items/food/**` | move | high |
| `voxel/item/consumable/**` | `media/voxel/items/consumables/**` | move | high |
| `voxel/item/crafting/**` | `media/voxel/items/crafting/**` | move | high |
| `voxel/item/**` | `media/voxel/items/needs_review/**` | move | medium |
| `voxel/figure/**` | `media/voxel/characters/humanoids/**` | move | medium |
| `voxel/npc/**` | `media/voxel/creatures/needs_review/**` | move | low until entity taxonomy exists |
| `voxel/object/**` | `media/voxel/props/interactables/**` | move | medium |
| `voxel/sprite/chests/**` | `media/voxel/props/containers/**` | move | high |
| `voxel/sprite/furniture/**` | `media/voxel/props/furniture/**` | move | high |
| `voxel/sprite/crafting_station/**` | `media/voxel/props/crafting_stations/**` | move | high |
| `voxel/sprite/door/**` | `media/voxel/props/doors/**` | move | high |
| `voxel/sprite/lantern/**` | `media/voxel/props/lights/**` | move | high |
| `voxel/sprite/grass/**` | `media/voxel/vegetation/grass/**` | move | high |
| `voxel/sprite/flowers/**` | `media/voxel/vegetation/flowers/**` | move | high |
| `voxel/sprite/mushrooms/**` | `media/voxel/vegetation/mushrooms/**` | move | high |
| `voxel/sprite/underwater_*` | `media/voxel/vegetation/underwater/**` | move | high |
| `voxel/sprite/*` uncertain | `legacy_imports/needs_review/voxel/sprite/**` | quarantine | medium |

The executable mapping lives in `tools/migrate_voxel_assets.ps1`.

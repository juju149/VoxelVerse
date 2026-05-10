# Voxel Asset Migration Map

This map defines the first migration pass from the historical `voxel/` bank to
the target `media/voxel/` architecture.

The executable version is `tools/migrate_voxel_assets.ps1`. It generates an
exhaustive CSV report at:

```text
assets/packs/core/generated/diagnostics/voxel_asset_migration_map.csv
```

The script is dry-run by default. Use `-Apply` only after reviewing the CSV and
collision report.

## Directory Rules

| old_path | new_path | category | action | confidence | notes |
| --- | --- | --- | --- | --- | --- |
| `assets/packs/core/voxel/*_manifest.ron` | `assets/packs/core/legacy_imports/manifests/*_manifest.ron` | legacy_manifest | quarantine | high | Preserve old assembly evidence |
| `assets/packs/core/voxel/README.md` | `assets/packs/core/legacy_imports/manifests/README.voxel_legacy.md` | legacy_manifest | quarantine | high | Historical organization note |
| `assets/packs/core/voxel/char_template.vox` | `assets/packs/core/media/voxel/debug/char_template.vox` | debug | move | high | Authoring/template asset |
| `assets/packs/core/voxel/not_found.vox` | `assets/packs/core/media/voxel/debug/not_found.vox` | debug | move | high | Runtime/editor fallback visual |
| `assets/packs/core/voxel/particle.vox` | `assets/packs/core/media/voxel/effects/particle.vox` | effect | move | high | Effect media |
| `assets/packs/core/voxel/armor/**` | `assets/packs/core/media/voxel/equipment/armor/**` | equipment_armor | move | high | Armor visuals only |
| `assets/packs/core/voxel/glider/**` | `assets/packs/core/media/voxel/equipment/gliders/**` | equipment_glider | move | high | Equipable glider visuals |
| `assets/packs/core/voxel/lantern/**` | `assets/packs/core/media/voxel/equipment/accessories/lanterns/**` | equipment_accessory | move | medium | Equipable lanterns; prop lanterns stay under props/lights |
| `assets/packs/core/voxel/figure/**` | `assets/packs/core/media/voxel/characters/humanoids/**` | character_humanoid | move | medium | Historical humanoid body visuals |
| `assets/packs/core/voxel/item/food/**` | `assets/packs/core/media/voxel/items/food/**` | item_food | move | high | Item model |
| `assets/packs/core/voxel/item/consumable/**` | `assets/packs/core/media/voxel/items/consumables/**` | item_consumable | move | high | Item model |
| `assets/packs/core/voxel/item/crafting/**` | `assets/packs/core/media/voxel/items/crafting/**` | item_crafting | move | high | Item model |
| `assets/packs/core/voxel/item/mineral/**` | `assets/packs/core/media/voxel/items/resources/mineral/**` | item_resource | move | high | Resource item model |
| `assets/packs/core/voxel/item/**` | `assets/packs/core/media/voxel/items/needs_review/**` | item_needs_review | move | medium | Needs item taxonomy |
| `assets/packs/core/voxel/weapon/projectile/**` | `assets/packs/core/media/voxel/projectiles/**` | projectile | move | high | Projectile visuals |
| `assets/packs/core/voxel/weapon/shield/**` | `assets/packs/core/media/voxel/equipment/shields/**` | equipment_shield | move | high | Shield visuals |
| `assets/packs/core/voxel/weapon/tool/**` | `assets/packs/core/media/voxel/equipment/tools/**` | equipment_tool | move | high | Tool visuals |
| `assets/packs/core/voxel/weapon/**` | `assets/packs/core/media/voxel/equipment/weapons/**` | equipment_weapon | move | medium | Weapons/components need later item defs |
| `assets/packs/core/voxel/object/**` | `assets/packs/core/media/voxel/props/interactables/**` | prop_interactable | move | medium | Props need `PropDef` later |
| `assets/packs/core/voxel/npc/**` | `assets/packs/core/media/voxel/creatures/needs_review/**` | creature_needs_review | move | low | Requires entity taxonomy pass |
| `assets/packs/core/voxel/sprite/chests/**` | `assets/packs/core/media/voxel/props/containers/**` | prop_container | move | high | Chest/container visual |
| `assets/packs/core/voxel/sprite/underwater_chests/**` | `assets/packs/core/media/voxel/props/containers/underwater/**` | prop_container | move | high | Chest/container visual |
| `assets/packs/core/voxel/sprite/furniture/**` | `assets/packs/core/media/voxel/props/furniture/**` | prop_furniture | move | high | Furniture visual |
| `assets/packs/core/voxel/sprite/crafting_station/**` | `assets/packs/core/media/voxel/props/crafting_stations/**` | prop_crafting_station | move | high | Station visual |
| `assets/packs/core/voxel/sprite/door/**` | `assets/packs/core/media/voxel/props/doors/**` | prop_door | move | high | Door visual |
| `assets/packs/core/voxel/sprite/window/**` | `assets/packs/core/media/voxel/props/structure_parts/windows/**` | prop_structure_part | move | high | Structure visual |
| `assets/packs/core/voxel/sprite/lantern/**` | `assets/packs/core/media/voxel/props/lights/**` | prop_light | move | high | Placed light visual |
| `assets/packs/core/voxel/sprite/camp/**` | `assets/packs/core/media/voxel/props/lights/camp/**` | prop_light | move | medium | Contains campfire-like assets |
| `assets/packs/core/voxel/sprite/sign/**` | `assets/packs/core/media/voxel/props/interactables/signs/**` | prop_interactable | move | high | Sign visual |
| `assets/packs/core/voxel/sprite/barricades_wood/**` | `assets/packs/core/media/voxel/props/structure_parts/barricades_wood/**` | prop_structure_part | move | high | Structure visual |
| `assets/packs/core/voxel/sprite/bars/**` | `assets/packs/core/media/voxel/props/structure_parts/bars/**` | prop_structure_part | move | high | Structure visual |
| `assets/packs/core/voxel/sprite/castle/**` | `assets/packs/core/media/voxel/props/structure_parts/castle/**` | prop_structure_part | move | medium | Structure visual |
| `assets/packs/core/voxel/sprite/grave/**` | `assets/packs/core/media/voxel/props/decoration/grave/**` | prop_decoration | move | high | Decorative prop |
| `assets/packs/core/voxel/sprite/grass/**` | `assets/packs/core/media/voxel/vegetation/grass/**` | vegetation_grass | move | high | Vegetation model |
| `assets/packs/core/voxel/sprite/flowers/**` | `assets/packs/core/media/voxel/vegetation/flowers/**` | vegetation_flower | move | high | Vegetation model |
| `assets/packs/core/voxel/sprite/mushrooms/**` | `assets/packs/core/media/voxel/vegetation/mushrooms/**` | vegetation_mushroom | move | high | Vegetation model |
| `assets/packs/core/voxel/sprite/cacti/**` | `assets/packs/core/media/voxel/vegetation/desert/cacti/**` | vegetation_desert | move | high | Vegetation model |
| `assets/packs/core/voxel/sprite/dead_bush/**` | `assets/packs/core/media/voxel/vegetation/desert/dead_bush/**` | vegetation_desert | move | high | Vegetation model |
| `assets/packs/core/voxel/sprite/snow_bush/**` | `assets/packs/core/media/voxel/vegetation/snow/bushes/**` | vegetation_snow | move | high | Vegetation model |
| `assets/packs/core/voxel/sprite/underwater_*` | `assets/packs/core/media/voxel/vegetation/underwater/**` | vegetation_underwater | move | high | Vegetation model |
| `assets/packs/core/voxel/sprite/carrot/**` | `assets/packs/core/media/voxel/vegetation/crops/carrot/**` | vegetation_crop | move | high | Crop model |
| `assets/packs/core/voxel/sprite/corn/**` | `assets/packs/core/media/voxel/vegetation/crops/corn/**` | vegetation_crop | move | high | Crop model |
| `assets/packs/core/voxel/sprite/pumpkin/**` | `assets/packs/core/media/voxel/vegetation/crops/pumpkin/**` | vegetation_crop | move | high | Crop model |
| `assets/packs/core/voxel/sprite/tomato/**` | `assets/packs/core/media/voxel/vegetation/crops/tomato/**` | vegetation_crop | move | high | Crop model |
| `assets/packs/core/voxel/sprite/wheat_*` | `assets/packs/core/media/voxel/vegetation/crops/wheat/**` | vegetation_crop | move | high | Crop model |
| `assets/packs/core/voxel/sprite/potion/**` | `assets/packs/core/media/voxel/items/consumables/potions/**` | item_consumable | move | medium | Some potion visuals may be props |
| `assets/packs/core/voxel/sprite/mineral/**` | `assets/packs/core/media/voxel/items/resources/mineral/**` | item_resource | move | medium | Some mineral visuals may be world props |
| `assets/packs/core/voxel/sprite/rocks/**` | `assets/packs/core/media/voxel/props/decoration/rocks/**` | prop_decoration | move | medium | Some may become crafting items |
| `assets/packs/core/voxel/sprite/*` | `assets/packs/core/legacy_imports/needs_review/voxel/sprite/*` | needs_review | quarantine | medium | Avoid blind classification |

## Active Runtime Paths Not Moved In This Pass

| path | reason |
| --- | --- |
| `assets/packs/core/blocks/` | Current loader expects this path |
| `assets/packs/core/worldgen/` | Current loader expects this path |
| `assets/packs/core/textures/` | Current texture registry expects this path |

These should move to `defs/` and `media/textures/` only with matching Rust
loader/schema changes.

## Review Gates Before `-Apply`

1. Run `powershell -ExecutionPolicy Bypass -File tools/migrate_voxel_assets.ps1`.
2. Review `generated/diagnostics/voxel_asset_migration_map.csv`.
3. Review `generated/diagnostics/voxel_asset_migration_collisions.csv`.
4. Fix mapping rules until collisions are zero or intentional.
5. Run `powershell -ExecutionPolicy Bypass -File tools/validate_content.ps1`.
6. Apply with `powershell -ExecutionPolicy Bypass -File tools/migrate_voxel_assets.ps1 -Apply`.
7. Run validation again.

## Applied Result

Applied on 2026-05-10.

| Check | Result |
| --- | ---: |
| Migration rows | 4555 |
| Destination collisions | 0 |
| Moved entries | 4555 |
| Total `.vox` files preserved under `assets/packs/core` | 4504 |
| `.vox` files under `media/voxel` | 4338 |
| `.vox` files quarantined in `legacy_imports/needs_review` | 166 |
| Legacy manifests quarantined in `legacy_imports/manifests` | 50 |
| Files left under old `voxel/` root | 0 |
| Post-migration numeric filename normalizations | 70 |

Additional generated diagnostics:

```text
assets/packs/core/generated/diagnostics/voxel_asset_migration_map.csv
assets/packs/core/generated/diagnostics/voxel_asset_migration_collisions.csv
assets/packs/core/generated/diagnostics/voxel_asset_migration_moves.csv
assets/packs/core/generated/diagnostics/voxel_asset_name_normalization_moves.csv
```

Validation status:

- `tools/validate_content.ps1`: passed.
- `cargo test -p vv-pack-compiler`: passed, 4 tests.
- Remaining warning: legacy manifests are intentionally quarantined but not yet
  converted to `defs/skeletons`, `defs/entities`, `defs/items`, or `defs/props`.

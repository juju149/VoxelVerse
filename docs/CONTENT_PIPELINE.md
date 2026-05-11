# VoxelVerse - Content Pipeline

This document describes the **only** recommended workflow for producing new
content in VoxelVerse. It is meant to be followed step-by-step by humans and AI
agents.

The runtime never reads raw files. The pipeline is:

```
brief -> raw .ron + media -> validate -> compile -> runtime registries -> game
                              ^             ^
                              |             +-- vv-pack-compiler
                              +-- vv-pack-doctor + validate_content.ps1
```

The validators are the gate. Nothing reaches the runtime without passing.

For the strict content-quality rules enforced by Pack Doctor, see
[`content_rules.md`](content_rules.md).

---

## 0. Folder cheat sheet

```
assets/packs/core/
  pack.ron                       <-- manifest, namespace, format version
  defs/                          <-- THE TRUTH of all content
    blocks/                      block definitions
    block_models/                block geometry
    materials/                   material -> textures
    items/                       items grouped by bucket
    loot/                        loot tables
    recipes/                     recipes by station
    tags/                        tag declarations
    worldgen/                    biomes, ores, terrain, vegetation, etc.
    entities/                    creatures, NPCs, props
    sounds/                      sound events
    skeletons/                   rigging
    props/                       prop collections
    vegetation/                  vegetation catalogs
  media/                         <-- RAW ASSETS
    textures/                    PNGs, organized by category/material
    voxel/                       .vox files, organized by role
  generated/                     <-- REBUILDABLE artifacts (never hand-edited)
    registries/                  e.g. voxel_assets.ron
    reports/                     pack doctor reports (JSON + HTML)
  render/                        <-- render pipeline data-driven definitions
  source/                        <-- HUMAN/AI MATERIAL (never read by runtime)
    production/                  briefs, checklists, MVP plans
      checklists/                actionable per-asset checklists
      allowed_unused.ron         intentional exceptions
```

Anything under `source/production/` is documentation and tooling input. The
loader does not look at it.

---

## 1. Identity

Identity is path-as-identity inside a pack namespace. Raw `.ron` files must
**not** repeat their own ID when the namespace and path already define it.

Examples:

```
defs/blocks/terrain/grass.block.ron
  -> core:block/terrain/grass

defs/materials/blocks/grass_block/grass_block_top.material.ron
  -> core:material/blocks/grass_block/grass_block_top
```

The mapping table in [`content_rules.md`](content_rules.md#3-path-derived-identity)
is the authoritative source.

---

## 2. Pipeline for a new block

Every block goes through these twelve steps. Skip none.

1. **Brief**. Write a short brief in
   `assets/packs/core/source/production/briefs/<stem>.md`. Describe the role,
   biome, palette, gameplay intent.
2. **Textures**. Produce `albedo`, `normal`, `roughness` (256x256 PNG) under
   `media/textures/blocks/<material_stem>/<material_stem>_<face>_<map>.png`.
3. **Materials**. Author one or more `.material.ron` files in
   `defs/materials/blocks/<material_stem>/`.
4. **Block model**. Reuse an existing model under `defs/block_models/`, or
   create a new one only if no existing model fits.
5. **Block definition**. Author `defs/blocks/<category>/<stem>.block.ron`,
   wiring `model`, `visual.materials`, `physical`, `gameplay`, `audio`, and
   `tags`.
6. **Loot table**. Author `defs/loot/blocks/<stem>.loot.ron` or reuse
   `core:loot/blocks/empty`.
7. **Item**. If the block is placeable, author
   `defs/items/blocks/<stem>.item.ron` with
   `gameplay: PlaceBlock("core:block/<...>/<stem>")`.
8. **Tags**. Add or declare any new tags in `defs/tags/blocks/`.
9. **Recipe**. If the block is craftable, author the recipe in
   `defs/recipes/<station>/<stem>.recipe.ron`.
10. **Worldgen**. If the block is natural, add it to a terrain layer, ore,
    cave, vegetation, or structure file under `defs/worldgen/`.
11. **Validate** with the three commands in [section 6](#6-validation-commands).
12. **In-game smoke test**. Launch the game, place the block, mine it back,
    confirm drops, texture, lighting, and tags behave as expected.

Use [`block_checklist.md`](../assets/packs/core/source/production/checklists/block_checklist.md).

---

## 3. Pipeline for a new item

1. **Brief** in `source/production/briefs/`.
2. **Inventory icon** (or world model for placeable items).
3. **Item definition** in `defs/items/<bucket>/<stem>.item.ron`.
4. **Tags**.
5. **Source recipe** if craftable.
6. **Usage recipe** if it is an ingredient.
7. **Loot / source** if it is found in the world.
8. **Validate** with Pack Doctor.

See [`item_checklist.md`](../assets/packs/core/source/production/checklists/item_checklist.md).

---

## 4. Pipeline for a new recipe

1. Pick the **station** (`crafting`, `smelting`, ...).
2. Define the **inputs** as existing item IDs.
3. Define the **outputs** as existing item IDs.
4. Set the **time / cost** if the station supports it.
5. Pick the **category** (used by future UI sorting).
6. **Reachability check**: every input must be obtainable through worldgen,
   loot, or another recipe whose inputs are reachable.
7. **Usefulness check**: the output must feed a real downstream use, or be a
   terminal craft (e.g. a tool, a structural block).
8. **Validate**.

See [`recipe_checklist.md`](../assets/packs/core/source/production/checklists/recipe_checklist.md).

---

## 5. Pipeline for a new texture

1. **Prompt or visual brief**. Optional, but strongly recommended for
   AI-driven textures.
2. **PNG 256x256** for block surfaces.
3. **Clear name**: `<material_stem>_<face>_<map>.png` where `<map>` is
   `albedo`, `normal`, or `roughness`.
4. **Consistency**: albedo, normal, and roughness obviously depict the same
   surface.
5. **Materials reference**. A texture has no value until a material points to
   it.
6. **Dimension check** via Pack Doctor.
7. **Usage check** via Pack Doctor.

See [`texture_checklist.md`](../assets/packs/core/source/production/checklists/texture_checklist.md).

---

## 6. Validation commands

The canonical commands to run before claiming a content task done:

```powershell
powershell -ExecutionPolicy Bypass -File tools\validate_content.ps1
powershell -ExecutionPolicy Bypass -File tools\pack_doctor.ps1
cargo test -p vv-pack-loader -p vv-pack-compiler
```

- `validate_content.ps1` is the legacy filesystem/reference validator. It
  remains the fastest first gate.
- `pack_doctor.ps1` runs the deeper Rust validator (`vv-pack-doctor`), which
  produces `generated/reports/core_pack_report.json` and a sibling HTML
  report.
- The `cargo test` line confirms the pack still parses and compiles into
  runtime registries.

All three must pass for a content change to be considered done.

---

## 7. Where does it go?

| Producing                    | Goes to                                            |
| ---------------------------- | -------------------------------------------------- |
| A block design idea          | `source/production/briefs/<stem>.md`               |
| A texture brief / prompt     | `source/production/briefs/<stem>.md`               |
| A texture (PNG)              | `media/textures/blocks/<material_stem>/...`        |
| A material definition        | `defs/materials/<category>/<stem>.material.ron`    |
| A block definition           | `defs/blocks/<category>/<stem>.block.ron`          |
| An item definition           | `defs/items/<bucket>/<stem>.item.ron`              |
| A loot table                 | `defs/loot/<bucket>/<stem>.loot.ron`               |
| A recipe                     | `defs/recipes/<station>/<stem>.recipe.ron`         |
| A worldgen entry             | `defs/worldgen/<bucket>/<stem>.<kind>.ron`         |
| A generated registry         | `generated/registries/...`                         |
| A pack health report         | `generated/reports/core_pack_report.{json,html}`   |
| An allowed-unused exception  | `source/production/allowed_unused.ron`             |
| A future-feature placeholder | `source/production/draft/`                         |

If a piece of content does not fit any of these slots, **stop** and update this
document - do not invent a new slot ad hoc.

---

## 8. Runtime rules (unchanged)

- Runtime code must not parse raw media files directly.
- Runtime code consumes generated registries.
- Voxels store compact runtime IDs, never strings.
- Blocks reference materials. Materials reference textures.
- Media files never define gameplay.

# VoxelVerse - Content Rules

This document is the **single source of truth** for content production rules in
VoxelVerse. It applies to humans and AI agents alike. Every `.ron`, every
texture, every entry under `assets/packs/core/` must obey these rules.

If a rule conflicts with `AGENTS.md`, `AGENTS.md` wins. If a rule conflicts with
code currently in `vv-content-schema`, **fix the data**, not the schema, unless
the schema is actually wrong.

---

## 1. Layered ownership

| Layer                       | What lives there                                                  | What never lives there                  |
| --------------------------- | ----------------------------------------------------------------- | --------------------------------------- |
| `.ron` (defs/, render/)     | The truth of the content: blocks, items, materials, recipes, etc. | Build artifacts, prompts, planning      |
| `media/`                    | Raw assets: PNG textures, `.vox` models                           | Anything authored by code, JSON reports |
| `generated/`                | Rebuildable derived data (registries, reports)                    | Hand-authored content                   |
| Rust (`crates/`, `apps/`)   | Systems, schemas, validators, compilers, runtime registries       | Hardcoded blocks, items, recipe lists   |
| `source/production/`        | Briefs, checklists, human/AI follow-ups, allowed-unused notes     | Anything the runtime reads              |

The runtime **never** reads `source/production/`. The runtime **never** reads
raw RON files directly; it consumes compiled registries.

---

## 2. Naming

These rules are enforced by `tools/validate_content.ps1` and `vv-pack-doctor`.

Required:

- `lowercase` only.
- `snake_case` only.
- No spaces.
- No hyphens.
- No bare numeric filenames (`00.vox` is forbidden, `flower_blue_00.vox` is fine).
- Filenames must describe the role, not the order in which they were created.

Forbidden (in filenames, IDs, or display names that drive IDs):

- `test`, `new`, `final`, `temp`, `tmp`, `misc_thing`, `stuff`, `placeholder`.
- The word `misc` is tolerated as a catalog folder, but not as an item stem.
- Numeric-only stems.
- Trailing version suffixes like `_v2`, `_final_final`.

Prefer precise names:

- `oak_log`, not `wood2`.
- `rough_stone`, not `stone_b`.
- `iron_ore`, not `ore_new`.
- `construction_table`, not `bench`.

---

## 3. Path-derived identity

The ID of any definition is **derived from its path**. A file must not repeat
its own stable ID inside its body. The loader rules:

```
defs/blocks/<dirs>/<stem>.block.ron        -> core:block/<dirs>/<stem>
defs/block_models/<dirs>/<stem>.ron        -> core:block_model/<dirs>/<stem>
defs/materials/<dirs>/<stem>.material.ron  -> core:material/<dirs>/<stem>
defs/items/<bucket>/<stem>.item.ron        -> core:item/<bucket_singular>/<stem>
defs/loot/<dirs>/<stem>.loot.ron           -> core:loot/<dirs>/<stem>
defs/recipes/<dirs>/<stem>.recipe.ron      -> core:recipe/<dirs>/<stem>
defs/tags/<dirs>/<stem>.ron                -> core:tags/<stem>
defs/worldgen/<bucket>/<stem>.<kind>.ron   -> core:<domain>/<stem>
```

Item bucket singularization is hard-coded in the loader:
`blocks->block`, `resources->resource`, `tools->tool`, `weapons->weapon`,
`consumables->consumable`.

Never invent a new bucket without first updating the loader.

---

## 4. Blocks

Every block file must:

- Live under `defs/blocks/<category>/<stem>.block.ron`.
- Use `BlockDef(...)` with the current `format_version`.
- Reference an existing block model in `defs/block_models/`.
- Reference existing materials in `defs/materials/`.
- Reference a real loot table in `defs/loot/blocks/` (use `core:loot/blocks/empty` for non-droppable blocks).
- Use a tag list that is restricted to declared tags.

Every **placeable** block must:

- Have a matching item under `defs/items/blocks/<stem>.item.ron` whose
  `gameplay: PlaceBlock("core:block/<...>/<stem>")` points back at it.
- Be reachable in worldgen, recipes, or loot - **or** be listed in
  `source/production/allowed_unused.ron`.

Slot keys in `visual.materials` must match what the block model expects
(e.g. `top`, `bottom`, `side`, `end`, `all`).

---

## 5. Items

Every item must:

- Live in the right bucket (`blocks/`, `resources/`, `tools/`, `weapons/`,
  `consumables/`, `food/`, `misc/`).
- Have a clear `category` and `gameplay` intent.
- Have a stack size in `[1, 99]` unless documented otherwise.
- Use `gameplay: PlaceBlock("core:block/...")` only when the target block exists
  and is placeable.

Tools (`tools/`, `weapons/`) must declare:

- Tool tier or weapon damage.
- Durability if applicable.
- Mining speed if applicable.

Resource items (`resources/`, `food/`, `consumables/`, `misc/`) should be
reachable through at least one of:

- A recipe output.
- A loot table.
- A worldgen drop.
- An explicit listing in `source/production/allowed_unused.ron`.

---

## 6. Materials

Every material must:

- Live under `defs/materials/<category>/<stem>.material.ron`.
- Reference an albedo texture that exists on disk.
- Either reference a normal and roughness texture **or** intentionally omit them
  (the `Option`s in `RawMaterialDef` make this explicit).
- Declare a `category` and a `sampling` mode.

The `authoring.source` field, when set, must point to a real directory under
`media/textures/`. The `generated_by` field is informational only.

---

## 7. Textures

Every texture must:

- Be a valid PNG.
- Live under `media/textures/<category>/<material>/<file>.png`.
- Have a filename that ends in `_albedo.png`, `_normal.png`, or `_roughness.png`
  for material-driven textures.
- Be 256x256 for block surfaces unless documented otherwise.
- Be referenced by at least one material - or be listed in
  `source/production/allowed_unused.ron`.

For now, block textures stay at 256x256 to keep atlas budgeting predictable.
Other texture categories (icons, sprites) may differ, and Pack Doctor only
warns about dimensions for textures referenced by `block_surface` materials.

---

## 8. Recipes

Every recipe must:

- Live under `defs/recipes/<station>/<stem>.recipe.ron`.
- Reference inputs and outputs that exist as items.
- Reference a real crafting station.
- Produce an item that is at least one of: usable, equipable, placeable, an
  ingredient in another recipe, or explicitly experimental.
- Not break progression: every input must itself be reachable from the basic
  gameplay loop (mine -> craft tool -> mine harder -> craft more).

If the recipe schema is not yet stabilized for a given station, the recipe
file may stay in `source/production/draft/` instead of `defs/recipes/`.

---

## 9. Loot tables

Every loot table must:

- Live under `defs/loot/<bucket>/<stem>.loot.ron`.
- Reference only items that exist.
- Use `core:loot/blocks/empty` as the canonical empty drop.
- Be referenced by at least one block, entity, or structure - or be the
  documented empty table.

---

## 10. Worldgen

Every worldgen entry must:

- Live under `defs/worldgen/<bucket>/<stem>.<kind>.ron`.
- Reference only blocks, materials, biomes, ores, caves, and vegetation that
  exist.
- Ores must replace a block that exists.
- Vegetation trunks and leaves must reference real blocks.
- Biomes must reference a real surface block, subsurface block, and underground
  block.

---

## 11. Tags

Tags are declared in `defs/tags/<bucket>/<stem>.ron`. The loader places them at
`core:tags/<stem>`. References to tag IDs use the form
`core:tag/<bucket>/<stem>` (singular noun). The convention is:

- `core:tag/block/<name>` for block tags.
- `core:tag/item/<name>` for item tags.
- `core:tag/dev/unused_allowed` for content intentionally unreferenced.

Tag namespaces are passively validated; Pack Doctor warns when a referenced tag
prefix is not declared anywhere.

---

## 12. Allowed-unused exceptions

Content that is intentionally orphaned (typically reserve content, modded-pack
hooks, or content kept for narrative or testing reasons) must be declared in:

```
assets/packs/core/source/production/allowed_unused.ron
```

This file is the only acceptable way to silence Pack Doctor's "unused" warnings
without deleting the content. Each entry must include a one-line **why**, so
future agents can audit it.

Silent unused content is forbidden. If a file is unused and undocumented, Pack
Doctor must emit a warning and a human or AI agent must either delete the file
or list it in `allowed_unused.ron`.

---

## 13. AI assistance

AI agents may generate content, but:

- They generate **proposals**, not truth.
- Generated `.ron` must be validated by Pack Doctor before being committed.
- Generated textures must pass dimensions, naming, and material-usage checks.
- No AI agent may invent a parallel architecture, a parallel registry, or a new
  source of truth.

The contract is one-way: the pack is the source of truth, and AI agents write
**into** it. They do not own it.

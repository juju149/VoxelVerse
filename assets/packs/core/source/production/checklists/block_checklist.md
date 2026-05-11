# Block Checklist

Use this list when authoring or reviewing a block. Every box must be checkable
before the block is considered shipped.

Reference rules: [`docs/content_rules.md`](../../../../../../docs/content_rules.md)
sections 2 (naming), 3 (identity), 4 (blocks).

---

## 1. Brief

- [ ] A brief exists at `source/production/briefs/<stem>.md` describing role,
      biome, palette, gameplay intent.
- [ ] If the block already existed before this checklist, a brief is optional
      but encouraged.

## 2. Textures

- [ ] PNG files exist under
      `media/textures/blocks/<material_stem>/<material_stem>_<face>_<map>.png`.
- [ ] Each file is **256x256** unless an exception is documented.
- [ ] Albedo, normal, and roughness maps belong to the same surface.
- [ ] Filenames are lowercase, snake_case, contain `_albedo`, `_normal`, or
      `_roughness`.

## 3. Materials

- [ ] One or more `.material.ron` files live under
      `defs/materials/blocks/<material_stem>/`.
- [ ] Each material references an existing albedo (and, if applicable, normal
      and roughness).
- [ ] `category` and `sampling` are set.
- [ ] `authoring.source` points to a real `media/textures/` directory.

## 4. Block model

- [ ] An existing block model under `defs/block_models/` is reused, or a new
      one is created with documented justification.
- [ ] Material slot keys in the block definition match the model's expected
      slots (`top`, `bottom`, `side`, `end`, `all`, ...).

## 5. Block definition

- [ ] `defs/blocks/<category>/<stem>.block.ron` uses `BlockDef(...)` with the
      current `format_version`.
- [ ] `model` references an existing block model.
- [ ] `visual.materials` references existing materials.
- [ ] `physical` block (solid, opaque, hardness, blast_resistance, friction,
      restitution) is filled.
- [ ] `gameplay.drops` references an existing loot table.
- [ ] `audio` references real sound events.
- [ ] `tags` only contains declared tags.

## 6. Loot table

- [ ] A loot table exists at `defs/loot/blocks/<stem>.loot.ron`, or
      `core:loot/blocks/empty` is intentionally used.
- [ ] Every entry item ID exists.

## 7. Item (if placeable)

- [ ] `defs/items/blocks/<stem>.item.ron` exists.
- [ ] `gameplay: PlaceBlock("core:block/<...>/<stem>")` points to this block.
- [ ] `visual.world_model` is set or intentionally omitted.
- [ ] `tags` contains at least `core:tag/item/block`.

## 8. Tags

- [ ] All tags referenced by the block are declared in `defs/tags/blocks/`.
- [ ] Tag IDs use the `core:tag/block/<name>` convention.

## 9. Recipe (if craftable)

- [ ] `defs/recipes/<station>/<stem>.recipe.ron` exists, or a draft brief is
      stored under `source/production/draft/recipes/`.

## 10. Worldgen (if natural)

- [ ] The block is placed by at least one terrain layer, ore, cave,
      vegetation, or structure entry under `defs/worldgen/`.

## 11. Validation

- [ ] `tools/validate_content.ps1` passes.
- [ ] `tools/pack_doctor.ps1` passes; no new errors, no new warnings for this
      block.
- [ ] `cargo test -p vv-pack-loader -p vv-pack-compiler` passes.

## 12. In-game smoke test

- [ ] The block is visible in the world at the expected biome / depth.
- [ ] Mining yields the expected drops.
- [ ] Placement from inventory works.
- [ ] Lighting, footsteps, break and place sounds behave correctly.

If any box stays unchecked, the block is not done.

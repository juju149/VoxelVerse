# Core Pack MVP

This document captures the minimum content the core pack must ship so that
VoxelVerse offers a coherent first-hour loop: **explore, mine, craft, build**.

It is a living document. Every checked item must remain valid according to
[`docs/content_rules.md`](../../../../../docs/content_rules.md) and pass Pack
Doctor.

---

## North star

The first-hour loop must work end to end:

1. The player lands on a planet.
2. They can identify and mine surface blocks.
3. They can craft a basic tool.
4. They can mine harder blocks (stone, then ore).
5. They can build a simple shelter using placeable blocks.

If a single step in that chain breaks, the MVP is incomplete - independent of
how much content exists elsewhere.

---

## Minimum block set

| Bucket          | Required stems                                                                    |
| --------------- | --------------------------------------------------------------------------------- |
| `terrain`       | `grass`, `dirt`, `stone`, `coarse_dirt`                                           |
| `natural/logs`  | `oak_log` (others optional but encouraged)                                        |
| `natural/leaves`| `oak_leaves`                                                                      |
| `ores`          | `coal_ore`, `iron_ore`                                                            |
| `surface`       | `snow`, `red_sand` (biome-flavored surfaces, at least one secondary biome)        |
| `air`           | `air` (the canonical empty block, must exist and be referenced)                   |

Each block above must satisfy the full per-block checklist:

- block model, materials, textures
- loot table
- placeable item (except `air`)
- worldgen reference (terrain layer, ore, or vegetation)

---

## Minimum item set

- Block items for every placeable block above.
- At least one **resource item**: `coal`, `iron_ingot`.
- At least one **tool item**: `wooden_pickaxe` (proves the tool + mining chain).
- At least one **food / consumable** is optional for MVP.

---

## Minimum recipe set

- `wooden_pickaxe` from a wood-derived ingredient.
- `iron_ingot` from `iron_ore` (smelting station once smelting recipes are
  schema-stable; until then, this recipe stays in `source/production/draft/`).

If the recipe schema is not yet stable, ship a **brief** for each missing
recipe so the design intent is preserved.

---

## Minimum worldgen

- One temperate biome that uses `grass`, `dirt`, `stone` with `oak_log` and
  `oak_leaves` vegetation.
- One contrast biome (cold or arid) that uses `snow` or `red_sand`.
- `coal_ore` and `iron_ore` distributed via ore definitions that replace
  `stone`.

---

## Definition of MVP done

The core pack MVP is **done** when:

- [ ] All blocks above exist with the full per-block checklist green.
- [ ] All items above exist with the per-item checklist green.
- [ ] All recipes above exist or are documented as drafts.
- [ ] Worldgen places every required block.
- [ ] `tools/validate_content.ps1` passes.
- [ ] `tools/pack_doctor.ps1` passes with zero errors.
- [ ] `cargo test -p vv-pack-loader -p vv-pack-compiler` passes.
- [ ] The in-game smoke test (mine grass -> dirt -> stone -> ore -> craft tool
      -> place block) succeeds.

Until every box is checked, do **not** add cosmetic content (extra wood
species, decorative blocks, paint variants). Depth before breadth.

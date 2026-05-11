# Recipe Checklist

Use this list when authoring or reviewing a recipe.

Reference rules: [`docs/content_rules.md`](../../../../../../docs/content_rules.md)
section 8 (recipes) and [`content_pipeline.md`](../../../../../../docs/CONTENT_PIPELINE.md)
section 4.

---

## 1. Station

- [ ] The recipe lives under `defs/recipes/<station>/<stem>.recipe.ron`.
- [ ] The station folder represents a real, supported crafting station
      (`crafting`, `smelting`, ...).

## 2. Inputs

- [ ] Every input item ID resolves to a real item under `defs/items/`.
- [ ] Counts are positive integers in a reasonable range.

## 3. Outputs

- [ ] Every output item ID resolves to a real item.
- [ ] Output counts are positive.

## 4. Time / cost

- [ ] Time (if the station supports it) is set to a sensible value.
- [ ] Fuel or other costs (if applicable) reference real items.

## 5. Category

- [ ] The recipe category matches what the UI sort uses.

## 6. Reachability

- [ ] Each input is reachable through:
  - another recipe, or
  - worldgen, or
  - loot.
- [ ] If an input is itself craftable, that recipe's inputs are reachable too
      (no progression cycles or dead ends).

## 7. Usefulness

- [ ] The output is at least one of: usable directly, equipable, placeable, an
      ingredient in another recipe, or a documented terminal craft.

## 8. Validation

- [ ] `tools/validate_content.ps1` passes.
- [ ] `tools/pack_doctor.ps1` passes; the recipe's items are not listed under
      `missing` or `unused`.
- [ ] `cargo test -p vv-pack-loader -p vv-pack-compiler` passes.

If any box stays unchecked, the recipe is not done.

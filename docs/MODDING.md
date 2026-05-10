# Modding Notes

Modding source files live under `assets/packs/<namespace>/`.

## Pack Layout

```text
pack.ron
defs/
media/
generated/
```

## Content Domains

- Blocks: `defs/blocks/**/*.block.ron`
- Materials: `defs/materials/**/*.material.ron`
- Items: `defs/items/**/*.item.ron`
- Entities: `defs/entities/**/*.entity.ron`
- Props: `defs/props/**/*.ron`
- Vegetation: `defs/vegetation/**/*.ron`
- Worldgen: `defs/worldgen/**/*.ron`
- Loot: `defs/loot/**/*.loot.ron`
- Recipes: `defs/recipes/**/*.ron`

## Rules

- Add content through data, not engine code.
- Keep media and gameplay separate.
- Use lowercase `snake_case` filenames.
- Prefer tags, roles and semantic references over name checks.
- Generated registries are compiler output, not hand-authored gameplay data.

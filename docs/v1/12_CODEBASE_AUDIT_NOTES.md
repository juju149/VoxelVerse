# VoxelVerse V1 Codebase Audit Notes

These notes are based on the current repository structure and code inspection.
They describe what should influence V1 documentation and next work.

## Strong foundations

### Workspace split exists

The project already uses a Rust workspace with dedicated crates for content schema, pack loading, compilation, diagnostics, math, audio, voxel runtime, worldgen, world, physics, meshing, rendering and gameplay.

This is the correct direction. V1 should sharpen boundaries, not collapse crates.

### Unified object schema exists

The content schema already has a unified `.object.ron` design where one gameplay identity can own optional sections such as block, item, mining, tool, weapon, food, station, storage, light, fuel, entity, loot and recipes.

This matches the desired V1 content philosophy.

### Pack loader derives identity from path

Pack loader already derives keys from namespace and file path. This supports strict modding and should become the permanent rule.

### Procedural planet generation is lazy

Worldgen uses lazy surface cache structures and resolves terrain on demand. This is compatible with spherical planets and streaming.

### LOD streaming has real budgets

Rendering already has visible voxel chunk and LOD tile budgets, dispatch/upload budgets, pending job limits and upload time/byte gates.

This should be preserved and made more observable.

### Persistent block damage exists

World state already has a block damage layer and mining feeds damage into world state. This is a major identity feature and should be polished.

### UI theme tokens exist

The UI already has state/style tokens for slots, panels, readability and responsive scaling. This should become the basis of V1 UI instead of ad-hoc UI drawing.

## Main risks

### App runtime loop is too central

The runtime loop currently wires content loading, renderer/audio setup, input events, console, inventory, mining, placing, hotkeys, frame ticking and render calls in one place.

Risk:

- hard to test;
- hard to extend;
- encourages gameplay/render/audio coupling;
- future features will pile up there.

V1 action:

- split into app state, event router, frame driver, feedback router and bootstrap modules.

### Renderer facade is too heavy internally

`Renderer` owns many concepts: GPU resources, text, shadows, UI, sky, fog, chunks, LODs, queues, metrics, first-person item, debug drawing and quality state.

Risk:

- every render change touches central struct;
- hard to reason about lifetimes and resources;
- easy to duplicate caches.

V1 action:

- keep a public facade but split internal owners.

### Existing docs can conflict with current `.object.ron` direction

Some existing content pipeline docs still describe separate folders for blocks, materials, items, loot and recipes as the main flow.

Risk:

- agents may implement old architecture;
- content can duplicate object definitions;
- modders get confused.

V1 action:

- rewrite docs to make `.object.ron` canonical;
- keep world rules separate;
- delete or mark outdated docs.

### Runtime still has dev convenience fallbacks

Content bootstrap has a fallback that auto-scans `.vox` files if no registry exists.

Risk:

- useful during development but dangerous as permanent identity;
- can hide missing manifests.

V1 action:

- replace with strict voxel model manifests and pack doctor errors;
- allow auto-scan only in explicit developer tooling, not normal runtime.

### Mining tuning is partly hardcoded

Mining strike damage, cooldown and impact strength are derived with hardcoded formulas.

Risk:

- hard to tune game feel per tool/block;
- modders cannot express special tools;
- balancing requires Rust changes.

V1 action:

- keep simple defaults but expose important tuning through content where useful.

### Content validation needs to become harsher

The compiler currently has pragmatic behavior such as implicit self-drop in some cases.

Risk:

- silent assumptions;
- content appears to work until progression breaks.

V1 action:

- Pack Doctor should reject ambiguous drops, unresolved tags, unused items and dead recipes.

## Next recommended work

1. Install these V1 docs in the repo.
2. Update root README to point to `AGENTS.md` and `docs/v1/`.
3. Rewrite old `docs/CONTENT_PIPELINE.md` so it no longer promotes old split content as target.
4. Split `apps/voxelverse/src/app/runtime_loop.rs` into focused modules.
5. Introduce a gameplay feedback event pipeline.
6. Make Pack Doctor stricter around `.object.ron`, drops, station tags and voxel model refs.
7. Tune LOD/streaming diagnostics and fix visible seam/horizon artifacts.
8. Polish mining and first-person feedback.

## Audit conclusion

VoxelVerse already has the bones of a serious engine. The danger is feature growth without hard laws.
V1 needs less shiny chaos and more architectural gravity.

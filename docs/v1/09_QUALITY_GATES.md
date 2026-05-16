# VoxelVerse V1 Quality Gates

No phase is complete until its gates pass.
This document defines the gates an agent must enforce before moving on.

## Universal Rust gate

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

All must pass.

## Content gate

```powershell
cargo run -p vv-pack-doctor -- assets/packs/core
cargo test -p vv-pack-loader -p vv-pack-compiler
```

Required result:

- 0 errors;
- 0 warnings for touched content;
- no unresolved references;
- no unused non-debug content unless explicitly allowed;
- no unknown fields.

## Runtime smoke gate

```powershell
cargo run -p voxelverse --release
```

Manual checks:

- game opens;
- world loads;
- player spawns safely;
- camera moves;
- hotbar visible;
- mining works;
- placing works;
- inventory opens/closes;
- no panic in logs;
- no obvious frame freeze.

## Performance gate

During 60 seconds of movement and turning:

- no unbounded pending mesh jobs;
- no visible terrain holes;
- no repeated chunk rebuild storm;
- no extreme upload spikes;
- FPS stable for selected profile;
- diagnostics overlay values make sense.

## Rendering gate

Check:

- spawn screenshot is attractive;
- shadows stable enough;
- fog not creating white screen bands;
- water not broken if enabled;
- LOD not popping near player;
- cracks visible on damaged blocks;
- UI readable over bright and dark backgrounds.

## Gameplay gate

Check:

- player can gather first resources;
- mining damage persists;
- breaking block gives drops or intentional no-drop feedback;
- first tool craft works;
- crafted tool improves mining;
- placing block consumes correct stack;
- inventory cannot lose items silently;
- notices are clear.

## Architecture gate

Reject change if:

- file exceeds 1000 lines;
- a file above 800 lines gets significant new behavior without split;
- code duplicates existing concept;
- old and new systems coexist;
- app loop absorbs system logic;
- renderer owns gameplay rules;
- runtime accepts invalid content;
- tests were not updated for behavior change.

## Documentation gate

Every new system must have:

- clear public API names;
- module-level comment if non-trivial;
- tests explaining edge cases;
- V1 doc update if it changes product/architecture rules.

## Done means done

A task is not done because code compiles.
A task is done when:

- it compiles;
- it is tested;
- it is integrated;
- old path is removed;
- player experience works;
- diagnostics prove no obvious regression;
- docs still match reality.

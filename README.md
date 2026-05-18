# VoxelVerse

VoxelVerse is a spherical-planet voxel survival and creation game written in Rust.

**Status: V1 not released.** This codebase is moving toward V1 under a strict no-legacy rule. No backward-compatibility layer is allowed, no legacy fields, no deprecated aliases, no migration shims. Old systems are deleted, not preserved.

## Sources of truth

The only documentation that describes what VoxelVerse should be is:

- [AGENTS.md](AGENTS.md) — the operating contract for every human or AI contributor.
- [docs/v1/](docs/v1/) — the V1 product, architecture, content, world, rendering, gameplay, UI, audio, quality and roadmap specifications.

Read [AGENTS.md](AGENTS.md) before touching the code. Read the V1 documents in the order listed there.

If a file outside `docs/v1/` describes a different architecture, content model, or visual target, the V1 documents win and the conflicting file must be fixed or deleted in the same change.

## Visual target

VoxelVerse V1 is **stylized modern voxel**, not photorealistic. See [docs/v1/00_PROJECT_VISION.md](docs/v1/00_PROJECT_VISION.md) for the full visual contract — large readable forms, simple clean textures, subtle PBR-lite lighting, strong per-biome color identity, atmosphere doing the heavy lifting, blocks that always read as blocks.

## Workspace layout

```text
apps/voxelverse              playable runtime app
crates/vv-audio              audio playback and audio asset access
crates/vv-content-schema     raw RON schemas only
crates/vv-diagnostics        diagnostics, counters, reports
crates/vv-gameplay           player-facing game rules and state transitions
crates/vv-math               pure math
crates/vv-meshing            CPU mesh generation and mesh scheduling
crates/vv-pack-compiler      validation, reference resolution, registries
crates/vv-pack-doctor        pack linting and reports
crates/vv-pack-loader        filesystem loading and RON parsing
crates/vv-physics            movement and collision primitives
crates/vv-render             GPU rendering, UI rendering, streaming
crates/vv-voxel              voxel IDs, coordinates, chunk keys, storage primitives
crates/vv-world              mutable world runtime and planet state
crates/vv-worldgen           deterministic procedural generation
assets/packs/core            source pack data and compiled registries
```

Crate boundaries and layer law are defined in [docs/v1/02_TARGET_ARCHITECTURE.md](docs/v1/02_TARGET_ARCHITECTURE.md).

## V1 validation commands

Run the universal Rust gate before any change is considered ready:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

When the change touches content packs or the content pipeline, also run:

```powershell
cargo run -p vv-pack-doctor -- assets/packs/core
cargo test -p vv-pack-loader -p vv-pack-compiler
cargo test -p vv-worldgen
```

When the change touches rendering, world streaming, or performance, also run the runtime smoke gate and verify diagnostics in-game:

```powershell
cargo run -p voxelverse --release
```

The full quality gate, with manual checks per area, is defined in [docs/v1/09_QUALITY_GATES.md](docs/v1/09_QUALITY_GATES.md).

## File size discipline

One responsibility per file. Hard limits (see [AGENTS.md](AGENTS.md)):

- 0 to 500 lines: comfortable;
- 500 to 800 lines: acceptable but watch closely;
- 800 to 1000 lines: split before adding significant behavior;
- more than 1000 lines: not acceptable for new V1 work.

# VoxelVerse V1 Target Architecture

This document describes the architecture VoxelVerse should have for V1.
It is intentionally stricter than the current code.

## Current foundation to preserve

The Rust workspace is already a strong base. V1 should keep the crate split and make ownership sharper:

```text
apps/voxelverse       playable runtime app
crates/vv-audio            audio playback and audio asset access
crates/vv-content-schema   raw RON schemas only
crates/vv-diagnostics      diagnostics, counters, reports
crates/vv-gameplay         player-facing game rules and state transitions
crates/vv-math             pure math
crates/vv-meshing          CPU mesh generation and mesh scheduling
crates/vv-pack-compiler    validation, reference resolution, registries
crates/vv-pack-doctor      pack linting and reports
crates/vv-pack-loader      filesystem loading and RON parsing
crates/vv-physics          movement and collision primitives
crates/vv-render           GPU rendering, UI rendering, streaming
crates/vv-voxel            voxel IDs, coordinates, chunk keys, storage primitives
crates/vv-world            mutable world runtime and planet state
crates/vv-worldgen         deterministic procedural generation
```

## Layer law

Dependencies must point downward, never sideways into random implementation details.

```text
app
  -> gameplay, world, render, audio, diagnostics

gameplay
  -> world read/write APIs, pack compiler registries, voxel ids

render
  -> world snapshots, meshing, diagnostics, pack compiler texture registries

world
  -> voxel, worldgen, pack compiler registries

worldgen
  -> math, voxel, compiled procedural registries

pack compiler
  -> content schema

pack loader
  -> content schema

content schema
  -> serde only, no runtime logic
```

Forbidden dependency examples:

- `vv-content-schema` importing `vv-render`;
- `vv-worldgen` importing `vv-render`;
- `vv-meshing` importing app state;
- `vv-gameplay` creating GPU resources;
- runtime app parsing raw RON after startup;
- renderer deciding item gameplay rules.

## App target architecture

`apps/voxelverse` should be a thin composition layer.

Target folders:

```text
apps/voxelverse/src/
  main.rs
  app/
    mod.rs
    bootstrap.rs
    game_app.rs
    event_router.rs
    frame_driver.rs
    input_router.rs
    scene_state.rs
    startup_loading.rs
    golden_scene.rs
  ui/
    mod.rs
```

### App responsibilities

The app may:

- create the window;
- initialize systems;
- route window events;
- own top-level state;
- call tick functions in order;
- request redraws;
- start and stop the game.

The app must not:

- contain mining logic;
- contain crafting rules;
- contain renderer internals;
- contain UI layout math;
- contain procedural generation rules;
- contain pack reference validation;
- grow into a giant runtime loop.

## Runtime loop target

The V1 frame should be explicit:

```text
input ingest
ui event routing
console event routing
player intent sampling
gameplay tick
world simulation tick
feedback event collection
audio event playback
render view update
render frame
clear transient input
```

Each step should have a small function with a clear input and output.

## Canonical state owners

| State | Owner | Notes |
| --- | --- | --- |
| Window and event loop | app | Winit only |
| Player movement state | gameplay/player | Uses physics/world queries |
| Player inventory | gameplay/inventory | Not renderer |
| Hotbar | gameplay/hotbar | Renderer only draws it |
| Mining cooldown and strike rhythm | gameplay/mining | World applies damage |
| Persistent block damage | world/block_damage | Render reads fractions |
| Voxel overrides | world/runtime | Sparse edits only |
| Procedural base terrain | worldgen | Deterministic and immutable |
| Mesh jobs | render + meshing | Renderer schedules, meshing builds |
| GPU resources | render | Never gameplay |
| UI theme tokens | render/ui | Reusable, data-like |
| Audio playback | audio | Triggered by feedback events |
| Pack rules | schema/compiler/doctor | Runtime consumes compiled data |

## Event architecture target

Gameplay systems should produce feedback events instead of directly playing audio or poking render state everywhere.

Target event categories:

```rust
pub enum GameFeedbackEvent {
    ToolSwing { strength: f32 },
    BlockHit { coord: VoxelCoord, sound: SoundKind, strength: f32 },
    BlockBreak { coord: VoxelCoord, sound: SoundKind, drops: Vec<ItemStack> },
    BlockPlace { coord: VoxelCoord, sound: SoundKind },
    InventoryChanged,
    Notice(PlayerNotice),
}
```

The app routes feedback to:

- renderer animation;
- audio engine;
- UI notices;
- diagnostics.

This prevents gameplay from becoming tangled with render/audio.

## Content architecture target

Raw authored content:

```text
assets/packs/<namespace>/defs/**/*.ron
```

Compiler output:

```text
assets/packs/<namespace>/generated/registries/**
assets/packs/<namespace>/generated/reports/**
```

Runtime must consume compiled registries. If raw authoring is still loaded at runtime during V1 development, the code must keep the same conceptual boundary:

```text
load raw -> compile -> runtime registries -> game
```

No gameplay system should know raw schema structs.

## Rendering architecture target

`Renderer` should be split into internal owners:

```text
Renderer
  device/resources owner
  render graph/pass executor
  world streamer
  chunk gpu cache
  lod gpu cache
  ui renderer
  debug renderer
  atmosphere renderer
  first person renderer
  frame metrics
```

A single public `Renderer` facade is fine, but the internals must remain modular.

## Meshing architecture target

Meshing is CPU-side and deterministic.

It must own:

- greedy meshing;
- LOD mesh generation;
- prop baking;
- ambient occlusion;
- rounded edge data packing;
- mesh scheduler budgets.

It must not own:

- GPU buffers;
- Winit window;
- gameplay rules;
- pack loading;
- UI.

## World architecture target

The world is a composition of:

```text
procedural terrain base
+ sparse voxel overrides
+ persistent block damage
+ broken prop layer
+ time state
+ entity state
+ storage/station state
```

V1 should avoid storing a full planet voxel array. Store only edits and generated cache data.

## Diagnostics architecture target

Diagnostics must become a first-class system.

Minimum V1 diagnostics:

- FPS;
- frame time;
- render time;
- update view time;
- LOD selection time;
- mesh jobs dispatched/uploaded/pending;
- GPU upload bytes and time;
- active voxel chunks;
- active LOD tiles;
- draw calls;
- texture atlas memory;
- worldgen cache hit/miss;
- audio voice count;
- player biome and chunk key.

Diagnostics must be visible in-game and usable from logs.

## No abstraction theater

Do not create empty crates or abstract interfaces just because they sound clean.
Create a boundary only when there are at least two real responsibilities to separate or when a file violates the size/responsibility rules.

## Architecture completion gate

V1 architecture is acceptable when:

- app loop is small and readable;
- renderer internals are split;
- gameplay emits feedback events;
- pack validation is strict;
- world state has one owner per concept;
- no file above 1000 lines;
- no duplicated concepts;
- every crate has a clear public API;
- `cargo clippy --workspace --all-targets -- -D warnings` passes.

# Roadmap

## Next Architecture Steps

1. Remove the remaining app-local content/math/voxel facade modules once all runtime modules import shared crates directly.
2. Split renderer inputs from game state, then extract a real `vv-render` crate.
3. Split meshing after renderer extraction so CPU mesh generation has a stable boundary.
4. Introduce `vv-cli` when there is a concrete command, starting with pack validation.
5. Introduce VoxelVerse Studio only after pack validation/export and preview-render boundaries are stable.

## Tooling Later

- Offline texture generation should become a real crate/tool when texture recipes are data-driven.
- Preview rendering should consume compiled registries and renderer-facing data, not game runtime state.
- Pack export should compile validated data into a future compact artifact format.


# VoxelVerse Studio 0.1

Local-first pack authoring app for VoxelVerse.

The first version is intentionally small:

- Textures page for mock texture recipes and previews.
- Blocks page for beginner-safe block creation fields.
- Mock validation warnings and export actions.
- A minimal Tauri shell prepared for future local file commands.

The Studio must stay aligned with the engine content pipeline:

```text
raw .ron and PNG assets
-> validation
-> compilation
-> runtime registries
-> game runtime
```

For now the frontend uses mock data from `src/data/mockStudioData.ts`. It does not parse packs, generate PNGs, or export files yet.

## Commands

```powershell
npm install
npm run dev
npm run check
npm run build
npm run tauri:dev
npm run tauri:build
```

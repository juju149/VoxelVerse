# Phase 4 - Renderer split

Fait par script :

- Création des modules cibles.
- Ajout garde-fou renderer.

À faire ensuite :

1. Sortir le hit-testing inventaire de `Renderer`.
2. Remplacer `Renderer::render(&Controller, &Player, ...)` par des données de frame :
   - CameraFrame
   - WorldRenderFrame
   - UiFrame
   - DebugOverlayFrame
3. Extraire shadow/terrain/sky/debug/ui pass depuis `frame.rs`.
4. Retirer la dépendance `vv-input` de `vv-render`.
# Phase 2 - Voxel core split

Fait par script :

- Création de `vv-voxel`.
- Migration mécanique des imports principaux.
- Compat temporaire dans `vv-core`.

À faire ensuite :

1. Supprimer les re-exports vv-core quand tout compile sans eux.
2. Décider si `LodKey` reste dans vv-voxel ou part dans render/mesh.
3. Faire échouer CI si vv-core recommence à contenir des concepts voxel.
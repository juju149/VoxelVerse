# VoxelVerse Studio — Architecture Report

> Document de décision stratégique — Mai 2026  
> Rédigé après analyse complète du code source de VoxelVerse Studio v0.1  
> Ce rapport doit être lu avant toute modification significative du Studio.

---

## 1. État actuel du projet

### 1.1 Structure générale

```
apps/voxelverse-studio/
  src/
    app/           App.tsx                     ← état global, routing, actions
    pages/         MaterialsPage, BlocksPage   ← deux routes actives
    components/
      layout/      AppShell, Sidebar, TopBar, StatusBar
      materials/   MaterialBlueprintEditor, MaterialLibrary,
                   MaterialPreview, MaterialRonEditor, MaterialWizard
      blocks/      BlockBuilder, BlockCubePreview, BlockLibrary,
                   BlockVariationPreview, BlockWizard
      ui/          composants shadcn/ui
      validation/  ValidationPanel, ExportButton
    lib/
      blueprint/   materialBlueprint.ts        ← graph → recipe / recipe → graph
      procedural/  color.ts, evaluator.ts, noise.ts, seed.ts
      presets/     materialPresets.ts, blockPresets.ts
      ron/         ronSerializer.ts, ronParser.ts (vide côté parsing)
      validation/  packValidator.ts
      ids.ts, cn.ts
    types/         studio.ts                   ← source de vérité des types
    data/          initialProject.ts           ← données mockées de démo
```

### 1.2 Pages existantes

| Page | Route | État |
|------|-------|------|
| Materials | `/materials` | Fonctionnelle avec graph, RON, preview |
| Blocks | `/blocks` | Fonctionnelle avec builder, preview, validation |

Il n'existe que deux routes. Aucun routeur de pages (React Router, TanStack Router) n'est utilisé — le routing est géré par un simple `useState<StudioRoute>` dans `App.tsx`.

### 1.3 Composants existants — analyse honnête

**Ce qui fonctionne vraiment :**
- Graph editor basé sur `@xyflow/react` avec ajout/suppression de nœuds
- Preview procédurale générée côté JS canvas (CPU, pixel par pixel)
- Preview 4×4 tile repeat et cube 3D CSS isométrique
- Serializer RON qui génère des fichiers `.ron` lisibles
- Validator avec fixes automatiques (normalize-id, clamp-hardness, etc.)
- Export multi-fichiers (.ron par material, .ron par block)
- Persistance `localStorage` avec schemaVersion guard
- Variation seeds déterministes et preview de 9 variations

**Ce qui est mocké / non connecté au moteur de jeu :**
- `initialProject.ts` : 18 matériaux et 17 blocs entièrement codés en dur en JS, jamais lus depuis des fichiers `.ron`
- Le "Save" sauvegarde dans `localStorage`, pas sur le disque
- L'export génère des fichiers `.ron` téléchargés par le navigateur — jamais écrits dans `assets/packs/`
- Le `ronParser.ts` ne contient qu'une fonction `normalizeRonEdit` qui fait `trimEnd()`. Il ne parse rien
- Le raw RON override est une textarea libre sans validation de syntaxe
- Le graph blueprint est visuel mais la connexion entre nœuds ne change pas réellement l'évaluation du rendu (tous les nœuds sont connectés à `output` de façon linéaire)
- Aucune connexion à Tauri — l'app fonctionne comme un SPA sans accès fichier

**Ce qui est fragile :**
- `MaterialKind` est une union de strings hardcodées : `"grass_top" | "grass_side" | "dirt_base" | ...`. Chaque nouveau matériau nécessite de modifier `studio.ts`, `materialPresets.ts`, les palettes `PALETTES`, les couches `PATHS`, et `layersFor()`. C'est une duplication en cascade.
- Le `schemaVersion: 4` dans `PackProject` mais il n'existe pas de migration entre versions
- `initialProject.ts` importe `validatePack` et appelle `project.validationIssues = validatePack(project)` en dehors d'un return — mutation après assignation, anti-pattern
- `MaterialBlueprintEditor` utilise `useEffect` pour synchroniser les nodes React Flow avec les données material — risque de désync
- La compilation `recipe → blueprint → recipe` dans `compileRecipeFromBlueprint` est une boucle fragile : elle écrase la recipe depuis le graph mais le graph est lui-même généré depuis la recipe

**Ce qui bloque l'avenir :**
- `MaterialKind` fermé empêche un moddeur d'ajouter son propre matériau sans modifier le moteur
- Pas de Tauri IPC = pas de lecture/écriture réelle de fichiers
- Pas de système de gestion de projet (quel dossier, quel pack, quel namespace)
- Le Studio ignore complètement les fichiers `.ron` existants dans `assets/packs/core/`
- Le Studio ignore complètement le schéma Rust (`vv-content-schema`) qui définit `RawBlockDef` avec `TextureRef` (albedo, normal, roughness) — format incompatible avec ce que le Studio génère
- Aucune preview WebGL/WebGPU — uniquement CSS et canvas CPU
- Aucune gestion de templates, de bibliothèques partagées, de packs multi-namespace

### 1.4 Incompatibilité critique Studio ↔ Moteur

Le Studio génère actuellement ce format RON pour un matériau :

```ron
MaterialFaceDef(
    id: "core:grass/top",
    material_kind: "grass_top",
    recipe: ProceduralMaterialRecipe(
        style: soft_natural,
        palette: (base: "#7BAA32", ...),
        pattern_layers: [...]
    )
)
```

Le moteur Rust attend ce format RON pour un bloc :

```ron
RawBlockDef(
    display_name: "Grass Block",
    solid: true,
    color: (0.49, 0.80, 0.21),
    visual: Some((
        top: Some((
            albedo: "core:blocks/grass_block/grass_block_top_albedo",
            normal: "...",
            roughness: "...",
        ))
    ))
)
```

**Ces deux formats sont totalement incompatibles.** Le Studio génère des recettes procédurales. Le moteur attend des références de textures PNG. Il n'existe aucun compilateur entre les deux. C'est le problème architectural le plus urgent.

---

## 2. Problèmes UX actuels

### 2.1 L'écran "New Material" actuel

Quand l'utilisateur clique sur le `+` dans la Material Library, une `Dialog` s'ouvre avec :

1. Une grille de 19 boutons représentant les `MaterialKind` : Grass Top, Grass Side, Dirt, Stone, Cobblestone...
2. Une rangée de 4 styles : Soft Natural, Clean Stylized, Rich Organic, Simple Flat

**Problèmes précis :**

- **Le nom est trompeur.** Ces boutons ne sont pas de vrais types de matériaux. Ce sont des presets préconfigurés avec leurs propres palettes, couches pattern et paramètres. Choisir "Grass Top" crée un matériau pré-câblé. L'utilisateur n'est pas en train de choisir un *type*, il choisit un *résultat*.

- **L'utilisateur est immédiatement enfermé.** Si je veux créer un matériau d'ardoise bleue, je dois partir de "Stone" et tout écraser. Le preset devient un bruit de fond que je dois défaire.

- **Le "Custom" preset est la seule porte de sortie** mais il crée un matériau quasi vide avec une seule couche `soft_noise`. Ce n'est pas assez guidé pour un débutant, et pas assez libre pour un expert.

- **La modale est disproportionnée.** Elle force un choix dès l'ouverture, avant que l'utilisateur sache ce qu'il veut faire.

- **Les styles sont orthogonaux aux kinds.** Pourquoi "Grass Top + Rich Organic" ? Ça n'a pas de sens visuel. Le style devrait être un paramètre de l'éditeur, pas du wizard.

- **Pas de preview dans le wizard.** L'utilisateur choisit à l'aveugle.

### 2.2 Réponses aux questions clés

**Faut-il afficher des presets au premier clic ?**  
Non. Pas comme première action. Les presets doivent exister, mais en tant que bibliothèque consultable séparément — pas comme gate obligatoire vers la création.

**Faut-il plutôt ouvrir un canvas vide ?**  
Oui, avec un minimum de structure initiale. Un canvas vide absolu désorienterait un débutant. La bonne cible : ouvrir un canvas avec seulement les nœuds structurels (Palette, Stylization, Surface, Variation, Output) sans aucune couche Pattern — l'utilisateur choisit ce qu'il veut construire.

**Comment éviter d'enfermer l'utilisateur dans des types rigides ?**  
Supprimer `MaterialKind` comme union fermée. Remplacer par un champ libre `label: string` et des catégories optionnelles. Le matériau est identifié par son path (namespace:path), pas par son "kind".

**Comment garder des templates utiles ?**  
Bibliothèque de templates séparée, accessible via un bouton "Start from template" dans l'éditeur, pas avant. Un template est un `.ron` de matériau complet que l'on clone. Il ne dicte pas le type — il propose un point de départ.

**Différence New Material / New From Template / Import :**

| Action | Description | Comportement |
|--------|-------------|--------------|
| New Material | Canvas vide avec structure minimale | Ouvre l'éditeur immédiatement |
| New From Template | Clone d'un template existant | Affiche la galerie de templates, puis clone |
| Import | Charge un `.ron` existant depuis le disque | Dialogue de fichier (via Tauri IPC) |

**UX pour enfant :**
- Boutons grands avec noms clairs
- Preview permanente et grande
- Simple Mode avec 3-4 paramètres seulement (couleur, rugosité, pattern)
- Pas de graph visible
- Résultats immédiats et visibles

**UX pour expert :**
- Graph editor complet
- Accès au `.ron` brut
- Paramètres exposés dans l'inspecteur
- Hot reload depuis fichier disque
- Accès aux seeds, au tiling, aux masques

### 2.3 UX cible recommandée pour New Material

```
[Clic sur +]
    │
    ▼
Layout 2 colonnes :
┌─────────────────────────────────────────────────────────┐
│  Left: Graph canvas vide                                 │
│  (Palette → Output, Stylization → Output, etc.)         │
│                                                          │
│  Right: Inspecteur                                       │
│    - Nom du matériau (input)                             │
│    - ID (autogénéré depuis le nom)                       │
│    - Mode : Simple / Advanced / Expert                   │
│    [Simple mode] : couleur de base, pattern, rugosité   │
│    [Advanced] : tous les paramètres de nœuds            │
│    [Expert] : graph + RON brut                          │
│                                                          │
│  Bottom : Preview live (tile + cube)                     │
│  Boutons : "Start from Template" | "Create"              │
└─────────────────────────────────────────────────────────┘
```

---

## 3. Vision produit du Studio

### 3.1 Mission du Studio

VoxelVerse Studio est un **outil de création de contenu local-first**, initialement centré sur :
- Créer des matériaux procéduraux
- Créer des blocs à partir de matériaux
- Valider et exporter des packs `.ron`

À terme, il doit couvrir :

```
Phase 1  : Materials + Blocks (actuel, à corriger)
Phase 2  : Graph Blueprint procédural réel
Phase 3  : RON Schema propre et compilateur Studio → moteur
Phase 4  : Models 3D voxel (items, décorations)
Phase 5  : Vegetation editor
Phase 6  : Structures editor
Phase 7  : Planet rules editor
Phase 8  : Connexion au metaverse / planètes joueur
Phase 9  : Marketplace de packs
Phase 10 : MCP / Agent IA intégré
```

### 3.2 Ce qui doit être codé maintenant vs préparé

| Domaine | Maintenant | Préparer sans coder |
|---------|-----------|---------------------|
| Materials | Canvas vide, graph sans presets, bridge → PNG | — |
| Blocks | Nettoyage BlockDef, formats face cohérents | Slab/Stairs geometry |
| RON | Schema unifié Studio ↔ moteur | Migration versioning |
| Tauri IPC | Lecture/écriture fichiers réels | Autocompletion paths |
| Preview | WebGL cube preview | WebGPU realtime |
| Validation | Validator enrichi avec warnings PBR | Budget de perf |
| Export | Pack compiler complet | Release vs dev mode |
| Models | — | Définir format VoxelModelDef |
| Vegetation | — | Définir VegetationDef |
| Planet | — | Définir PlanetRuleDef |
| Marketplace | — | Prévoir namespace / signing |
| MCP | — | Exposer les types comme API |

---

## 4. Architecture Material

### 4.1 La question fondamentale : preset ou canvas vide ?

**Réponse : canvas vide avec structure minimale.**

Un material n'est pas un type. C'est une composition de propriétés. L'identité du matériau vient de :
- Son namespace:path (ex: `core:terrain/grass_top`)
- Son graph de nœuds procéduraux
- Ses paramètres de surface (roughness, normal, height, AO, emission)
- Sa seed

`MaterialKind` doit disparaître en tant qu'union fermée. Il peut subsister comme `category: string` libre.

### 4.2 Granularité des faces

**Question : chaque face doit-elle avoir son propre `.ron` ?**

**Oui, recommandé.** C'est la architecture cible pour plusieurs raisons :

1. **Réutilisabilité** : `core:terrain/dirt_base` peut être référencé par le bottom de Grass, le Dirt Block, la surface de Mycelium, etc.
2. **Variation per-face** : le top d'un bloc peut avoir une variation différente du side
3. **Override par biome** : un mod peut remplacer uniquement `core:terrain/grass_top` sans toucher au reste
4. **Compilation séparée** : on peut bake chaque face indépendamment
5. **Déduplication** : deux blocs partageant le même matériau ne dupliquent pas les données

Structure de fichiers recommandée :

```
assets/packs/core/
  materials/
    terrain/
      grass_top.ron
      grass_side.ron
      dirt_base.ron
      stone_base.ron
    wood/
      oak_rings.ron
      oak_planks.ron
    liquid/
      water_surface.ron
      lava_flow.ron
  blocks/
    grass_block.ron   ← référence core:terrain/grass_top etc.
    dirt.ron
```

### 4.3 Gestion des matériaux partagés, variantes, seeds

| Problème | Solution recommandée |
|----------|---------------------|
| Matériaux partagés | Chaque face = un fichier `.ron` référencé par ID |
| Variantes par biome | `variants: [{ biome_tags: ["cold"], override: { base_color: "#A0B0C0" } }]` dans le `.ron` |
| Variation per-block | `variation.per_block_strength` dans la recipe (déjà présent) |
| Seeds | `seed` dans le matériau + `pack_seed` dans `pack.toml` combinés via hash |
| Tile seamless | Pas de bords artificiels dans le graph — les patterns doivent être tileables |
| Voisins identiques | Variation per-block hash sur position voxel (en cours, à connecter) |

### 4.4 Comparaison des stratégies de texture

| Option | Description | Avantages | Inconvénients | Score VoxelVerse |
|--------|-------------|-----------|---------------|-----------------|
| **1. Full procedural runtime** | Le shader évalue le graph à chaque frame | Variation infinie, pas de PNG | Coût GPU élevé, shader complexe, difficile à moddeer | 4/10 |
| **2. Procedural offline → cache texture** | Le graph est évalué offline, résultat = PNG baked | GPU léger, moddable, compatible atlas | Pas de variation par bloc au runtime sans tableau de textures | 7/10 |
| **3. PNG source classique** | L'artiste dessine les PNG | Simple, prévisible, rapide | Pas de variation, 0 moddabilité sans recompilation, pas scalable | 3/10 |
| **4. Hybride graph .ron + cache GPU** | Graph bake vers texture array, variation legère en shader | Scalable, moddable, beau | Pipeline plus complexe | **9/10** |
| **5. Shader procedural spécialisé** | Shader WGSL custom par type de matériau | Très performant | Non moddable, inflexible | 2/10 |

**Recommandation : Option 4 — Hybride graph .ron + cache GPU**

```
.ron graph
    │
    ▼
Studio Compiler
    │  évalue le graph procédural
    ▼
Texture PNG baked (albedo, normal, roughness)
    │  (8-16 variantes)
    ▼
Texture Array GPU (BC7/ASTC compressed)
    │
    ▼
Renderer — shader choisit variante via (block_seed XOR position_hash)
```

Le shader peut faire une variation légère (tint, micro-detail) sans réévaluer le graph complet.

### 4.5 Le PNG ne doit-il pas disparaître ?

**Non. Le PNG est le cache.** Il est le résultat compilé du graph `.ron`. Il ne doit jamais être source de vérité ni éditable directement. Le workflow est :

```
Source de vérité : material.ron (graph procédural)
Cache compilé    : material_v1234.png (baked)
Runtime          : texture array GPU
```

Le PNG dans le repo est valide comme cache versioned. Modifier le graph invalide le cache.

### 4.6 PBR-lite

La pipeline doit supporter au minimum :
- `albedo` (couleur de base)
- `normal` (bump léger, optionnel)
- `roughness` (réflexion)
- `ao` (ambient occlusion baked, optionnel)
- `emission` (pour lava/cristaux/portails)

`height` peut être dérivé de `normal` automatiquement. Pas besoin d'une map dédiée pour l'instant.

Les matériaux animés (eau, lava) nécessitent une `animation` section dans le `.ron` :
```ron
animation: Some((
    frames: 8,
    fps: 4,
    kind: scroll_uv,
))
```

---

## 5. Architecture du Graph Blueprint

### 5.1 Problème actuel du graph

Le graph actuel dans `MaterialBlueprintEditor.tsx` est visuellement présent mais architecturalement faux :

- Tous les nœuds sont connectés à `output` de façon plate et linéaire
- Il n'existe pas de vrai graphe orienté avec propagation de valeurs
- `compileRecipeFromBlueprint` extrait les paramètres nœud par nœud sans tenir compte des connexions
- Les connexions visuelles entre nœuds n'affectent pas l'évaluation
- Le graph est un cosmétique sur une recipe linéaire

**Ce n'est pas un graph. C'est un form déguisé en graph.**

### 5.2 Ce qu'un vrai graph procédural requiert

Un vrai graph évalue ainsi :

```
Input nodes (constantes, couleurs, seeds)
    │
    ▼
Transform nodes (noise, patterns, shapes, math)
    │
    ▼
Blend nodes (mix, multiply, overlay)
    │
    ▼
Output node (albedo, normal, roughness)
```

Chaque nœud doit avoir des ports typés (Float, Color, Mask, etc.) et les connexions doivent déterminer l'évaluation, pas les être ignorées.

### 5.3 Nodes minimaux pour v0.1

```
CATÉGORIE INPUT
  ├── Color      : constante de couleur
  ├── Float      : constante flottante
  ├── Seed       : seed du matériau (pour variation)
  └── UV         : coordonnées UV de base

CATÉGORIE PATTERNS (génèrent une valeur 0..1)
  ├── Noise FBM  : value noise fractal (3 octaves)
  ├── Voronoi    : cellules organiques
  ├── Stripes    : bandes parallèles
  ├── Rings      : cercles concentriques (radial)
  ├── Dots       : points aléatoires via Voronoi
  └── Flat       : valeur constante (0 ou 1)

CATÉGORIE SHAPES (masques géométriques)
  ├── Gradient   : gradient linéaire ou radial
  ├── Band       : bande horizontale ou verticale
  └── Edge Mask  : masque d'arête (edge wear)

CATÉGORIE WARP
  └── Domain Warp : déforme les coordonnées UV d'un pattern

CATÉGORIE BLEND
  ├── Mix        : lerp entre deux valeurs
  ├── Multiply   : multiplication
  ├── Screen     : screen blend
  └── Overlay    : overlay

CATÉGORIE ADJUST
  ├── Remap      : remap 0..1 → custom range
  ├── Contrast   : ajuste contraste
  ├── Colorize   : applique une couleur sur un mask
  └── Quantize   : posterize (cartoon steps)

CATÉGORIE OUTPUT
  └── Material Output : albedo (Color), normal (Mask), roughness (Float)
```

### 5.4 Nodes long terme

```
CATÉGORIE GEOMETRY SHAPES
  ├── Brick       : pattern de briques (offset rows)
  ├── Planks      : bandes parallèles avec joints
  ├── Cobblestone : Voronoi avec grout
  ├── Cells       : cellules organiques variées
  ├── Cracks      : fissures (seuil sur Voronoi)
  └── Strata      : couches géologiques horizontales

CATÉGORIE MASKS AVANCÉS
  ├── Normal Mask : masque basé sur direction de normale (utile pour planètes)
  ├── Height Mask : masque basé sur hauteur Y (variation altitude)
  └── Curvature   : masque courbure du mesh (AO baked)

CATÉGORIE TEXTURE
  ├── Image Input : texture PNG source
  ├── Tiler       : tile avec variation
  └── Blend Textures : mix entre deux textures

CATÉGORIE ANIMATION
  └── Time        : drive des nœuds animés (eau, lava)

CATÉGORIE BIOME
  ├── Biome Tint  : injecte la couleur de biome
  └── Climate Mix : blend entre variantes selon température/humidité
```

### 5.5 Exposition Simple Mode

Chaque nœud peut marquer certains paramètres `exposed: true` pour les rendre visibles en Simple Mode. L'utilisateur Simple Mode voit uniquement une liste de paramètres exposés, pas le graph.

```ron
FloatNode(
    id: "roughness_control",
    value: 0.75,
    exposed: true,
    exposed_label: "Surface roughness",
)
```

### 5.6 Sauvegarder le graph en .ron

Le graph doit être sérialisé dans un format stable et versionnable :

```ron
ProceduralGraph(
    version: 1,
    nodes: [
        ColorNode(id: "base_color", value: "#7BAA32"),
        NoiseNode(id: "large_patches", kind: fbm, frequency: 4.0, octaves: 3),
        VoronoiNode(id: "cells", scale: 6.0),
        BlendNode(id: "blend1", mode: overlay, strength: 0.45),
        MaterialOutput(id: "output"),
    ],
    connections: [
        (from: "base_color.out", to: "blend1.base"),
        (from: "large_patches.out", to: "blend1.layer"),
        (from: "blend1.out", to: "output.albedo"),
        (from: "cells.out", to: "output.roughness"),
    ],
    exposed_params: [
        (node: "base_color", param: "value", label: "Base Color"),
        (node: "blend1", param: "strength", label: "Pattern Strength"),
    ],
)
```

### 5.7 Versionnage et compatibilité des nœuds

Chaque type de nœud doit avoir une version :

```ron
NoiseNode_v2(...)   ← breaking change → nouveau type
NoiseNode(...)      ← v1 supportée via migration
```

La migration doit être gérée dans le compilateur du Studio, pas dans le moteur de jeu.

### 5.8 Validation et compilation du graph

Le graph doit être validé avant export :
- Tous les nœuds référencés dans les connexions existent
- L'output reçoit au moins un `albedo`
- Pas de cycles
- Les types des ports sont compatibles (Color → Color, Float → Float)
- La profondeur du graph ne dépasse pas N nœuds (budget de performance)

---

## 6. RON Schema

### 6.1 Principes fondamentaux

**Un fichier = une entité.** Pas de tableaux multiples dans un même fichier.

**Path-as-identity.** L'ID d'un contenu est dérivé du chemin du fichier :

```
assets/packs/core/materials/terrain/grass_top.ron
→ ID : core:terrain/grass_top
```

Le fichier `.ron` ne doit pas répéter son propre ID en champ `id`.

**Dépendances explicites.** Toute référence à un autre contenu utilise son ID namespaced.

### 6.2 Exemple : MaterialFaceDef

```ron
// Fichier : assets/packs/core/materials/terrain/grass_top.ron
// ID automatique : core:terrain/grass_top

MaterialFaceDef(
    display_name: "Cartoon Grass Top",
    category: "terrain",
    tags: ["natural", "grass", "animated_tint"],

    // Seed du matériau — combiné avec pack_seed pour la variation finale
    seed: 149,

    // Paramètres PBR-lite exportés vers le runtime
    surface: (
        roughness: 0.78,
        normal_strength: 0.22,
        ao_strength: 0.35,
        emission: 0.0,
    ),

    // Paramètres de variation par-bloc
    variation: (
        enabled: true,
        per_block_strength: 0.18,
        color_jitter: 0.08,
    ),

    // Supports biome tint depuis le worldgen
    biome_tint_slot: Some("grass"),

    // Graph procédural source de vérité
    graph: "core:graphs/terrain/grass_top",

    // Cache PNG baked (généré automatiquement)
    // Ne pas modifier manuellement
    baked_cache: Some((
        albedo: "core:baked/terrain/grass_top_albedo_v3",
        normal: "core:baked/terrain/grass_top_normal_v3",
        roughness: "core:baked/terrain/grass_top_roughness_v3",
        resolution: 32,
        variants: 8,
    )),
)
```

### 6.3 Exemple : ProceduralGraph

```ron
// Fichier : assets/packs/core/graphs/terrain/grass_top.ron
// ID automatique : core:graphs/terrain/grass_top

ProceduralGraph(
    version: 1,
    nodes: [
        ColorNode(id: "base", value: "#7BAA32"),
        ColorNode(id: "shadow", value: "#5F8D29"),
        ColorNode(id: "highlight", value: "#9ACB4E"),
        NoiseFbmNode(id: "broad", frequency: 4.0, octaves: 3, seed_salt: 0),
        VoronoiNode(id: "cells", scale: 9.0, mode: f2_minus_f1),
        BlendNode(id: "broad_blend", mode: overlay, strength: 0.55),
        BlendNode(id: "cell_blend", mode: shadow, strength: 0.45),
        ColorizeNode(id: "colorize", color_low: "shadow", color_high: "highlight"),
        MaterialOutput(id: "output"),
    ],
    connections: [
        (from: "base.out",        to: "broad_blend.base"),
        (from: "broad.out",       to: "broad_blend.layer"),
        (from: "broad_blend.out", to: "cell_blend.base"),
        (from: "cells.out",       to: "cell_blend.layer"),
        (from: "cell_blend.out",  to: "colorize.mask"),
        (from: "colorize.out",    to: "output.albedo"),
        (from: "broad.out",       to: "output.roughness"),
    ],
    exposed_params: [
        (node: "base", param: "value", label: "Grass Color"),
        (node: "broad", param: "frequency", label: "Patch Scale"),
        (node: "broad_blend", param: "strength", label: "Contrast"),
    ],
)
```

### 6.4 Exemple : BlockDef

```ron
// Fichier : assets/packs/core/blocks/grass_block.ron
// ID automatique : core:grass_block

BlockDef(
    display_name: "Grass Block",
    category: "terrain",
    tags: ["natural", "terrain", "grass"],
    seed: 1001,

    geometry: (
        shape: cube,
        collision: solid_cube,
    ),

    render: (
        faces: (
            top: "core:terrain/grass_top",
            side: "core:terrain/grass_side",
            bottom: "core:terrain/dirt_base",
        ),
        ao: true,
        transparent: false,
        cull_faces: true,
        light_emission: 0,
        tint_slots: { grass: "biome_grass" },
    ),

    gameplay: (
        walk_through: false,
        hardness: 0.6,
        break_category: soft,
        drops: ["core:dirt"],
        tool_required: None,
    ),

    roles: [default_place],
)
```

### 6.5 Exemple : VoxelModelDef

```ron
// Fichier : assets/packs/core/models/items/iron_pickaxe.ron
// ID : core:models/items/iron_pickaxe

VoxelModelDef(
    display_name: "Iron Pickaxe",
    grid_size: 16,          // 16×16×16 micro-voxels
    voxel_size_cm: 3.125,   // 1 block = 50cm → 50/16 = 3.125cm par micro-voxel
    lod_levels: 2,
    collision: aabb,        // bounding box simple pour items

    // Source soit voxels inline soit référence fichier .vox
    source: Inline([
        // (x, y, z, material_id)
        (0, 0, 0, "core:iron"),
        (1, 0, 0, "core:iron"),
        ...
    ]),
)
```

### 6.6 Exemple : ItemDef

```ron
// Fichier : assets/packs/core/items/iron_pickaxe.ron
// ID : core:iron_pickaxe

ItemDef(
    display_name: "Iron Pickaxe",
    category: "tool",
    tags: ["tool", "pickaxe", "iron"],
    model: "core:models/items/iron_pickaxe",
    stack_size: 1,

    tool_props: Some((
        kind: pickaxe,
        tier: iron,
        speed_multiplier: 6.0,
        breaks: ["stone", "ore", "metal"],
        durability: 250,
    )),
)
```

### 6.7 Exemple : VegetationDef

```ron
// Fichier : assets/packs/core/vegetation/oak_tree.ron
// ID : core:vegetation/oak_tree

VegetationDef(
    display_name: "Oak Tree",
    kind: tree,
    biome_tags: ["forest", "plains"],

    trunk: (
        block: "core:oak_log",
        min_height: 4,
        max_height: 7,
        taper: 0.0,
    ),

    canopy: (
        shape: sphere,
        block: "core:oak_leaves",
        radius_min: 2.5,
        radius_max: 3.5,
        density: 0.85,
        hanging_vines: false,
    ),

    placement: (
        surface_block_tags: ["grass", "soil"],
        min_spacing: 5,
        cluster_size: (1, 3),
        tilt_to_gravity: true,
    ),
)
```

### 6.8 Gestion des versions de schéma

```ron
// pack.toml
[schema]
block_version = 2
material_version = 1
graph_version = 1
```

Le compilateur Studio doit inclure un module `migration.rs` qui fait correspondre les anciennes versions aux nouvelles. Les packs anciens sont migrés à la compilation, pas au runtime.

### 6.9 Gestion des overrides

Un pack secondaire peut override un fichier core en reproduisant le même path :

```
packs/my_mod/blocks/grass_block.ron → override core:grass_block
packs/my_mod/materials/terrain/grass_top.ron → override core:terrain/grass_top
```

L'ordre de chargement définit la priorité. `pack.toml` déclare les dépendances :

```toml
[pack]
name = "My Mod"
depends_on = ["core"]
override_policy = "merge"   # ou "replace"
```

---

## 7. Architecture des blocs

### 7.1 Tous les blocs sont-ils des cubes ?

Non, et c'est une question stratégique. L'approche recommandée est **progressive** :

| Phase | Types supportés |
|-------|----------------|
| v0.1 | Cube plein uniquement |
| v0.2 | Cross-plane (plantes) |
| v0.3 | Slab (demi-bloc) |
| v0.4 | Stairs, Fence (multi-box) |
| v0.5 | VoxelModel custom |

### 7.2 Recommandation sur les Planks

Les planches en bois (`planks`) sont un cas test de la stratégie de matériau.

| Approche | Description | Qualité | Perf | Moddabilité |
|----------|-------------|---------|------|-------------|
| Full procedural | Graph évalué au runtime | Très élevée | Médiocre | Très élevée |
| Semi-procedural | Bandes + grain via graph, baked en PNG | Élevée | Très bonne | Élevée |
| Graph + shapes géo | Nœuds Planks + Grain dans le graph | Élevée | Bonne | Élevée |
| Texture dessinée | PNG peint à la main | Artisanale | Excellente | Faible |
| Modèle custom | VoxelModel avec géométrie | Overkill pour planks | Variable | Moyenne |

**Recommandation : Semi-procedural baked.** Un graph simple avec un nœud `PlanksNode` (bandes horizontales avec offset alternant) + grain FBM, compilé vers 8 variantes PNG. Les planches ont une texture très régulière — le procedural baked est idéal.

### 7.3 États et variants directionnels

La gestion des états (`BlockState`) doit être pensée maintenant mais pas codée :

```ron
// Futur
block_states: [
    State(id: "facing", values: ["north", "south", "east", "west"]),
    State(id: "waterlogged", values: [true, false]),
]
```

Pour v0.1 : les blocs sont stateless. La direction peut être gérée par la seed.

### 7.4 LOD des blocs

Les blocs proches : mesh voxel plein avec matériaux
Les blocs distants : couleur plate dérivée de `color` dans `BlockDef` (déjà présent dans le schéma Rust ✓)
Les blocs très distants : impostor ou heightmap

Le Studio doit exposer la `color` de LOD dans le BlockBuilder.

---

## 8. Geometry, Voxel Models et Items

### 8.1 Faut-il un éditeur voxel intégré ?

**Pas maintenant.** Un éditeur voxel intégré est un projet de 3-6 mois. Pour v0.1-v0.3, supporter l'import de fichiers `.vox` (MagicaVoxel) est suffisant. Le Studio fournira un éditeur de propriétés (taille, collision, LOD) mais pas d'édition directe des voxels.

### 8.2 Quelle taille de micro-voxel ?

| Résolution | Taille voxel | Détail | Perf | Utilisation |
|-----------|-------------|--------|------|-------------|
| 1×1×1 block | 50 cm | Aucun | Excellente | Blocs de terrain |
| 8×8×8 | 6.25 cm | Faible | Bonne | Items simples |
| 16×16×16 | 3.125 cm | Bonne | Correcte | Items standards |
| 32×32×32 | 1.56 cm | Très bonne | Médiocre | Décorations riches |
| 64×64×64 | 0.78 cm | Excessive | Mauvaise | Inutilisable en masse |

**Recommandation : 16×16×16 comme résolution standard.**

C'est le sweet spot entre détail visuel et coût mesh. Les items 3D dans cet espace donnent un look voxel cartoon premium comparable à Minecraft mais avec plus de détail. Les items très simples (petites ressources, plantes) peuvent utiliser 8×8×8.

### 8.3 Comparaison des stratégies de modèle

| Option | Description | Pour VoxelVerse |
|--------|-------------|-----------------|
| Cube only | Tout est un cube 1m | Trop limité pour items |
| Multi-box | Cubes rectangulaires assemblés (Minecraft JSON) | Bon pour slabs/stairs/clôtures |
| Micro-voxel 16×16×16 | Grille de petits voxels | Parfait pour items et décorations |
| Import .vox | Fichiers MagicaVoxel | Bon pour assets externes |
| Mesh import | .obj/.gltf | Pas voxel, incompatible avec l'esthétique |
| Hybride multi-box + micro-voxel | Multi-box pour géo, micro-voxels pour détail | Trop complexe |

**Recommandation : stratégie progressive.**

```
Terrain/Blocs → Cube 1m (actuel)
Items simples → Micro-voxel 8³ ou 16³ (Phase 4)
Décorations   → Micro-voxel 16³ ou .vox importé (Phase 4)
Structures    → Assemblage de blocs terrain (Phase 6)
Entités/Mobs  → Micro-voxel + animations (Phase 8+)
```

### 8.4 Items : voxel, mesh ou hybride ?

**Voxel micro-grid (16³).** C'est cohérent avec l'esthétique, moddable, et rend bien avec une lumière voxel stylisée. Les items de type "minecraft" (pioches, épées, outils) fonctionnent parfaitement dans cet espace et sont iconiques.

---

## 9. Végétation et plantes

### 9.1 Comparaison des approches

| Approche | Grass | Fleur | Buisson | Feuilles | Tronc | Champignon |
|----------|-------|-------|---------|----------|-------|------------|
| Cross-plane (Minecraft) | ✓✓ | ✓✓ | ✓ | ✓ | — | ✓ |
| Voxel mini-model 8³ | ✓ | ✓✓ | ✓✓ | ✓ | ✓✓ | ✓✓ |
| Multi-box stylized | — | ✓ | ✓✓ | ✓ | ✓✓ | ✓ |
| Mesh instanced | ✓✓ | ✓✓ | ✓✓ | ✓✓ | — | ✓✓ |
| Hybride | ✓✓ | ✓✓ | ✓✓ | ✓✓ | ✓✓ | ✓✓ |

**Recommandations par type :**

| Plante | Approche recommandée | Raison |
|--------|---------------------|--------|
| Grass tuft (herbe haute) | Cross-plane | Simple, performant, iconique |
| Flower | Cross-plane ou voxel 8³ | Cross-plane plus lisible à distance |
| Bush | Cross-plane multi-layer | Dense, performant |
| Tree leaves | Blocs voxel pleins (transparent) | Cohérent avec le monde voxel |
| Tree trunk | Blocs voxel plein | Cohérent |
| Crops | Cross-plane + state | 8 états de croissance |
| Mushroom | Blocs voxel (chapeau + pied) | Cohérent et beau |

**Stratégie recommandée : Cross-plane pour végétation fine + blocs voxel pour végétation volumique.**

Cross-plane avec material alpha-tested. Les feuilles d'arbres restent des blocs voxel transparents car elles interagissent avec le world voxel (lumière, greedy meshing, AOC). Pas de mesh instanced pour l'instant — le rendu voxel avec good LOD et culling sera suffisant.

### 9.2 Performance massive de la végétation

```
Strategy :
- Vegetation blocks dans le chunk comme tous les autres blocs
- Le mesher détecte les blocs cross_plant et génère 2 triangles croisés
- LOD : à distance > 64 voxels, les cross_plants disparaissent
- Les feuilles d'arbres au loin fusionnent en un bloc opaque coloré (LOD fallback)
```

### 9.3 Animation du vent

L'animation du vent doit être gérée dans le shader, pas dans les données :

```wgsl
// Le shader de vegetation reçoit :
// - position du voxel
// - time uniform
// - wind_strength (depuis le biome)
// Il applique un sway vertex shader léger
```

Le `.ron` de végétation déclare seulement `wind_responsive: true`.

---

## 10. Répétition, variation et beauté côte à côte

### 10.1 Le problème des tuiles répétitives

Un `grass_top` répété 100 fois sur une plaine = tiling pattern immédiatement visible. C'est le problème numéro 1 de l'esthétique voxel.

### 10.2 Stratégie en couches

```
Niveau 1 : Variation per-block via seed (actuel dans Studio)
  → seed = hash(pack_seed + block_id + block_seed + face + position)
  → chaque bloc tire une seed différente → variation de teinte et pattern léger

Niveau 2 : Multiple variantes baked (4-8 textures PNG)
  → Le shader sélectionne une variante selon (position_hash % nb_variants)
  → Brisure du pattern sur grande surface

Niveau 3 : Biome tint (futur)
  → Le renderer injecte un tint de couleur biome par vertex interpolé
  → La grass devient plus verte en zones humides, plus jaune en zones sèches

Niveau 4 : Micro-detail shader léger (optionnel)
  → Un micro-noise procédural dans le fragment shader
  → Très léger (1-2 octaves, haute fréquence)
  → Brise le pattern pour les observateurs proches
```

### 10.3 Budget de variantes

| Matériau | Variantes recommandées |
|----------|----------------------|
| Terrain (grass, dirt, stone) | 4-8 |
| Bois (planks, logs) | 4 |
| Ores | 2-4 |
| Briques, pavés | 4-8 |
| Eau, lava | Animée (8-16 frames) |
| Neige, glace | 2-4 |

### 10.4 Garantir la compatibilité entre voisins

Les matériaux doivent être **tileables intrinsèquement** — leurs patterns ne doivent pas créer d'arêtes artificielles entre deux instances.

Règle dans le graph : aucun node ne doit produire une valeur qui dépend de la position absolue UV du bord de tile. Les patterns FBM sont tileables par nature à condition de ne pas utiliser de gradient directionnel fort aux bords.

La variation de teinte (color_jitter) ne casse pas le tiling car elle est appliquée uniformément sur tout le bloc.

### 10.5 Multijoueur et déterminisme

La variation doit être **100% déterministe** :
- Même seed → même résultat, indépendamment du client
- Les seeds sont dérivées de données statiques (position voxel, pack seed, material seed)
- Aucune valeur aléatoire `Math.random()` dans le rendu des blocs

Le Studio doit tester cette propriété : un même pack exporté sur deux machines doit produire une preview identique.

---

## 11. Performance et scalabilité

### 11.1 Scenarios de charge

| Scenario | Blocs | Materials | Graphs | Variantes totales |
|----------|-------|-----------|--------|------------------|
| Pack solo débutant | 50 | 30 | 30 | 120-240 |
| Pack solo avancé | 500 | 300 | 300 | 1 200-2 400 |
| Pack communautaire | 2 000 | 1 200 | 1 200 | 4 800-9 600 |
| Planet avec 10 mods | 10 000+ | 6 000+ | 6 000+ | 24 000-48 000 |

À 48 000 variantes de textures 32×32 (RGBA) = 48 000 × 32 × 32 × 4 bytes = ~196 Mo **non compressé**.  
Avec BC7 compression (8:1) → ~24 Mo. C'est dans les budgets modernes pour un texture array.

À 96×96 (pour preview de qualité) → ~1.7 Go non compressé. Impossible en masse. Stratégie nécessaire.

### 11.2 Budget cible

| Ressource | Budget recommandé par pack |
|-----------|---------------------------|
| Blocs | ≤ 2 000 |
| Materials (faces) | ≤ 1 200 |
| Graphs de nœuds | ≤ 1 200 |
| Variantes par matériau | ≤ 8 |
| Résolution preview | 128×128 (Studio) |
| Résolution runtime | 32×32 (normal) / 64×64 (premium) |
| Résolution LOD | 16×16 |
| Nœuds max par graph | ≤ 64 |
| Connexions max par graph | ≤ 128 |
| Taille cache baked max | 256 Mo par pack |

### 11.3 Cache et lazy loading

```
strategy :
- Chaque graph a un hash de version basé sur le contenu
- Si hash == cache hash → skip la compilation
- Le cache est stocké dans assets/packs/core/baked/
- Les previews du Studio utilisent un cache LRU limité à 256 entrées
- Les graphs sont compilés en batch (background worker dans Tauri)
```

### 11.4 Hot reload

Pour le développement :

```
Studio watches → assets/packs/core/**/*.ron
    │
    ▼
Si changement détecté → recompile le graph concerné
    │
    ▼
Met à jour le cache baked
    │
    ▼
Notifie le jeu via Tauri IPC (futur)
    │
    ▼
Le jeu reload le pack sans restart
```

### 11.5 Mods communautaires non optimisés

Un mod peut soumettre un graph de 500 nœuds. Il faut :
1. Un linter de graph avec avertissements de complexité
2. Un budget dur (N nœuds max) avec erreur de validation
3. Un timeout de compilation (>5s → erreur)
4. Des previews à basse résolution pour les packs non signés

---

## 12. Studio UX final

### 12.1 v0.1 — Materials et Blocks

#### Page Materials

```
┌──────────────────────────────────────────────────────────────────────┐
│ Sidebar  │  Topbar: [Save] [Validate] [Export Pack]                  │
│ Materials│  Breadcrumb: My Pack > Materials > grass_top              │
│ Blocks   ├──────────────────────────────────────────────────────────┤
│          │ Left 280px      │ Center                 │ Right 300px   │
│          │                 │                        │               │
│          │ [+ New]         │ [Graph] [Parameters]   │ Preview 1:1   │
│          │ [From Template] │  [Simple/Expert toggle] │ Preview 4×4   │
│          │ [Import .ron]   │                        │ Cube preview  │
│          │                 │  Graph canvas / Params  │               │
│          │ Filter: [ ]     │  (selon mode)           │ [Randomize]   │
│          │                 │                        │ [Save]        │
│          │ Library list    │                        │               │
│          │ (scrollable)    │                        │ Status badge  │
└──────────────────────────────────────────────────────────────────────┘
```

**New Material flow :**
```
[+ New] → Ouvre dans la zone centrale un canvas vide
           avec nœuds initiaux (Palette, Stylization, Surface, Variation, Output)
           Pas de dialog. Pas de modal. Direct dans l'éditeur.
           Le nom par défaut est "New Material" éditable en haut.
```

**Modes :**

| Mode | Ce qu'on voit | Pour qui |
|------|--------------|---------|
| Simple | 4-6 sliders clés, grande preview | Enfants, débutants |
| Advanced | Tous les paramètres des nœuds, graph visible | Créateurs |
| Expert | Graph complet + RON brut | Power users |

**Pas de modal "New Material" avec grille de presets.** Les templates existent dans une section "Templates" accessible depuis le bouton "From Template".

#### Page Blocks

```
┌──────────────────────────────────────────────────────────────────────┐
│          │ Left 280px      │ Center 460px           │ Right 400px   │
│          │                 │                        │               │
│          │ Block Library   │  Block Builder         │ Cube preview  │
│          │ [+ New]         │  Name/ID               │ 9 variations  │
│          │                 │  Shape selector        │ Validation    │
│          │                 │  Face material picker  │               │
│          │ Filter list     │  Gameplay params       │               │
│          │                 │  [Advanced .ron]       │               │
└──────────────────────────────────────────────────────────────────────┘
```

### 12.2 v0.2 — Models et Items

- Nouvelle route `models` dans la sidebar
- Import de fichiers `.vox` via Tauri file dialog
- Éditeur de propriétés (collision, LOD, taille)
- Preview 3D avec WebGL simple

### 12.3 v0.3 — Vegetation

- Route `vegetation`
- Form de définition d'arbre, buisson, plante
- Preview dans un contexte "planté dans du sol"
- Support cross-plane et blocs voxel

### 12.4 v0.4 — Planet Procedural

- Route `planet`
- Configuration des règles de génération de planète
- Biomes, terrain layers, ores, caves
- Preview de heightmap 2D de la planète

### 12.5 Features UX transversales

**Preview permanente :** La preview est toujours visible. Elle se met à jour dans les 200ms après chaque changement de paramètre (debounced).

**Erreurs inline :** Les erreurs de validation apparaissent directement sur le nœud ou le champ concerné — pas uniquement dans un panneau séparé.

**Bouton Fix :** Chaque erreur fixable a un bouton Fix au niveau du champ, pas seulement dans le panneau global.

**Recherche dans la bibliothèque :**
```
Filtres : nom, namespace, tags, catégorie, status
Tri : récents, alphabétique, status
Search : full-text sur displayName + id + tags
```

**Pas de modales géantes :** Toute action s'ouvre dans la zone centrale, inline, pas dans des dialogs qui masquent le contexte.

---

## 13. Export, validation et compilation

### 13.1 Définition des rôles

| Rôle | Contenu | Où |
|------|---------|-----|
| Source de vérité | `.ron` du graph, `.ron` du bloc, `.ron` du material | `assets/packs/core/` |
| Généré | Texture PNG baked | `assets/packs/core/baked/` |
| Cache Studio | Thumbnails, previews | `~/.voxelverse-studio/cache/` |
| Export dev | Pack complet non optimisé | `export/dev/` |
| Export release | Pack compressé BC7 | `export/release/` |

### 13.2 Pipeline d'export

```
1. Validation
   ├── IDs valides (namespace:path, lowercase)
   ├── Références résolues (tous les materials référencés existent)
   ├── Graphs valides (connexions, pas de cycles, types compatibles)
   ├── Budget de nœuds respecté
   ├── Surfaces PBR dans les plages valides
   └── Pas de doublons

2. Compilation graphs → textures
   ├── Pour chaque graph modifié : évaluation procédurale
   ├── Génération N variantes (configurable, default 4)
   ├── Résolution configurable (32×32 standard, 64×64 premium)
   └── Sauvegarde PNG dans baked/

3. Pack assembly
   ├── Collecte tous les .ron sources
   ├── Résolution de toutes les références
   ├── Génération pack.toml final avec checksums
   └── Optionnel : compression des PNG en BC7 (release only)

4. Rapport d'export
   ├── Nombre de fichiers générés
   ├── Taille totale du pack
   ├── Textures non utilisées (warning)
   ├── Budget de performance estimé
   └── Erreurs bloquantes / warnings non bloquants
```

### 13.3 Export dev vs release

| Feature | Dev | Release |
|---------|-----|---------|
| PNG compression | Non | BC7/ASTC |
| Variantes | 2 | 8 |
| Résolution | 32px | 32-64px selon budget |
| Debug info | Oui | Non |
| Source .ron | Inclus | Non |
| Hot reload | Oui | Non |

### 13.4 Le futur .vvpack

```
my_pack_v1.2.0.vvpack
  manifest.json (version, dependencies, signing)
  blocks/*.ron
  materials/*.ron
  graphs/*.ron
  baked/*.png.bc7
  models/*.vox
  sounds/*.ogg
  lang/*.toml
```

Format : archive zip signée avec clé pack. La signature empêche la modification silencieuse des packs marketplace.

---

## 14. Architecture technique recommandée

### 14.1 Vue d'ensemble

```
┌─────────────────────────────────────────────────────────────┐
│                    FRONTEND React                            │
│                                                              │
│  Pages         : Materials, Blocks, Models, Vegetation...   │
│  Graph Editor  : @xyflow/react (évaluation côté JS ou Wasm) │
│  Preview       : Canvas 2D (actuel) → WebGL (futur)         │
│  State         : React useState / useReducer local           │
│  Persists      : Tauri IPC → fichiers réels (à implémenter)  │
└──────────────────────┬──────────────────────────────────────┘
                       │ Tauri IPC Commands
┌──────────────────────▼──────────────────────────────────────┐
│                    TAURI (Rust backend)                      │
│                                                              │
│  Commandes IPC :                                             │
│  - open_project(path) → PackProject JSON                    │
│  - save_file(path, content) → ()                            │
│  - compile_graph(graph_ron) → BakedResult                   │
│  - validate_pack(path) → Vec<ValidationIssue>               │
│  - export_pack(path, mode) → ExportReport                   │
│  - watch_files(path) → stream FileChangeEvent               │
│                                                              │
│  Crates utilisées :                                          │
│  - vv-content-schema (types RON)                            │
│  - vv-pack-loader (lecture fichiers)                         │
│  - vv-pack-compiler (validation + compilation)              │
│  - vv-texture-gen (nouveau : graph → PNG)                   │
└──────────────────────────────────────────────────────────────┘
```

### 14.2 Crates Rust à créer / modifier

**Existant — à enrichir :**
- `vv-content-schema` : ajouter `MaterialFaceDef`, `ProceduralGraph`, `VoxelModelDef`, `VegetationDef`
- `vv-pack-compiler` : enrichir la validation, ajouter la résolution de MaterialFace → TextureRef

**Nouveau — à créer :**
- `vv-texture-gen` : évaluation des graphs procéduraux Rust, génération PNG baked
  - Implémente les mêmes nodes que le JS evaluator
  - Gère le baking multithread (rayon)
  - Output : PNG + métadonnées (hash, résolution, variantes)

**Nouveau — à créer :**
- `vv-studio-server` : commandes Tauri IPC
  - Pas un serveur réseau — juste les handlers Tauri
  - Bridge entre le frontend React et les crates Rust

### 14.3 Ce qui doit vivre où

| Responsabilité | Frontend React | Tauri Rust |
|----------------|---------------|------------|
| Graph editor UI | ✓ | — |
| Preview canvas CPU | ✓ (actuel) | — |
| Preview WebGL | ✓ (futur) | — |
| Simple Mode UI | ✓ | — |
| RON serialization affichage | ✓ | — |
| RON parsing réel | — | ✓ (vv-pack-loader) |
| Validation complète | — | ✓ (vv-pack-compiler) |
| Graph evaluation/baking | — | ✓ (vv-texture-gen) |
| File watch | — | ✓ (notify crate) |
| Export pack | — | ✓ |
| BC7 compression | — | ✓ |
| Cache gestion | — | ✓ |

### 14.4 Shared types Studio ↔ moteur de jeu

Le format des `.ron` doit être unifié. Les types Rust dans `vv-content-schema` sont la source de vérité. Le Studio doit générer des fichiers que le moteur peut lire sans transformation.

**Action immédiate requise :** aligner le format RON exporté par le Studio avec `RawBlockDef` et définir `MaterialFaceDef` dans `vv-content-schema`.

### 14.5 Future MCP / Agent IA

Le Studio exposera une API MCP via Tauri pour permettre à des agents IA de :
- Lister les matériaux/blocs
- Créer un matériau depuis un prompt
- Valider un pack
- Suggérer des fixes

```json
// Exemple de tool MCP futur
{
  "tool": "create_material",
  "params": {
    "name": "Blue Crystal",
    "style": "clean_stylized",
    "base_color": "#4488FF"
  }
}
```

---

## 15. Roadmap par étapes

### Phase 1 — Remise à plat Materials et Blocks *(Priorité absolue)*

**Objectifs :**
- Supprimer `MaterialKind` comme union fermée — remplacer par `category: string`
- Implémenter New Material comme canvas vide (pas de dialog preset)
- Créer la section "Templates" séparée avec les 19 anciens presets
- Aligner le format RON exporté par le Studio sur `vv-content-schema`
- Ajouter les modes Simple / Advanced dans le Material editor

**Livrables :**
- `MaterialFaceDef` sans `materialKind` fermé
- Canvas vide par défaut
- `studio.ts` allégé
- RON compatible `RawBlockDef`

**Risques :**
- Casser la persistance `localStorage` → gérer avec migration schemaVersion

**Ne pas coder :**
- Tauri IPC (trop tôt)
- Graph evaluation réelle (trop tôt)
- Système de templates avancé

---

### Phase 2 — Graph Blueprint fonctionnel

**Objectifs :**
- Implémenter la vraie évaluation de graph (propagation de valeurs)
- Connecter les nodes par types (Color, Float, Mask)
- Supprimer la dépendance entre graph et recipe (ne pas avoir les deux)
- Le graph est la seule source de vérité — la recipe disparaît ou devient un dérivé en lecture seule

**Livrables :**
- Engine d'évaluation de graph JS côté Studio
- Ports typés sur les nœuds
- Preview recalculée depuis le graph réel
- Nodes v0.1 complets (voir §5.3)

**Risques :**
- Perf de l'évaluateur JS pour les graphs complexes → WebWorker

**Ne pas coder :**
- Nodes custom de mods
- Animation
- Biome tint

---

### Phase 3 — RON Schema propre

**Objectifs :**
- Ajouter `MaterialFaceDef` et `ProceduralGraph` dans `vv-content-schema`
- Aligner tous les formats Studio ↔ moteur
- Implémenter le vrai RON parser dans le Studio (via Tauri)
- Supprimer `schemaVersion` comme guard fragile — remplacer par version explicite dans le `.ron`

**Livrables :**
- `vv-content-schema` avec tous les types Studio
- Parser RON côté Tauri
- Studio lit et écrit de vrais fichiers sur le disque
- Pas de localStorage pour les projets

**Risques :**
- Breaking change sur les `.ron` existants dans `assets/packs/core/`

**Ne pas coder :**
- Migrations complexes entre versions

---

### Phase 4 — Validation/Export réel

**Objectifs :**
- Tauri IPC complet
- `vv-pack-compiler` enrichi pour valider les nouvelles entités
- Export pack dev et release
- Cache PNG baked depuis les graphs
- Rapport d'export complet

**Livrables :**
- Pack `.ron` complet lisible par le moteur de jeu
- Workflow : éditer dans le Studio → jouer immédiatement dans le jeu
- Hot reload

**Risques :**
- BC7 compression : choisir une crate Rust (texture-compress, squish)
- Performance du baking pour des milliers de materials

---

### Phase 5 — Model Editor minimal

**Objectifs :**
- Route Models dans le Studio
- Import `.vox` via Tauri
- Propriétés : collision, LOD, taille
- Preview 3D WebGL basique (Three.js ou WGPU WebAssembly)
- Export `VoxelModelDef` en `.ron`

**Ne pas coder :**
- Éditeur voxel intégré (trop ambitieux)
- Animation des modèles

---

### Phase 6 — Vegetation

**Objectifs :**
- Route Vegetation
- Forme des arbres, buissons, plantes via form
- Preview en contexte (sol + plante)
- Export `VegetationDef` en `.ron`

---

### Phase 7 — Planet Procedural

**Objectifs :**
- Route Planet
- Configuration des règles procédurales (climate, biomes, terrain, ores)
- Preview heightmap 2D de la planète
- Import/export vers `vv-content-schema` procédural

---

### Phase 8 — Connexion metaverse

**Objectifs :**
- API pour accéder aux planètes joueur
- Envoi de packs vers une planète
- Validation de compatibilité
- Signature de packs

---

## 16. Décisions recommandées

### Final Decisions

| Question | Décision |
|----------|---------|
| **New Material doit-il ouvrir un canvas vide ?** | **Oui.** Plus de dialog preset. Canvas avec structure initiale minimale. |
| **Les presets doivent-ils devenir des templates ?** | **Oui.** Section "Templates" séparée, non bloquante, non obligatoire. |
| **Le PNG doit-il disparaître ou rester cache ?** | **Reste comme cache compilé.** Source = graph `.ron`, cache = PNG baked. |
| **Chaque face doit-elle avoir son propre `.ron` ?** | **Oui.** Réutilisabilité, override par mods, compilation indépendante. |
| **Faut-il du procedural pour tous les blocs ?** | **Non.** La plupart des blocs ont juste besoin d'une référence material. Le procedural est dans le material, pas dans le bloc. |
| **Faut-il un éditeur voxel intégré ?** | **Pas maintenant.** Import `.vox` d'abord. Éditeur intégré en Phase 7+. |
| **Quelle taille micro-voxel recommander ?** | **16×16×16** pour items standards. 8³ pour items simples. |
| **Cross-plane ou .vox pour végétation ?** | **Cross-plane** pour végétation fine (grass, fleurs). **Blocs voxel** pour végétation volumique (feuilles, troncs). |
| **Comment gérer les planks ?** | **Semi-procedural baked.** Graph avec `PlanksNode` + grain FBM, compilé vers 4-8 variantes PNG. |
| **Comment gérer les items ?** | **Micro-voxel 16³.** Import `.vox` ou éditeur futur. |
| **Comment gérer la variation ?** | **4 niveaux** : seed per-block + 4-8 variantes baked + biome tint + micro-detail shader léger. |
| **Comment garder la simplicité ?** | **Mode Simple** masque le graph. Expose 4-6 paramètres clés. Preview permanente. Pas de modal géante. |
| **Quelle est la meilleure architecture maintenant ?** | **React + @xyflow pour le graph, Tauri IPC pour l'I/O fichier réel, vv-texture-gen (nouveau crate Rust) pour le baking, vv-content-schema unifié comme source de vérité.** |

---

### Prochaines tâches concrètes

**Semaine 1 :**
1. Supprimer `MaterialKind` comme union fermée dans `studio.ts`
2. Remplacer le dialog `MaterialWizard` par un canvas inline vide
3. Créer le composant `TemplateGallery` séparé avec les 19 anciens presets
4. Aligner le format RON exporté avec `RawBlockDef` du moteur Rust

**Semaine 2 :**
1. Implémenter l'évaluation réelle du graph (propagation de valeurs)
2. Supprimer `ProceduralMaterialRecipe` comme concept parallèle — le graph devient la seule source
3. Implémenter les modes Simple / Advanced / Expert

**Semaine 3 :**
1. Ajouter `MaterialFaceDef` et `ProceduralGraph` dans `vv-content-schema`
2. Créer le crate `vv-texture-gen` (baking Rust)
3. Premiers handlers Tauri IPC (open_project, save_file, compile_graph)

**Semaine 4 :**
1. Connecter le Studio aux vrais fichiers dans `assets/packs/`
2. Supprimer `localStorage` comme système de persistance principal
3. Test end-to-end : créer un matériau → exporter → charger dans le moteur de jeu

---

## Annexe : Problèmes architecturaux classés par urgence

| Urgence | Problème | Impact |
|---------|---------|--------|
| 🔴 Critique | Format RON Studio ≠ format RON moteur | Rien n'est jouable |
| 🔴 Critique | `MaterialKind` union fermée | Impossible à modder |
| 🔴 Critique | Pas de Tauri IPC | Studio = éditeur virtuel déconnecté |
| 🟠 Élevé | Graph = cosmétique (pas d'évaluation réelle) | La feature principale est fausse |
| 🟠 Élevé | `ProceduralMaterialRecipe` + `MaterialBlueprint` = deux vérités | Désync possible |
| 🟠 Élevé | `ronParser.ts` ne parse rien | Import `.ron` impossible |
| 🟡 Moyen | Dialog "New Material" avec presets obligatoires | UX bloquante |
| 🟡 Moyen | Preview CSS isométrique vs WebGL | Qualité insuffisante long terme |
| 🟡 Moyen | `schemaVersion` sans migration | Cassures localStorage silencieuses |
| 🟢 Faible | `initialProject.ts` avec données hardcodées | Acceptable pour démo, pas pour prod |
| 🟢 Faible | Pas de recherche dans les bibliothèques | Gênant au-delà de 50 items |

---

*Rapport rédigé après analyse complète de :*
- *`apps/voxelverse-studio/src/` (tous les fichiers)*
- *`crates/vv-content-schema/src/` (block.rs, visual.rs, procedural.rs)*
- *`assets/packs/core/` (blocs .ron, textures, structure)*
- *`docs/ARCHITECTURE.md`*
- *`AGENTS.md`*

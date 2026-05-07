# VoxelVerse - Plan directeur de fondation

Ce document decrit le plan de construction de VoxelVerse a partir de l'etat reel du repo.
Il complete `AGENTS.md` avec une trajectoire concrete, des gates de qualite et un ordre de travail strict.

Le but n'est pas d'empiler des features. Le but est de construire une base moteur assez saine pour porter plusieurs annees de developpement sans dette structurelle.

## 1. Vision executable

VoxelVerse doit devenir un sandbox voxel d'exploration, construction et survie legere sur petites planetes rondes vivantes.

La promesse centrale est:

- marcher sur un petit monde rond coherent;
- voir tres loin autour de soi;
- explorer des biomes lisibles;
- miner et construire simplement;
- transformer la planete;
- ajouter du contenu par donnees et mods, sans changer le moteur.

La base doit donc privilegier:

- stabilite planetaire;
- runtime voxel compact;
- data-driven complet;
- rendu lointain performant;
- architecture lisible par plusieurs agents;
- fichiers courts;
- diagnostics permanents;
- suppression active des doublons et du legacy.

## 2. Etat actuel du repo

Etat observe apres lecture de `AGENTS.md`, `README.md` et `src/`.

### Deja en place

- Architecture par dossiers:
  - `content`
  - `diagnostics`
  - `gameplay`
  - `generation`
  - `input`
  - `physics`
  - `rendering`
  - `voxel`
  - `world`
- Runtime voxel initial:
  - `VoxelId`
  - `VoxelCoord`
  - `VoxelChunk`
  - `VoxelChunkKey`
  - `VoxelRuntime`
- Planete ronde initiale:
  - `PlanetProfile`
  - projection cube-sphere
  - heightmap deterministe
  - gravite vers le centre
  - raycast voxel
  - collisions spheriques
- Rendu wgpu:
  - chunks voxel proches
  - LOD terrain lointain
  - ombres
  - fog
  - console texte
  - debug simple
- Tests minimaux:
  - monotonie rayon/couche
  - roundtrip coordonnees cube-sphere

### Problemes critiques actuels

- `src/rendering/renderer.rs` depasse 1000 lignes. C'est le blocage numero 1.
- `src/generation/mod.rs` approche 800 lignes. Il doit etre separe avant ajout de worldgen avancee.
- Le contenu est encore partiellement hardcode:
  - `voxelverse:core`
  - `voxelverse:dirt`
  - `voxelverse:grass`
  - couleurs et registres builtin.
- Il n'existe pas encore de pipeline raw data -> validation -> compilation -> runtime registry.
- Le meshing est encore mele a `generation`.
- Le renderer possede trop de responsabilites:
  - init GPU
  - pipelines
  - streaming
  - LOD
  - mesh upload
  - debug draw
  - texte
  - shadow pass
  - main pass
  - console mesh
- Le spawning de threads est direct et non gouverne par un vrai scheduler/budget.
- Les diagnostics existent mais ne couvrent pas encore les budgets critiques:
  - temps meshing
  - upload GPU par frame
  - temps worldgen
  - chunks visibles vs charges
  - memoire runtime voxel
- Le README est plus marketing que reflet exact de la base actuelle.

### Conclusion d'etat

Le projet est en fondation avancee mais pas encore pret pour exploration, biomes, outils, inventaire, mobs ou structures.

La prochaine phase doit etre une phase de durcissement moteur, pas une phase de gameplay.

## 3. Definition de "base parfaite" pour VoxelVerse

La base est consideree saine uniquement quand toutes les conditions suivantes sont vraies.

### Compilation et verification

- `cargo fmt --check` passe.
- `cargo clippy -- -D warnings` passe.
- `cargo test` passe.
- `cargo build` passe.
- Aucun warning Rust.
- Aucun fichier Rust au-dessus de 1000 lignes.
- Aucun fichier au-dessus de 800 lignes sans plan de split immediat.

### Architecture

- Chaque dossier a une responsabilite unique.
- Aucun fichier `common.rs`, `utils.rs`, `manager.rs` ou equivalent vague.
- Le meshing n'est pas dans generation.
- Le renderer ne decide pas du monde.
- Le runtime monde ne lit pas de fichiers de contenu.
- Le gameplay ne hardcode pas les blocs.
- L'input produit des intentions, pas des regles.
- Les diagnostics sont separes du gameplay et du rendu gameplay.

### Donnees et contenu

- Aucun bloc concret n'est verite moteur permanente.
- Les identifiants runtime sont compacts.
- Les noms de contenu sont path-as-identity.
- Le runtime ne stocke pas de strings par voxel.
- Les erreurs de contenu sont reportees clairement.
- Le contenu core peut etre recharge depuis des fichiers raw valides.

### Performance

- Pas de rebuild global pour une modification locale.
- Pas de spawn de threads illimite.
- Pas d'allocations massives par frame.
- Budget de generation mesh.
- Budget d'upload GPU.
- Culling stable.
- LOD stable.
- Streaming priorise par camera/joueur.

### Planete

- Coordonnees stables sur les six faces.
- Pas de trous entre faces.
- Gravite stable.
- Raycast fiable.
- Collisions fiables.
- Spawn fiable.
- Surface agreable a parcourir.
- Relief deterministe.
- Base compatible grands mondes voxel spheriques.

## 4. Gates obligatoires avant toute feature

Ces gates doivent etre traites dans l'ordre.

### Gate 0 - Hygiene de repo

Objectif: rendre l'etat du repo explicite et fiable.

Travaux:

- Mettre `AGENTS.md` sous controle de version.
- Ajouter `PLAN.md`.
- Mettre a jour `README.md` pour refleter l'etat reel, pas une promesse trop avancee.
- Ajouter une commande de verification documentee:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - `cargo build`
- Ajouter une convention de line count:
  - aucun fichier > 1000 lignes;
  - alerte a 800 lignes.

Definition de termine:

- Le statut git ne contient que des changements intentionnels.
- La documentation de base correspond au repo.

### Gate 1 - Split renderer

Objectif: sortir `renderer.rs` de l'etat fichier geant.

Le renderer doit etre decoupe en modules:

- `rendering/device.rs`
  - instance wgpu
  - adapter
  - device
  - queue
  - surface config
- `rendering/pipelines.rs`
  - pipeline fill
  - pipeline wire
  - pipeline line
  - pipeline ui
  - pipeline shadow
- `rendering/buffers.rs`
  - creation buffers
  - uniform buffers
  - buffer upload helpers
- `rendering/render_world.rs`
  - main pass
  - shadow pass
  - draw chunks
  - draw LODs
- `rendering/render_ui.rs`
  - console mesh
  - text renderer
  - FPS text
- `rendering/debug_draw.rs`
  - cursor voxel
  - collision debug
  - future frustum debug
- `rendering/streaming.rs`
  - load queue
  - pending chunks
  - pending LODs
  - mesh receive/upload budgets
- `rendering/lod_selection.rs`
  - quadtree selection
  - LOD factors
  - visible sets
- `rendering/renderer.rs`
  - facade courte
  - orchestration seulement

Contraintes:

- Aucun nouveau comportement gameplay.
- Pas de duplication de pipeline.
- Pas de wrapper temporaire inutile.
- Chaque module < 500 lignes si possible.

Definition de termine:

- `renderer.rs` < 400 lignes.
- Aucun module rendering > 800 lignes.
- `cargo clippy -- -D warnings` passe.

### Gate 2 - Extraire meshing

Objectif: separer generation de monde et extraction mesh.

Nouveaux modules cibles:

- `meshing/mod.rs`
- `meshing/voxel_mesher.rs`
- `meshing/lod_mesher.rs`
- `meshing/collision_debug_mesh.rs`
- `meshing/cpu_mesh.rs`
- `meshing/mesh_bounds.rs`
- `meshing/ambient_occlusion.rs`

Responsabilites:

- `generation` produit des hauteurs, biomes, profils, champs de densite plus tard.
- `world` repond aux requetes voxel.
- `meshing` transforme des voxels et surfaces en vertices/indices CPU.
- `rendering` upload et affiche.

Definition de termine:

- `generation/mod.rs` < 400 lignes.
- Mesh voxel et LOD ne dependent pas de wgpu.
- Les types CPU mesh sont propres.
- Tests sur bounds et indices basiques.

### Gate 3 - Core et math propres

Objectif: sortir les maths pures des systemes.

Modules cibles:

- `core`
  - erreurs communes minimales;
  - ids generiques si necessaire.
- `math`
  - ray;
  - frustum;
  - sphere;
  - cube sphere mapping;
  - intersections;
  - bounds.

Deplacements:

- `Frustum` quitte `rendering/types.rs`.
- projection cube-sphere quitte `generation/mod.rs`.
- logique de ray future quitte controller.

Definition de termine:

- `generation` utilise `math::cube_sphere`.
- `rendering` utilise `math::frustum`.
- tests math sans dependance wgpu/winit.

### Gate 4 - Intentions input et actions gameplay

Objectif: clarifier la boucle jeu.

Modules cibles:

- `input/input_state.rs`
- `input/action.rs`
- `gameplay/actions.rs`
- `gameplay/player_controller.rs`
- `gameplay/block_interaction.rs`

Regle:

- input lit clavier/souris;
- input produit intentions;
- gameplay valide miner/placer;
- world applique modification;
- rendering observe le resultat.

Definition de termine:

- `main.rs` ne contient plus les regles de minage/placement.
- Le controller ne fait plus directement de logique monde.
- Les actions sont testables hors winit.

### Gate 5 - Diagnostics moteur

Objectif: rendre le moteur observable.

Diagnostics obligatoires:

- FPS.
- Frame time.
- Position joueur.
- Coord voxel ciblee.
- Chunks voxel actifs.
- LOD actifs.
- Chunks en attente.
- Mesh jobs en vol.
- Uploads GPU par frame.
- Vertices/indices visibles.
- Estimation memoire CPU/GPU.
- Temps meshing moyen/max.
- Temps selection LOD.
- Temps render passes.
- Distance de rendu effective.

Implementation cible:

- `diagnostics/metrics.rs`
- `diagnostics/frame_stats.rs`
- `diagnostics/debug_overlay.rs`
- `diagnostics/log.rs`

Definition de termine:

- overlay debug active par commande ou touche debug.
- aucun diagnostic critique encode dans rendering directement.

## 5. Fondation data-driven

Cette phase est obligatoire avant d'ajouter biomes, ressources, recettes, outils ou blocs nouveaux.

### Phase 1 - Schema raw

Ajouter:

- `content/schema/block.rs`
- `content/schema/item.rs`
- `content/schema/texture.rs`
- `content/schema/tag.rs`
- `content/schema/planet.rs`
- `content/schema/biome.rs`
- `content/schema/mod.rs`

Formats autorises:

- RON ou TOML au debut.
- JSON seulement si utile pour tooling.

Definitions minimales:

- `RawBlockDef`
  - solid
  - hardness
  - visual
  - tags
  - drops plus tard
- `RawBlockVisual`
  - texture all
  - texture top/bottom/side
  - tint optionnel
- `RawPlanetDef`
  - resolution
  - seed
  - radius profile
  - terrain params
- `RawBiomeDef`
  - temperature range
  - humidity range
  - surface block refs
  - color tint

Interdit:

- ajouter `grass`, `dirt`, `core` comme constantes moteur definitives.

### Phase 2 - Pack loading

Ajouter:

- `content/pack/manifest.rs`
- `content/pack/loader.rs`
- `content/pack/path_identity.rs`
- `content/pack/source.rs`

Structure cible:

```text
packs/
  core/
    pack.toml
    blocks/
      air.ron
      core.ron
      dirt.ron
      grass.ron
    textures/
      blocks/
    planets/
      small_living_world.ron
    biomes/
```

Regles:

- id derive du chemin: `core:blocks/dirt` ou `core:dirt` selon convention finale.
- aucune double verite id dans le fichier.
- erreurs de parsing localisees avec chemin et champ.

### Phase 3 - Compilation de contenu

Ajouter:

- `content/compile/compiler.rs`
- `content/compile/diagnostics.rs`
- `content/compile/resolver.rs`
- `content/compile/tags.rs`
- `content/compiled/block.rs`
- `content/compiled/planet.rs`
- `content/compiled/biome.rs`

Le compilateur doit:

- valider les references;
- construire les tags;
- resoudre textures;
- creer les ids runtime compacts;
- produire diagnostics lisibles;
- refuser les fallbacks silencieux.

### Phase 4 - Runtime registries

Remplacer `VoxelRegistry::builtin()` par:

- `BlockRegistry`
- `ItemRegistry`
- `TextureRegistry`
- `BiomeRegistry`
- `PlanetRegistry`

Contraintes runtime:

- ids compacts;
- tables contigues;
- lookups rapides par id;
- lookups par key uniquement hors chemins chauds;
- monde stocke seulement ids runtime.

Definition de termine data-driven:

- le jeu demarre depuis `packs/core`;
- supprimer un bloc raw utile donne une erreur claire;
- modifier une couleur/texture dans les donnees change le rendu sans code;
- `grass/dirt/core` ne sont plus hardcodes comme contenu permanent.

## 6. Fondation voxel et monde

### Objectif

Faire du runtime voxel une base durable pour grands mondes spheriques.

### Travaux

- Clarifier deux niveaux de chunk:
  - chunk voxel 3D runtime;
  - tile surface/LOD pour rendu planetaire.
- Eviter la confusion entre `ChunkKey` surface et `VoxelChunkKey` 3D.
- Renommer si necessaire:
  - `SurfaceChunkKey`
  - `VoxelChunkKey`
  - `LodTileKey`
- Ajouter `VoxelRead` et `VoxelWrite` traits si utile.
- Ajouter batch modification:
  - placer plusieurs voxels;
  - annuler plus tard;
  - marquer dirty regions.
- Ajouter dirty tracking:
  - chunk touche;
  - voisins touches;
  - mesh rebuild local.
- Ajouter tests:
  - set/get voxel;
  - override AIR sur terrain genere;
  - override bloc sur air;
  - suppression d'override si retour au voxel genere;
  - chunk vide supprime.

### Definition de termine

- modification d'un voxel ne rebuild que les chunks necessaires;
- world ne connait pas le rendu;
- rendering ne connait pas les regles de generation;
- meshing lit via interface claire.

## 7. Fondation planete ronde

### Objectif

La planete doit etre le coeur stable du jeu, pas un effet visuel.

### Travaux

- Stabiliser `PlanetProfile`:
  - seed;
  - radius;
  - layer height;
  - core layers;
  - terrain amplitude;
  - biome scale;
  - gravity strength plus tard.
- Deplacer cube-sphere dans `math`.
- Ajouter tests:
  - roundtrip sur centres et bords;
  - continuite entre faces;
  - rayon couche monotone;
  - spawn toujours au-dessus du sol;
  - raycast ne traverse pas le noyau;
  - collision stable sur poles et aretes de faces.
- Introduire un `PlanetAddress` si necessaire:
  - face;
  - surface coord;
  - layer;
  - planet id futur.
- Preparer multi-planetes sans l'implementer:
  - ne pas hardcoder une seule planete partout;
  - garder `PlanetData` propre.

### Generation jouable

Le monde doit produire:

- zones plates constructibles;
- collines douces;
- montagnes lisibles;
- silhouettes visibles a distance;
- transitions naturelles;
- pas de bruit chaotique partout.

Approche:

- macro relief basse frequence;
- detail secondaire faible;
- masque de zones plates;
- biome map plus tard;
- landmarks plus tard.

## 8. Chunks, streaming et LOD

### Objectif

Voir tres loin sans brute force.

### Travaux

- Creer un module `streaming`.
- Creer un scheduler de jobs:
  - pas de `std::thread::spawn` direct par chunk;
  - pool borne;
  - priorites;
  - cancellation ou generation obsolescente ignoree.
- Unifier budgets:
  - jobs mesh par frame;
  - uploads GPU par frame;
  - LOD creations par frame;
  - destructions par frame.
- Stabiliser LOD:
  - hysteresis;
  - transitions;
  - pas de popping violent;
  - pas de trous.
- Ajouter dirty chunk pipeline:
  - modification voxel;
  - marquage dirty;
  - meshing async;
  - upload budgete.

### Definition de termine

- camera immobile = aucun churn de chunks;
- mouvement joueur = streaming borne;
- edition locale = rebuild local;
- diagnostics montrent jobs et budgets.

## 9. Meshing performant

### Objectif

Rendre beaucoup de terrain proprement.

### Etapes

- Extraire meshing CPU.
- Ajouter `CpuMesh`.
- Ajouter mesh bounds fiable.
- Optimiser allocation:
  - reuse buffers;
  - preallocation;
  - pas de HashSet voxel massif dans hot path si remplaçable.
- Introduire greedy meshing proche.
- Garder LOD heightmap separe.
- Preparer atlas UV:
  - vertex inclut uv;
  - texture index;
  - face material id.

### Tests

- cube seul = 6 faces.
- cube entoure = 0 face.
- deux cubes adjacents = face interne supprimee.
- bounds corrects.
- indices multiples de 3 pour triangles.

## 10. Rendering base AAA stylisee

### Objectif

Un rendu propre, lisible, lumineux, performant.

### Priorites

- split renderer avant tout.
- texture atlas data-driven.
- shader clair:
  - world pass;
  - shadow pass;
  - ui/text pass;
  - debug pass.
- materials simples:
  - albedo atlas;
  - tint;
  - fog;
  - shadows.
- camera stable.
- shadow stable.
- fog planetaire doux.
- debug modes propres.

### A ne pas faire trop tot

- PBR complexe.
- post-process lourd.
- microdetails.
- effets organiques avant atlas/textures.

### Definition de termine

- rendu voxel texture;
- LOD lointain coherent avec palette/biome;
- debug overlay utilisable;
- renderer module par responsabilite.

## 11. Gameplay minimal seulement apres fondation

Le gameplay ne commence vraiment qu'apres les gates moteur.

### Gameplay V0

- marcher;
- sauter;
- miner;
- placer;
- selection bloc simple;
- hotbar minimale;
- feedback visuel de ciblage;
- debug creative mode.

### Gameplay V1

- inventaire minimal;
- outils;
- durete blocs;
- drops;
- recettes simples;
- resources.

### Gameplay V2

- survie legere;
- danger lisible;
- jour/nuit si utile;
- mobs plus tard.

Regle:

Le gameplay consomme registries et monde. Il ne definit pas le contenu.

## 12. Exploration et biomes

`AGENTS.md` mentionne explicitement exploration et biomes. Ils sont importants, mais pas maintenant.

### Preconditions obligatoires

- pipeline content fonctionne;
- biome schema existe;
- planet generation accepte des biomes compiles;
- renderer peut afficher tint/texture par biome;
- diagnostics worldgen disponibles.

### Biomes V0

- grassland;
- hills;
- rocky;
- forest placeholder sans arbres complexes;
- color tint;
- surface block refs.

### Exploration V0

- silhouettes lointaines;
- points hauts;
- zones plates;
- petites variations memorables;
- debug map biome/height.

### Biomes V1

- ressources;
- decorations simples;
- transitions;
- landmarks;
- structures plus tard.

## 13. Roadmap pluriannuelle

### Annee 0 - Fondation technique

But: base moteur propre.

- gates 0 a 5 terminees;
- data-driven V0;
- split renderer;
- meshing separe;
- runtime monde/voxel teste;
- planet profile teste;
- diagnostics utiles.

Livrable:

- marcher, miner, placer sur petite planete ronde;
- rendu propre basique;
- contenu core charge depuis donnees;
- aucune dette structurelle majeure.

### Annee 1 - Moteur jouable

But: boucle sandbox minimale.

- atlas textures;
- blocs data-driven;
- outils;
- hotbar;
- inventaire minimal;
- chunks streaming robustes;
- LOD stable;
- biomes V0;
- generation plus composee;
- debug tooling.

Livrable:

- une planete jouable pendant 30 minutes;
- construction/minage agreables;
- exploration simple;
- framerate stable.

### Annee 2 - Monde vivant

But: profondeur de contenu.

- ressources;
- recettes;
- structures simples;
- decorations;
- biomes V1;
- cycle visuel;
- sons;
- UI propre;
- sauvegarde/chargement robuste.

Livrable:

- petite boucle survie/construction;
- monde transformable;
- premiers packs/mods simples.

### Annee 3 - Qualite production

But: polish, scale, modding.

- mod loading complet;
- diagnostics contenu avancés;
- sauvegardes versionnees;
- outils createurs;
- optimisation GPU/CPU;
- rendu final stylise;
- QA longue duree;
- compatibilite packs controlee si le jeu commence a sortir.

Livrable:

- vertical slice solide;
- base modding credible;
- experience visuelle forte.

## 14. Ordre immediat des prochaines taches

Priorite stricte:

1. Split `src/rendering/renderer.rs`.
2. Extraire `meshing` depuis `generation`.
3. Extraire `math` depuis generation/rendering.
4. Mettre a jour `README.md` pour l'etat reel.
5. Ajouter diagnostics frame/chunk/upload.
6. Introduire content schema raw.
7. Introduire pack loading.
8. Introduire content compiler.
9. Remplacer `VoxelRegistry::builtin()` par registry compilee.
10. Ajouter atlas texture data-driven.
11. Stabiliser streaming/job scheduler.
12. Ajouter dirty tracking voxel.
13. Ajouter gameplay actions propres.
14. Ajouter hotbar minimale.
15. Commencer biomes V0.

Interdiction:

Ne pas commencer les biomes, mobs, outils avances, structures ou inventaire complet avant les points 1 a 9.

## 15. Checklist agent avant chaque changement

Avant de coder:

- Lire `AGENTS.md`.
- Lire `PLAN.md`.
- Identifier le proprietaire du concept.
- Verifier s'il existe deja un type canonique.
- Refuser toute duplication.
- Refuser tout hardcoding de contenu permanent.
- Verifier les fichiers proches de 800 lignes.
- Prevoir tests si systeme critique.

Apres avoir code:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`
- verifier line count;
- verifier qu'aucun fichier > 1000 lignes;
- decrire les changements et les risques.

## 16. Definition de non-negociable

Une modification est refusee si elle:

- augmente `renderer.rs`;
- ajoute du contenu hardcode au moteur;
- cree un second systeme de coordonnees;
- ajoute un loader sans validation;
- melange meshing et rendu GPU;
- melange input et gameplay;
- ajoute un fallback silencieux sur contenu invalide;
- ajoute une feature gameplay avant les gates de fondation;
- rend le prochain agent moins capable de continuer.

## 17. Definition de succes

La base de VoxelVerse sera consideree prete pour le vrai jeu quand:

- le joueur marche sur une planete ronde stable;
- le moteur voit loin sans churn;
- le runtime voxel est compact et teste;
- le contenu core vient de packs;
- le renderer est modulaire;
- le meshing est separe;
- les diagnostics expliquent le moteur;
- les fichiers restent courts;
- les agents peuvent continuer sans deviner l'architecture;
- les biomes peuvent etre ajoutes par donnees;
- l'exploration peut etre construite sur une planete deja saine.

Jusque-la, la priorite est la fondation.

Construire peu. Construire propre. Construire durable.

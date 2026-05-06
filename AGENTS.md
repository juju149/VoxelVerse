VoxelVerse - Constitution permanente du projet

Ce fichier est la source de vérité pour tous les agents IA, développeurs, refactors et itérations du projet VoxelVerse.

Il doit être lu avant toute modification du code. Il définit le jeu que l'on veut construire, les règles d'architecture, les limites techniques, les objectifs visuels, les règles de performance et la manière dont un agent doit continuer le travail sans casser la vision.

VoxelVerse repart de zéro. Il n'y a pas de legacy à préserver. Il n'y a pas de compatibilité avec un ancien système à maintenir. Il n'y a pas de rustines acceptables.

Le but est de construire un moteur voxel moderne, data-driven, performant et beau, inspiré de Minecraft dans sa simplicité, mais avec une identité forte : des planètes rondes, une immense distance de rendu, un monde vivant, lisible et moddable.

1. Vision du jeu

VoxelVerse est un sandbox voxel d'exploration, construction et survie légère.

La base du jeu est simple :

Un Minecraft moderne sur de petites planètes rondes, avec un rendu stylisé, une distance de vue immense, une construction fluide et un système de contenu entièrement data-driven.

Le joueur doit ressentir :

La liberté de Minecraft
Le plaisir de construire rapidement
La curiosité d'explorer une planète entière visible autour de lui
La satisfaction d'un monde lisible, coloré, cohérent et performant
L'envie de transformer un petit monde rond en territoire personnel

VoxelVerse ne cherche pas à être le jeu avec le plus de systèmes. VoxelVerse cherche à être plus clair, plus beau, plus fluide, plus moddable et plus agréable à jouer.

2. Référence de gameplay

La référence de base est Minecraft, mais adaptée à l'identité VoxelVerse.

On garde :

Des blocs lisibles
Des textures 2D stylisées
Une construction simple
Du minage
De l'exploration
Des biomes
Des ressources
Des outils
Des recettes
Des mobs plus tard
Des structures plus tard
Une logique sandbox très compréhensible

On change :

Le monde n'est pas une map plate infinie
Le monde est composé de planètes rondes
La distance de rendu doit être immense
La planète doit être lisible à grande échelle
La construction doit être plus rapide et moins répétitive
Le contenu doit être 100% data-driven
Le moteur doit être pensé dès le début pour les mods
Le rendu doit être stylisé, propre, lumineux et moderne
3. Objectif final de rendu

Le rendu final voulu est :

Voxel stylisé
Textures 2D propres, non réalistes
Arêtes nettes
Formes lisibles
Couleurs travaillées
Ambiance lumineuse forte
Ombres propres
Brouillard atmosphérique doux
Horizon lisible
Planètes rondes visibles
Grande distance de rendu
Transitions LOD discrètes
Très haut framerate
Aucune surcharge visuelle inutile

Le jeu doit ressembler à un Minecraft moderne et artistique sur planète ronde, pas à un prototype technique.

Les blocs doivent utiliser des textures classiques de type atlas, avec une direction cartoon, propre et lisible. Le moteur ne doit pas dépendre d'effets procéduraux complexes pour rendre les blocs beaux. La beauté doit venir de :

Bons assets
Bonne lumière
Bonne palette
Bonne distance de vue
Bon LOD
Bon shader
Bonne composition du terrain
Bonne lisibilité
4. Objectifs techniques prioritaires

Les priorités permanentes du moteur sont :

Architecture propre
Performance
Rendu lointain
Data-driven complet
Modding futur
Lisibilité du code
Absence de duplication
Absence de legacy
Itérations sûres avec agents IA
Qualité visuelle finale

Le moteur doit être conçu pour grandir. Chaque changement doit rendre la suite plus simple, pas plus fragile.

5. Règle absolue : pas de legacy

Le jeu n'est pas sorti. Aucun ancien système ne doit être préservé par peur de casser une compatibilité.

Un agent a le droit de :

Supprimer un ancien fichier
Réécrire un système entier
Renommer une structure
Changer un format de données
Déplacer du code
Fusionner ou séparer des modules
Supprimer des abstractions inutiles

Un agent n'a pas le droit de :

Garder deux systèmes pour la même chose
Ajouter une couche de compatibilité inutile
Laisser un vieux chemin de code actif
Ajouter des adaptateurs temporaires non justifiés
Conserver une mauvaise architecture parce qu'elle compile

Le moteur doit converger vers la meilleure architecture, pas accumuler des fossiles.

6. Règle absolue : data-driven à 100%

VoxelVerse doit être data-driven.

Le contenu doit être défini dans des fichiers de données, pas codé en dur dans le moteur.

Doivent être data-driven :

Blocs
Textures de blocs
Items
Outils
Recettes
Tags
Biomes
Ressources
Ores
Structures
Loot tables
Paramètres de génération
Paramètres de planète
Langues
UI themes
Sons
Entités plus tard
Mobs plus tard
Règles de spawn plus tard

Le code définit les systèmes. Les données définissent le contenu.

Exemples :

Le code sait comment miner
Les données disent quel bloc est minable et avec quel outil
Le code sait comment générer un biome
Les données disent quels biomes existent
Le code sait comment afficher un bloc texturé
Les données disent quelle texture utiliser

Aucun agent ne doit hardcoder grass, dirt, stone, wood, iron, ou tout autre contenu comme vérité moteur permanente.

7. Pipeline de contenu obligatoire

Le contenu ne doit jamais être utilisé brut directement par le runtime.

Le pipeline permanent est :

Raw data
Validation
Compilation
Runtime registry
Runtime engine

Définition :

Raw data : fichiers écrits par les devs ou moddeurs
Validation : vérification stricte des erreurs
Compilation : normalisation et résolution des références
Runtime registry : tables compactes optimisées
Runtime engine : monde, rendu, gameplay, génération

Le runtime ne doit pas lire directement les fichiers de contenu. Le monde ne doit pas stocker des chaînes de caractères par voxel. Le monde doit stocker des identifiants runtime compacts.

8. Path-as-identity

L'identité du contenu vient du chemin du fichier et du namespace du pack.

Exemple :

packs/core/blocks/dirt.ron

Peut devenir :

core:dirt

Un fichier ne doit pas répéter inutilement son propre id si le chemin le définit déjà.

Objectif : éviter les doubles vérités.

9. Architecture cible

Le moteur doit être séparé en responsabilités claires.

Les couches conceptuelles sont :

Core
Math
Voxel runtime
World runtime
Meshing
Rendering
Input
Physics
Content schema
Pack loading
Content compilation
Runtime registries
World generation
Gameplay
UI
Diagnostics

Chaque couche doit avoir une responsabilité claire. Aucune couche ne doit devenir un tiroir à bazar.

10. Responsabilités des couches
Core

Contient uniquement les concepts partagés minuscules :

Erreurs génériques
Identifiants génériques
Types utilitaires très bas niveau

Core ne doit pas connaître :

Les blocs
Le joueur
La planète
Le rendu
Les recettes
Le gameplay
Math

Contient les maths pures :

Vecteurs spécialisés si nécessaire
Rayons
AABB
Plans
Frustum
Intersections
Géométrie pure

Math ne doit pas connaître le gameplay.

Voxel runtime

Contient le langage voxel bas niveau :

Coordonnées voxel
Coordonnées chunk
Faces
Directions
Tailles de chunk
Block runtime ids
Stockage voxel compact

Ne doit pas connaître les blocs concrets.

World runtime

Contient l'état du monde :

Planètes
Chunks actifs
Données modifiées
Accès lecture/écriture
Requêtes de monde
Stockage compact

Ne doit pas charger les packs. Ne doit pas parser les fichiers.

Meshing

Transforme les voxels en mesh.

Contient :

Greedy meshing
Mesh extraction
Mesh chunk
Frontières de chunks
Données de mesh CPU

Ne fait pas le rendu GPU. Ne décide pas du gameplay.

Rendering

Affiche le monde.

Contient :

WGPU
Pipelines
Shaders
Buffers GPU
Textures
Atlas
Culling rendu
Ombres
Fog
Post-process
Debug drawing

Rendering ne doit pas décider des règles du monde.

Input

Convertit les périphériques en intentions.

Exemples :

Avancer
Sauter
Miner
Placer
Ouvrir inventaire
Changer de slot
Debug toggle

Input ne décide pas si l'action est valide.

Physics

Gère :

Mouvement
Gravité planétaire
Collisions
Step-up
Glissement
Corps joueur

Physics ne gère pas l'inventaire ni les recettes.

Content schema

Définit les formats raw des données.

Contient :

RawBlockDef
RawItemDef
RawRecipeDef
RawBiomeDef
RawPlanetDef
RawTextureRef
RawTagDef

Ne charge pas les fichiers. Ne compile pas le contenu.

Pack loading

Gère :

Découverte des packs
Lecture des fichiers
Parsing
Ordre de chargement
Manifest

Ne crée pas les runtime ids.

Content compilation

Gère :

Validation profonde
Résolution des références
Normalisation
Tags
Overrides
Diagnostics
Création du contenu compilé

Ne fait pas le rendu. Ne fait pas le gameplay.

Runtime registries

Gère :

Runtime ids compacts
Tables rapides
Lookups
Indexes
Accès au contenu compilé

C'est la vérité runtime du contenu.

World generation

Génère le monde à partir de :

Seed
Paramètres planète
Biomes compilés
Blocs compilés
Registries

Worldgen ne doit pas hardcoder les listes de blocs.

Gameplay

Gère les règles :

Minage
Placement
Inventaire
Hotbar
Craft
Outils
Drops
Progression
Survie légère

Gameplay consomme les registries. Il ne redéfinit pas le contenu.

UI

Gère :

HUD
Console dev
Debug panels
Inventaire plus tard
Menus plus tard

UI ne doit pas contenir les règles du jeu.

Diagnostics

Gère :

FPS
Chunks visibles
Chunks chargés
Mesh count
VRAM estimée
Temps CPU/GPU
LOD actif
Culling
Logs utiles
Erreurs de contenu

Diagnostics doit aider les agents à comprendre l'état du moteur.

11. Règle des fichiers de moins de 1000 lignes

Aucun fichier source ne doit dépasser 1000 lignes.

Si un fichier approche 800 lignes, l'agent doit envisager une séparation. Si un fichier dépasse 1000 lignes, l'agent doit le refactorer avant d'ajouter plus de complexité.

Un fichier doit porter un concept clair.

Interdits sauf justification très forte :

common.rs
utils.rs
helpers.rs
misc.rs
manager.rs
system.rs trop vague
Fichiers géants avec plusieurs responsabilités

Préférer :

chunk_key.rs
voxel_pos.rs
planet.rs
chunk_map.rs
block_registry.rs
texture_atlas.rs
greedy_mesher.rs
frustum.rs
player_controller.rs
content_compiler.rs
12. Règle anti-duplication

Un concept doit avoir un seul propriétaire.

Interdit :

Deux types différents pour la même coordonnée
Deux systèmes de chunk concurrents
Deux registres de blocs
Deux loaders de contenu
Deux formats d'identifiants
Deux logiques de meshing pour la même chose
Deux définitions de block visual
Deux pipelines de textures

Si un agent trouve une duplication, il doit :

Identifier le propriétaire canonique
Supprimer ou fusionner l'autre version
Adapter proprement les appels
Éviter les wrappers temporaires inutiles
13. Performance comme règle de design

La performance est une fonctionnalité centrale.

Le moteur doit viser :

Grande distance de rendu
Beaucoup de chunks visibles
Peu d'allocations par frame
Streaming fluide
LOD stable
Framerate élevé
Temps de chargement raisonnable
Mesh generation asynchrone
Upload GPU maîtrisé
Culling efficace

L'agent doit éviter :

Recalculs massifs par frame
Clones inutiles de grosses données
HashMap partout dans les chemins chauds
Spawn illimité de threads
Upload GPU non borné
Allocations par voxel dans les boucles critiques
Rebuild global après une petite modification

Optimiser ne veut pas dire salir l'architecture. Une optimisation qui casse les responsabilités est refusée.

14. Distance de rendu immense

Un objectif majeur de VoxelVerse est de voir très loin.

Le joueur doit pouvoir percevoir :

La courbure de la planète
Les montagnes lointaines
Les biomes éloignés
Les grandes silhouettes
Les constructions importantes
L'horizon complet d'un petit monde

Pour cela, le moteur doit utiliser :

Streaming intelligent
Quadtree ou structure équivalente
LOD terrain
LOD voxel proche
Meshs simplifiés au loin
Culling robuste
Priorité basée sur caméra et joueur
Budget d'upload par frame
Transitions propres

La distance de rendu ne doit pas être obtenue en affichant naïvement tous les chunks voxel détaillés.

15. Règles de rendu voxel

Le style actuel visé est :

Blocs cubiques
Textures 2D classiques
Arêtes nettes
Atlas de textures
Matériaux simples mais propres
Lumière stylisée
Ombres lisibles
Ambiance atmosphérique

Ne pas partir sur :

Blocs procéduraux complexes par défaut
Meshs organiques partout
Micro-geometry inutile
Systèmes visuels trop compliqués avant d'avoir une base stable

La priorité est d'abord :

Afficher beaucoup de terrain proprement
Avoir des textures nettes
Avoir une lumière belle
Avoir un LOD efficace
Avoir une pipeline data-driven
16. Règles de textures

Les blocs doivent utiliser un système de textures data-driven.

Objectifs :

Atlas de textures
Références logiques aux textures
Variantes possibles
Faces différentes possibles
Textures par face possibles
Support futur des animations de texture
Support futur des normal maps ou effets simples si nécessaire

Un bloc doit pouvoir dire dans les données :

Texture top
Texture bottom
Texture side
Texture all
Tint éventuel
Règles visuelles éventuelles

Le moteur ne doit pas coder en dur les textures des blocs.

17. Règles de génération de monde

Le monde doit être beau et lisible.

La génération doit produire :

Reliefs intéressants
Grandes formes visibles
Biomes différenciés
Transitions propres
Landmarks naturels
Zones plates constructibles
Montagnes lisibles
Grottes plus tard
Ressources plus tard

Le bruit seul ne suffit pas. La génération doit être composée comme une carte jouable.

La génération doit être déterministe. Même seed + mêmes données = même monde.

18. Règles de planète ronde

La planète ronde est au cœur du jeu.

Le moteur doit préserver :

Gravité vers le centre
Coordonnées stables
Chunks adaptés à la sphère
Pas de trous entre faces
Continuité visuelle
Raycast fiable
Collisions fiables
LOD adapté à la courbure

La planète ne doit pas être un gimmick visuel. Elle doit structurer l'exploration.

19. Règles de construction

La construction doit être plus agréable que dans un Minecraft classique lorsque le jeu sera avancé.

Objectifs futurs :

Placement simple bloc par bloc
Ligne de blocs
Remplissage de surface
Outils mur
Outils sol
Outils escalier
Smart snapping
Prévisualisation claire
Annulation plus tard
Outils créatifs plus tard

Même si le moteur utilise des voxels précis, le joueur ne doit pas subir une micro-construction lente.

20. Survie légère

La survie doit créer du rythme, pas de la paperasse.

À éviter :

Trop de jauges
Trop de maintenance
Trop de menus
Trop d'obligations répétitives
Trop de punition passive

À viser :

Santé
Faim simple si utile
Danger lisible
Nuit ou obscurité importante
Exploration motivée
Progression par outils et confort
21. Modding

VoxelVerse doit être pensé pour les mods dès le départ.

Un moddeur doit pouvoir ajouter plus tard :

Un bloc
Un item
Une recette
Un biome
Une structure
Une ressource
Un outil
Un pack de textures

Sans modifier le moteur.

Les erreurs de contenu doivent être utiles. Un fichier cassé doit produire un diagnostic clair.

22. Diagnostics obligatoires

Le moteur doit toujours aider à comprendre son état.

Les agents doivent maintenir ou améliorer les diagnostics suivants :

FPS
Frame time
Chunks chargés
Chunks visibles
Chunks en attente
LOD visibles
Mesh vertices
Mesh indices
Mémoire estimée
Temps de génération mesh
Temps de génération monde
Upload GPU par frame
Distance de rendu effective
Position joueur
Coordonnée voxel ciblée

Les diagnostics ne doivent pas être mélangés au gameplay.

23. Qualité de code Rust

Le code Rust doit rester :

Clair
Modulaire
Idiomatique
Testable
Peu couplé
Peu public
Facile à lire par un agent IA

Règles :

Préférer pub(crate) à pub
Éviter les fichiers géants
Éviter les abstractions inutiles
Éviter les macros opaques
Éviter les noms vagues
Garder les responsabilités séparées
Ajouter des tests sur les systèmes critiques
Ne pas paniquer pour des erreurs de contenu normales
24. Nommage

Les noms doivent expliquer le rôle.

Préférer :

BlockRuntimeId
ChunkKey
VoxelPos
PlanetChunkMap
BlockRegistry
TextureAtlas
PackLoader
ContentCompiler
CompiledBlock
RawBlockDef
GreedyMesher
RenderChunk
WorldGenerator

Éviter :

Manager
System
Data
Helper
Stuff
Common
Util

Sauf si le mot est réellement précis dans le contexte.

25. Règles pour les agents IA

Avant toute modification, un agent doit lire le repo et répondre mentalement à ces questions :

Quel système possède cette responsabilité ?
Est-ce du runtime, du contenu, du rendu, du gameplay ou de la génération ?
Existe-t-il déjà un concept canonique ?
Est-ce que je crée une duplication ?
Est-ce que ce contenu devrait être data-driven ?
Est-ce que je respecte la limite de 1000 lignes par fichier ?
Est-ce que je supprime le legacy au lieu de l'envelopper ?
Est-ce que le projet compile après changement ?
Est-ce que l'agent suivant comprendra facilement où continuer ?
26. Comportement attendu d'un agent

Quand un agent reçoit une tâche, il doit :

Lire les fichiers concernés
Identifier les responsabilités
Refactorer proprement si nécessaire
Supprimer la duplication
Implémenter la demande
Garder les fichiers sous 1000 lignes
Vérifier la compilation
Signaler clairement ce qui a changé
Signaler les prochaines étapes évidentes

L'agent ne doit pas :

Demander à préserver l'ancien système
Ajouter une compatibilité legacy
Ajouter un hack local
Créer un nouveau concept doublon
Mettre du contenu hardcodé dans le moteur
Cacher une erreur avec un fallback silencieux
Laisser un fichier géant empirer
27. Définition de terminé

Une tâche est terminée seulement si :

Le projet compile
Le code est lisible
Les responsabilités sont respectées
Aucune duplication évidente n'est ajoutée
Aucun fichier ne dépasse 1000 lignes
Le contenu reste data-driven quand nécessaire
Le legacy inutile est supprimé
Les diagnostics restent utiles
Le moteur est plus proche de la vision finale

Compiler ne suffit pas. Le moteur doit devenir plus sain.

28. État actuel attendu d'un agent

Un agent qui arrive sur le repo doit comprendre que le projet est en fondation.

Il doit d'abord chercher :

Où est le runtime voxel ?
Où est le runtime monde ?
Où est le renderer ?
Où est le meshing ?
Où est la génération ?
Où est le contenu data-driven ?
Où sont les diagnostics ?
Quels fichiers dépassent ou approchent 1000 lignes ?
Quels concepts sont dupliqués ?
Quels systèmes sont encore hardcodés ?
Qu'est-ce qui bloque la distance de rendu immense ?

Il doit continuer le projet à partir de cet état réel, pas inventer une architecture parallèle.

29. Ordre de priorité recommandé

Les agents doivent généralement avancer dans cet ordre :

Compilation propre
Architecture claire
Séparation des responsabilités
Runtime voxel compact
Runtime planète robuste
Chunks et streaming
Distance de rendu
LOD
Meshing performant
Renderer propre
Textures et atlas
Pipeline data-driven
Runtime registries
Gameplay de base
Construction améliorée
Génération de monde avancée
Polish visuel
Modding complet

Ne pas commencer par des systèmes de gameplay avancés si le moteur n'est pas stable. Ne pas ajouter beaucoup de contenu si la pipeline data-driven n'est pas prête.

30. Interdictions permanentes

Interdit :

Legacy inutile
Compatibilité avec un vieux système non sorti
Duplication de concept
Fichiers de plus de 1000 lignes
Hardcoding de contenu
Renderer qui décide du gameplay
Input qui contient les règles du jeu
Worldgen qui possède le meshing
Runtime qui lit les fichiers raw
Monde qui stocke des strings de contenu
Fallback silencieux sur erreur de contenu
Optimisation qui détruit l'architecture
Gros fichier common.rs qui mélange tout
Patch rapide sans propriétaire clair
31. Résultat final voulu

À la fin, VoxelVerse doit être :

Un jeu voxel sandbox clair et beau
Inspiré de Minecraft, mais sur planètes rondes
Avec textures 2D stylisées
Avec très grande distance de rendu
Avec un moteur fluide
Avec un système de chunks robuste
Avec un LOD discret
Avec une génération de planète mémorable
Avec un rendu lumineux et atmosphérique
Avec une construction agréable
Avec un contenu 100% data-driven
Avec une architecture modulaire
Avec des fichiers courts et lisibles
Avec zéro legacy inutile
Avec zéro duplication conceptuelle
Avec une base solide pour les mods

Le joueur doit pouvoir marcher sur une petite planète ronde, voir très loin, reconnaître les biomes à l'horizon, miner, construire, explorer et sentir que le monde est complet, vivant et transformable.

Le moteur doit être assez propre pour que plusieurs agents IA puissent continuer à l'améliorer sans se marcher dessus.

32. Principe final

VoxelVerse doit être construit comme une planète :

Rond dans sa vision
Solide dans ses fondations
Lisible à distance
Riche quand on s'approche
Stable quand on creuse
Beau quand on l'observe

Chaque commit doit rapprocher le projet de cette sensation.

Si une modification rend le moteur plus confus, plus dupliqué, plus hardcodé, plus lent, plus fragile ou plus difficile à continuer pour le prochain agent, elle va contre VoxelVerse.

Construis peu de systèmes, mais construis-les parfaitement.
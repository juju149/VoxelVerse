# VoxelVerse — Météo, Ambiance et Cosmos

Document de référence pour la couche **Beauté du monde** : météo dynamique,
ambiances par biome, cycle céleste complet (étoiles, soleil-planète, lunes,
système solaire, galaxie, aurores boréales, entrée dans l'atmosphère).

Ce document décrit **l'architecture cible**, le **modèle de données**, le
**pipeline de rendu**, et un **plan d'implémentation par phases** que des agents
ou un humain peuvent suivre étape par étape sans casser l'archi existante.

---

## 1. Pilliers de design

1. **Lisibilité avant réalisme.** Le voxel doit rester lisible : la météo ne
   doit jamais noyer le jeu (brouillard total infranchissable, pluie qui mange
   le framerate). Tout effet a un *gameplay impact* contrôlé et un *visual
   impact* contrôlé.
2. **Data-driven.** Aucune météo, aucune ambiance, aucune palette céleste n'est
   codée en dur dans le moteur. Tout vit dans `vv-content-schema` (RON) et est
   chargé par `vv-pack-loader`. Le moteur n'est qu'un *évaluateur*.
3. **Découplage temps / état / rendu.** L'horloge planétaire et le simulateur
   météo produisent un **snapshot immuable** par frame ; le renderer consomme
   ce snapshot sans connaître les règles. Comme `WorldTime` / `EvaluatedAtmosphere`
   aujourd'hui, mais étendu.
4. **Échelle logarithmique.** Pas de coordonnées flottantes pour les distances
   cosmiques. On stratifie : *local* (voxels, m), *planétaire* (km, f32 OK),
   *orbital* (Mkm, f64), *stellaire* (al, f64 + unité), *galactique* (purement
   visuel, pas simulé).
5. **Budgets stricts.** Toute couche météo/cosmos respecte le contrat
   `05_RENDERING_AND_PERFORMANCE.md` : sky+weather+celestial < 2.5 ms à 60 fps
   sur profil High. Pas de couche optionnelle qui pète le budget.
6. **Zéro état caché.** Les transitions (météo qui change, aurore qui apparaît,
   transition atmosphère→espace) sont des **interpolations explicites** entre
   deux états sérialisables — pas des `lerp` dispersés.

---

## 2. Architecture par couches

```text
+---------------------------------------------------------------+
| L5  Galaxie / fond stellaire / cosmologie       (skybox lointaine) |
| L4  Système solaire / corps célestes voxel       (sun, moons)      |
| L3  Atmosphère planétaire (sky, scattering, sun-disc, stars)       |
| L2  Météo dynamique (clouds, rain, snow, storms, wind)             |
| L1  Ambiance biome (palette, fog tint, particle bias, post-fx)     |
| L0  Voxels (terrain, water, foliage, GI)                           |
+---------------------------------------------------------------+
```

Chaque couche est :

- **Lecture seule** sur celle du dessous (jamais d'aller-retour),
- **Écrit par** un système ECS ou un solver dédié,
- **Sérialisée** dans un snapshot consommé par le renderer.

### 2.1 Crates impactées / nouvelles

| Crate                           | Rôle dans la nouvelle archi                                 | Statut |
|----------------------------------|-------------------------------------------------------------|--------|
| `vv-content-schema`              | + schémas `WeatherProfile`, `BiomeAmbience`, `CelestialBody`, `StarCatalog`, `AuroraConfig` | étendre |
| `vv-pack-loader`                 | Charger les nouveaux schémas + valider                       | étendre |
| `vv-pack-doctor`                 | Lints : pas de palette à 0, pas de cloud_speed > X, etc.     | étendre |
| `vv-world`                       | + `WeatherState`, `CelestialState`, intégration `WorldTime`  | étendre |
| **`vv-weather`** *(nouveau)*    | Solver météo (transitions, conditions, vent, pluie, foudre)  | créer  |
| **`vv-celestial`** *(nouveau)*  | Mécanique orbitale (sun, lunes, étoiles, galaxie)            | créer  |
| `vv-render` → `atmosphere.rs`    | Étendu : devient `atmosphere/` (mod multi-fichiers)          | refactor |
| `vv-render` → `renderer/sky_renderer.rs` | Réécrit en passes (sky, stars, sun, moons, aurora, weather, fog) | refactor |
| `vv-render` → `renderer/cloud_renderer.rs` | Étendu pour pluie, neige, foudre, tempête          | étendre |
| `vv-render` → nouveaux         | `precipitation_renderer.rs`, `aurora_renderer.rs`, `star_field_renderer.rs`, `celestial_body_renderer.rs`, `space_transition.rs` | créer |
| `vv-audio`                       | Ambiances liées à `WeatherState` et `BiomeAmbience`          | étendre |

### 2.2 Découpage `vv-render/src/atmosphere/` (refactor)

L'unique `atmosphere.rs` actuel grossit ; on le découpe :

```text
vv-render/src/atmosphere/
  mod.rs              // ré-export public (compat)
  config.rs           // AtmosphereConfig, FogConfig, CloudConfig, ...
  presets.rs          // PlanetAtmospherePreset → AtmosphereConfig
  evaluate.rs         // EvaluatedAtmosphere (sortie par frame)
  weather_blend.rs    // mélange météo + biome → atmosphère effective
  celestial_blend.rs  // overlay céleste (sun dir, moons, stars)
```

Le module reste **stateless** : il consomme `WorldTime + WeatherState + CelestialState + BiomeAmbience` et produit un `EvaluatedAtmosphere` immutable.

---

## 3. Modèle de données (data-driven)

### 3.1 `WeatherProfile` — schéma RON

Décrit *un type de condition météo* (pas un état instantané).

```ron
WeatherProfile(
    id: "thunderstorm",
    rarity: 0.05,                  // [0..1] base spawn weight
    biome_bias: { "savanna": 1.5, "tundra": 0.0 },
    cloud_coverage: 0.95,
    cloud_density_mul: 1.4,
    cloud_speed_mul: 2.6,
    cloud_tint: (0.18, 0.18, 0.22),
    fog_multiplier: 1.20,
    fog_tint: (0.32, 0.32, 0.36),
    precipitation: Some((
        kind: Rain,
        intensity: 0.85,           // 0..1
        wind_drift: 0.6,
        splash_density: 0.7,
        sound: "weather/rain_heavy",
    )),
    wind: (
        base_speed: 12.0,          // m/s
        gust_speed: 22.0,
        gust_interval_s: 4.5,
        direction_drift_per_s: 0.05,
    ),
    lightning: Some((
        strikes_per_minute: 2.0,
        flash_intensity: 4.0,
        thunder_delay_per_km: 3.0,
        sound: "weather/thunder",
    )),
    post_fx: (
        exposure_mul: 0.78,
        saturation_mul: 0.85,
        contrast_add: 0.05,
    ),
    transitions: (
        fade_in_s: 8.0,
        fade_out_s: 12.0,
        min_duration_s: 60.0,
        max_duration_s: 240.0,
    ),
)
```

Profils V1 minimum : `clear`, `overcast`, `light_rain`, `thunderstorm`,
`light_snow`, `blizzard`, `fog_dense`, `sandstorm`, `aurora_calm`, `toxic_haze`.

### 3.2 `BiomeAmbience`

Couche L1 — *ce que le biome fait au sky/fog/post-fx* en plus du profil météo.

```ron
BiomeAmbience(
    id: "polar_ice",
    fog_tint_mul: (0.92, 0.96, 1.05),
    sky_horizon_tint: (0.86, 0.94, 1.00),
    ambient_dust_density: 0.0,
    ambient_particles: Some(("snow_drift", 0.20)),
    post_fx: ( saturation_mul: 0.78, exposure_mul: 0.94, contrast_add: 0.02 ),
    allowed_weather: ["clear", "light_snow", "blizzard", "fog_dense", "aurora_calm"],
    weather_weights: { "blizzard": 1.6, "aurora_calm": 1.2 },
    aurora: Some((
        latitude_threshold: 0.78,  // fraction de la coord polaire
        color_a: (0.10, 0.92, 0.55),
        color_b: (0.38, 0.42, 0.95),
        intensity_curve: "polar_smooth",
    )),
)
```

### 3.3 `CelestialBody`

Décrit un soleil, une lune, ou une planète visible depuis le sol.

```ron
CelestialBody(
    id: "sol_primary",
    kind: Star,                    // Star | Moon | Planet | Belt | Ring
    voxel_model: Some("celestial/sol.vox"),   // optionnel : soleil voxel
    radius_m: 6.96e8,
    orbit: Some((
        parent: None,               // None = barycentre du système
        semi_major_axis_m: 0.0,
        eccentricity: 0.0,
        period_s: 0.0,
    )),
    spin: (axis: (0.0, 1.0, 0.0), period_s: 2.16e6),
    surface: (
        emissive_color: (1.00, 0.92, 0.78),
        emissive_intensity: 8.0,
        corona: Some(( inner: (1.0, 0.9, 0.6), outer: (1.0, 0.5, 0.2), radius_mul: 4.0 )),
    ),
    visible_from_surface: true,
    lod_billboard_distance_m: 1.0e8,   // au-delà, on rend en sprite HDR
)
```

Pour le **soleil voxel** (le user a explicitement demandé) :

- modèle `.vox` chargé depuis le pack ;
- rendu en **mesh impostor** depuis la surface (sprite + bumpmap) tant qu'on
  est dans l'atmosphère, en **vrai mesh voxel** dès qu'on dépasse une
  altitude `space_transition_altitude_m` (cf. §5.4) ;
- LOD céleste : mesh complet < 100 Mm, mesh décimé sinon, billboard au-delà.

### 3.4 `StarCatalog`

```ron
StarCatalog(
    id: "milky_way_local",
    seed: 0xC051_7AE5,
    star_count: 8000,              // V1 cap
    magnitude_range: (-1.5, 6.0),
    spectral_distribution: ("O", 0.0001, "B", 0.001, "A", 0.01, "F", 0.03,
                            "G", 0.08, "K", 0.13, "M", 0.76),
    milky_way: Some((
        density_texture: "celestial/milky_way_density.ktx2",
        tint: (0.85, 0.88, 1.00),
        intensity: 0.6,
    )),
    nebulae: [
        ( name: "Orion", center_lonlat: (1.42, -0.10), radius_rad: 0.08,
          color: (0.55, 0.45, 0.95), intensity: 0.3 ),
    ],
)
```

### 3.5 Snapshots runtime

Sortie par frame, consommée par le renderer :

```rust
pub struct WeatherState {
    pub current: WeatherProfileId,
    pub next: Option<WeatherProfileId>,
    pub blend: f32,                  // 0..1 transition
    pub wind: WindVector,            // dir + speed (m/s)
    pub precipitation: PrecipitationSample,
    pub lightning_events: SmallVec<[LightningStrike; 4]>,
}

pub struct CelestialState {
    pub sun_dir_world: Vec3,
    pub sun_disc_color: Vec3,
    pub sun_disc_angular_radius: f32,
    pub moons: SmallVec<[MoonSample; 4]>,
    pub stars_visibility: f32,       // 0..1
    pub aurora_intensity: f32,       // 0..1
    pub eclipse_factor: f32,         // 0..1, dims sun
    pub altitude_band: AltitudeBand, // Ground | Strato | Meso | Space
}
```

Ces deux structures **plus** `BiomeAmbience` du biome local **plus**
`AtmosphereConfig` de la planète sont mélangés en **un seul**
`EvaluatedAtmosphere` (ordre : preset → biome → weather → celestial → post-fx).

---

## 4. Solveurs

### 4.1 Solver météo (`vv-weather`)

Modèle **markovien biaisé par biome et heure** :

1. Chaque biome local (sous le joueur ou la caméra) maintient un *climate cell*
   `(temperature, humidity, pressure)` interpolé depuis la table planétaire.
2. À intervalle fixe (ex. 30 s), on tire la prochaine météo selon
   `weather_weights * biome_bias * climate_pressure`.
3. La transition `current → next` est interpolée sur `transitions.fade_in_s`.
4. **Vent** : champ scalaire global (jet stream simplifié, fonction de latitude
   + bruit Perlin lent), modulé par la météo locale. Stocké dans
   `WeatherState.wind`.
5. **Foudre** : Poisson process `strikes_per_minute`, position choisie autour
   du joueur dans un rayon `R` ; déclenche flash + son retardé.

**État** stocké côté `vv-world` : `WeatherSimState` (RNG + transition + horloge),
sérialisable dans le save.

### 4.2 Solver céleste (`vv-celestial`)

Mécanique orbitale képlérienne simplifiée (orbites circulaires V1, elliptiques
V1.1) :

```text
position(t) = parent.position + orbit.radius * (cos(θ), 0, sin(θ))
où θ = 2π * (t / orbit.period_s + phase_offset)
```

- En `f64` pour les distances.
- Le repère **monde** du joueur reste centré sur la planète locale : on calcule
  les positions célestes dans le repère **système**, puis on les projette en
  *direction unitaire* depuis l'observateur (pour la skybox) ou en *vecteur
  réel* (en espace).
- Cycle jour/nuit dérive directement de la rotation de la planète autour de
  son axe (déjà dans `WorldTime`) et de la direction du soleil dans le repère
  système.
- **Étoiles fixes** dans le repère galactique ; on applique la rotation
  combinée (axial spin + révolution) pour les rendre dans le ciel.

### 4.3 Aurore

Pas une météo, une **modulation** du sky :

```text
aurora_intensity = aurora.allowed(lat)
                 * smooth(world_time.night_factor)
                 * geomagnetic_storm_factor(t)      // bruit lent
                 * (1 - cloud_coverage * 0.6)
```

Rendue dans le sky pass en post-clouds, pre-stars.

---

## 5. Pipeline de rendu

Ordre dans `render_passes.rs` (étendu) :

```text
1. Shadow pass
2. Terrain G-buffer
3. Water pass
4. Decals / damage overlays
5. Sky pass               ──── BLOC NOUVEAU ───────────
   5a. star_field            (raymarched ou sphere skybox)
   5b. galaxy / milky way    (texture cubemap pré-cuit)
   5c. celestial_bodies      (sun voxel/impostor, moons, planètes)
   5d. atmosphere_scattering (Bruneton-style simplifié)
   5e. aurora_band           (additive volumetric raymarch)
   5f. clouds                (existant, étendu)
6. Precipitation pass     ──── NOUVEAU ──────────────
   6a. rain streaks (GPU instanced)
   6b. snow particles
   6c. splash decals (rain on water/ground)
7. Fog volumetric (existant)
8. Lightning flash pass   ──── NOUVEAU ──────────────
9. Post-process (tone, exposure, contrast, bloom)
10. UI
```

### 5.1 Sky pass (réécriture)

Shader unique multi-passe basé sur **raymarch atmosphère analytique** :

- Modèle Bruneton/Hillaire simplifié (Rayleigh + Mie, single scattering).
- Look-up table 2D `transmittance(altitude, sun_angle)` pré-cuit par planète
  (256×128 f16). Rebuild seulement quand `AtmosphereConfig` change.
- Sortie HDR linéaire, tone-mappée plus tard.

### 5.2 Étoiles

Deux modes :

- **Field statique** : cubemap pré-cuit (off-line via outil dans
  `vv-pack-compiler`) à partir du `StarCatalog`. Rapide, joli, ne scintille pas
  trop.
- **Field dynamique** : 8k étoiles instancées avec scintillement basé sur
  `time + position`. Coût ~0.1 ms. Recommandé pour V1.

Magnitude → taille en pixels via courbe HDR, blend additif.

### 5.3 Corps célestes voxel (soleil, lunes)

- **Impostor billboard** par défaut depuis le sol : sprite HDR avec corona,
  shading direct par `EvaluatedAtmosphere`.
- **Vrai mesh voxel** activé quand :
  - le corps est plus proche que `lod_billboard_distance_m`, **ou**
  - le joueur est en `AltitudeBand::Space`.
- LOD voxel : `vv-meshing` génère 3 niveaux (high / medium / low) en
  pré-compilation pour chaque corps céleste.
- Eclipse : test simple sphère ↔ disque solaire pour atténuer `sun_disc_color`.

### 5.4 Transition atmosphère → espace

`AltitudeBand` est dérivée de l'altitude du joueur :

| Bande     | Altitude (planète terrestre)   | Effets                                           |
|-----------|--------------------------------|--------------------------------------------------|
| Ground    | 0..2 km                        | météo nominale                                   |
| Strato    | 2..20 km                       | nuages bas disparaissent, ciel plus sombre       |
| Meso      | 20..80 km                      | étoiles visibles le jour, sky desaturé           |
| Space     | > 80 km                        | sky noir, étoiles full, atmosphère = halo fin    |

Interpolation **smooth** entre bandes sur ~10 km pour éviter les pops.

Implémentation : un `space_transition.rs` calcule `space_factor ∈ [0,1]` et
le passe à tous les uniforms du sky pass.

### 5.5 Précipitations

GPU-instanced :

- **Pluie** : streaks orientés par `wind.dir`, rendus en quads instanced dans
  une coque cylindrique autour du joueur (rayon 60 m, 4096 streaks max).
- **Neige** : quads soft, vitesse de chute liée à `wind`, alpha animé.
- **Splash decals** : pour la pluie au sol, marker pass additif court (3 frames)
  sur les blocs solides + sur l'eau (ride sur water shader).

Pas de simulation physique par goutte — purement visuel.

### 5.6 Foudre

- Quand un `LightningStrike` est émis : on génère un **bolt mesh** (random
  walk segmenté, 8-16 segments), instancié 1 frame.
- Frame du strike : `flash_intensity` ajouté à l'ambient + bloom.
- Audio : event différé selon `thunder_delay_per_km * distance`.

### 5.7 Aurore

Raymarch dans une coque sphérique haute (80-200 km altitude planète), bruit
2D animé sur la coordonnée polaire, blend additif HDR. Coût cible : 0.3 ms à
half-res + upscale.

---

## 6. Performance — budgets

Profil **High**, 60 fps (16.67 ms total, ~10 ms render) :

| Passe                          | Budget   | Notes                                  |
|--------------------------------|----------|----------------------------------------|
| Sky atmosphere scattering      | 0.6 ms   | full-res, LUT mode                     |
| Stars + galaxy                 | 0.2 ms   | cubemap + 8k instances                 |
| Celestial bodies (impostor)    | 0.1 ms   | < 10 corps visibles                    |
| Celestial bodies (voxel mesh)  | 0.8 ms   | uniquement en `AltitudeBand::Space`    |
| Aurora                         | 0.3 ms   | half-res raymarch + upscale            |
| Clouds (vol.)                  | 1.5 ms   | déjà mesuré (existant)                 |
| Rain (heavy)                   | 0.4 ms   | 4096 instances                         |
| Snow                           | 0.3 ms   |                                        |
| Lightning bolt                 | 0.05 ms  | 1 frame                                |
| **Total météo+cosmos cap**     | **~3.5 ms** | doit tenir avec marge                |

Profil **Low** : aurora off, vol clouds → flat, rain density × 0.4, stars cubemap statique.

Tous ces budgets sont **mesurés** via `vv-diagnostics::render_stats` et
**affichés** dans le HUD debug (déjà en place).

---

## 7. Plan d'implémentation par phases

Chaque phase est **mergeable indépendamment**, livre une feature visible,
et passe `cargo test`, `cargo clippy`, et les scripts `check_*.ps1` existants.

### Phase 0 — Refactor préparatoire (1-2 jours)
- [ ] Découper `vv-render/src/atmosphere.rs` en module `atmosphere/` (cf. §2.2).
- [ ] Extraire `WeatherConfig`/`CloudConfig` de l'`AtmosphereConfig` actuel
      vers `weather_blend.rs` sans changer le comportement.
- [ ] Ajouter `EvaluatedAtmosphere::altitude_band` (toujours `Ground` pour
      l'instant).
- **DoD** : pas de régression visuelle, tests roundtrip RON OK.

### Phase 1 — Schémas data (2 jours)
- [ ] `vv-content-schema` : `WeatherProfile`, `BiomeAmbience`, `CelestialBody`,
      `StarCatalog`, `AuroraConfig`.
- [ ] `vv-pack-loader` : loaders + validation.
- [ ] `vv-pack-doctor` : lints (densité bornée, IDs uniques, références OK).
- [ ] Tests `ron_roundtrip` pour chacun.
- **DoD** : pack v1 charge 5 profils météo et le catalogue stellaire local.

### Phase 2 — Solver météo (`vv-weather`) (3-4 jours)
- [ ] Crate `vv-weather` : `WeatherSimState`, RNG seedé, transitions.
- [ ] Intégration `WorldTime` → tick.
- [ ] Champ de vent global + locale.
- [ ] Snapshot `WeatherState` → renderer.
- [ ] Debug overlay : météo courante, prochaine, blend, wind vector.
- **DoD** : météo change toutes ~2 min, visible dans le HUD debug ; pas de
  rendu nouveau encore (juste valeurs).

### Phase 3.A — Intégration data-driven (1 jour) ✅
- [x] `vv-render` dépend de `vv-weather`.
- [x] `RenderFrameSnapshot.weather: Option<&WeatherState>`.
- [x] `EvaluatedAtmosphere::apply_weather` : override `cloud_coverage`,
      multiplie `cloud_density` / `cloud_speed` / `fog_density`, additionne
      `flash_intensity` des strikes (capé à 1.5).
- [x] Câblage dans `render_passes.rs` après `evaluate(...)`.
- **DoD** : un `WeatherState` "orage" assombrit visiblement le ciel et
  épaissit le brouillard sans nouveau pipeline, et un strike fait un flash.

### Phase 3.B — Pipelines pluie / neige / foudre (partiel ✅, polish à venir)
- [x] WGSL `precipitation.frag.wgsl` : streaks rain + flakes snow procéduraux
      en screen-space, lus depuis `weather_params` du `GlobalUniform`.
- [x] `precipitation_renderer.rs` : pass full-screen entre fog et post.
      Self-skip dans le shader quand `intensity == 0` → coût ~nul.
- [x] `GlobalUniform.weather_params` (vec4, +16 octets → 272 total) : intensité,
      direction du vent xz, kind (0..6).
- [x] Tilt des streaks proportionnel à `wind.x` ; drift des flocons modulé
      par `|wind.x|`.
- [x] Flash ambient câblé en 3.A via `apply_weather` (boost `sun_intensity`).
- [ ] **Polish 3.B-2** (suivi) : splash decals (water shader + additive sol),
      bolt mesh (random walk 8-16 segments) à la place du flash uniforme,
      streaks instancés pour densité élevée si nécessaire.
- **DoD partiel** : pluie/neige visibles à `intensity > 0`, vent qui drague
  les streaks, kind switch automatique entre rain/snow.

### Phase 3.C — Audio météo (mixer ✅, sinks rodio à câbler)
- [x] `vv-audio::WeatherAudioMix` : volumes cibles dérivés du `WeatherState`
      (pluie via `precipitation.intensity`, vent via `wind.speed`, thunder
      ambient sur frame de strike).
- [x] `WeatherThunderEvent { delay_s, volume }` : claps retardés depuis
      `LightningStrike.thunder_delay_s`, atténuation quadratique avec distance.
- [x] Tests déterministes du mixage (sans device audio).
- [ ] **Polish 3.C-2** (suivi) : câbler les `rodio::Sink` sustained pour les
      loops pluie/vent une fois les .ogg ajoutés au pack, scheduling des
      one-shots de tonnerre via timer interne.
- **DoD partiel** : `WeatherAudioMix::from_state` est correct et testé ;
  le host n'a plus qu'à brancher les sinks quand les assets arrivent.

### Phase 4 — Solver céleste (`vv-celestial`) (3 jours)
- [ ] Crate `vv-celestial` : orbites circulaires, `f64`, positions soleil/lunes.
- [ ] Couplage `WorldTime` + repère système.
- [ ] Snapshot `CelestialState`.
- [ ] Eclipse simple soleil/lune.
- **DoD** : positions céleste correctes, debug overlay angles.

### Phase 5 — Rendu sky complet (5-7 jours)
- [ ] Réécriture `sky_renderer.rs` avec atmosphere scattering (LUT pré-cuit).
- [ ] `star_field_renderer.rs` (mode dynamique 8k).
- [ ] `celestial_body_renderer.rs` (impostor sun/moons + corona).
- [ ] `aurora_renderer.rs` (raymarch half-res).
- [ ] Tone mapping ajusté pour HDR sky + bloom corona.
- **DoD** : ciel jour/nuit/dawn/dusk + lunes + étoiles + aurore aux pôles.

### Phase 6 — Ambiance par biome (2 jours)
- [ ] Application `BiomeAmbience` au blend final.
- [ ] Particules ambient (poussière désert, plumes neige, pollen, spores).
- [ ] Post-fx biome (saturation/contraste).
- **DoD** : passer d'un biome à l'autre se sent immédiatement.

### Phase 7 — Espace et corps voxel (5-7 jours)
- [ ] `AltitudeBand` + `space_transition.rs`.
- [ ] Soleil-voxel : modèle .vox, mesh LOD, rendu en `Space`.
- [ ] Lunes voxel.
- [ ] Skybox galaxie (Voie Lactée) pré-cuit.
- [ ] Fond de système solaire : planètes lointaines en billboard.
- **DoD** : on peut monter en altitude → voir l'atmosphère devenir un halo,
  étoiles plein le ciel, et le soleil-voxel à pleine résolution.

### Phase 8 — Polish (continu)
- Eclair fork mesh procédural ;
- Arc-en-ciel post-pluie (1 ligne dans le sky pass) ;
- Halos lunaires / parhélies ;
- Vapeur de respiration du joueur en biome froid ;
- Effet "rentrée atmosphérique" (heat shield trail) si vélocité > seuil.

---

## 8. Tests et garde-fous

- **Tests unitaires** : solver météo (probabilités, transitions monotones),
  solver céleste (positions à T=0, périodes, eclipse).
- **Roundtrip RON** : tous les nouveaux schémas (déjà la convention du repo).
- **Pack doctor** : lints sur palettes vides, densités hors bornes, orbites
  négatives, refs cassées.
- **Smoke render** : un test integration `vv-render` qui rend 1 frame par
  météo × par biome (snapshot perceptuel : `Δ < ε` vs baseline).
- **Budgets** : `render_stats` doit afficher chaque passe ; le test perf
  échoue si une passe dépasse son budget de plus de 20 %.
- **Determinisme** : même seed + même `WorldTime` → mêmes `WeatherState` et
  `CelestialState`.

---

## 9. Ordre de démarrage recommandé

Pour commencer **dès maintenant** sans tout casser :

1. **Phase 0** (refactor `atmosphere.rs`) — *purement mécanique, zéro risque*.
2. **Phase 1** (schémas RON) — *bloque le reste, à faire en priorité*.
3. **Phase 2** (solver météo) en parallèle avec **Phase 4** (solver céleste) si
   on veut deux pistes parallèles (ils sont indépendants).
4. **Phase 3** (rendu météo) — premier effet visible spectaculaire.
5. **Phase 5** (sky complet) — gros gain de beauté.
6. **Phase 6** (biome ambiance) — finition.
7. **Phase 7** (espace) — feature *wow*.

Chaque phase = une PR. Chaque PR doit : compiler, passer clippy, passer les
scripts `check_*`, rendre la même image qu'avant si la feature n'est pas
activée par défaut, et ajouter sa propre ligne dans le HUD debug.

---

## 10. Références à consulter avant code

- `docs/v1/04_WORLD_AND_PLANETS.md` — contrat planète/biome ;
- `docs/v1/05_RENDERING_AND_PERFORMANCE.md` — budgets et streaming ;
- `crates/vv-render/src/atmosphere.rs` — état actuel à refactorer ;
- `crates/vv-render/src/renderer/sky_renderer.rs` — point d'extension principal ;
- `crates/vv-world/src/world_time.rs` — horloge planétaire existante ;
- *Production Sky Rendering* (Hillaire 2020) — modèle scattering analytique recommandé ;
- *Real-Time Rendering* ch. 14 — fondements atmosphère/clouds.

---

**TL;DR pour commencer demain :**
créer la branche `feat/weather-cosmos-phase0`, découper `atmosphere.rs` en
module, ajouter les types vides `WeatherState` / `CelestialState` /
`BiomeAmbience` dans `vv-world`, et brancher trois lignes dans le HUD debug.
Tout le reste est déjà préparé par ce document.

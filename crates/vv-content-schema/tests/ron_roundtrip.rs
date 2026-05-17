//! RON round-trip parsing tests for every top-level schema type.
//!
//! These tests are pure schema-level — they don't touch the actual content
//! pack on disk. Each test renders a small, hand-written RON sample and
//! confirms the schema can ingest it without error. When you add a new
//! top-level field, add a sample here so the doctor isn't the only line of
//! defense.

use vv_content_schema::*;

fn parse<T: serde::de::DeserializeOwned>(name: &str, src: &str) -> T {
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str::<T>(src)
        .unwrap_or_else(|e| panic!("{name} sample failed to parse: {e}"))
}

fn parse_err<T: serde::de::DeserializeOwned>(src: &str) -> String {
    match ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str::<T>(src)
    {
        Ok(_) => panic!("sample should fail to parse"),
        Err(err) => err.to_string(),
    }
}

// ── pack.ron ─────────────────────────────────────────────────────────────────

#[test]
fn pack_manifest_parses() {
    let src = r#"PackManifest(
        format_version: 1,
        namespace: "core",
        display_name: "VoxelVerse Core",
        version: "0.1.0",
        kind: builtin,
        description: "Built-in foundation content.",
        authors: ["VoxelVerse Team"],
        license: "project-internal",
        load_priority: 0,
        dependencies: [],
        features: ["objects", "world"],
        content_roots: (definitions: "defs", media: "media", generated: "generated"),
        rules: (identity: path_derived, id_style: "namespace:category/name",
                runtime_loads_raw_files: false),
    )"#;
    let m: RawPackManifest = parse("pack manifest", strip_wrapper(src));
    assert_eq!(m.namespace, "core");
    assert_eq!(m.features.len(), 2);
}

// ── object.ron — minimal block ───────────────────────────────────────────────

#[test]
fn object_block_minimum_parses() {
    let src = r##"Object(
        format_version: 1,
        name: "Dirt",
        tags: ["terrain", "soil"],
        block: (
            texture: all("blocks/dirt/all"),
            hardness: 0.5,
        ),
        item: (
            stack: 99,
            category: block,
            visible_in_inventory: true,
            inventory_icon: block,
        ),
    )"##;
    let o: RawObjectDef = parse("object block", strip_wrapper(src));
    assert_eq!(o.format_version, 1);
    assert!(o.block.is_some());
    assert!(o.item.is_some());
    assert!(o.recipes.is_empty());
}

#[test]
fn object_uses_default_format_version_when_omitted() {
    let src = r##"Object(
        name: "Dirt",
        block: (texture: none),
    )"##;
    let o: RawObjectDef = parse("object default version", strip_wrapper(src));
    assert_eq!(o.format_version, OBJECT_FORMAT_VERSION);
}

#[test]
fn object_rejects_removed_item_icon_field() {
    let src = r##"Object(
        name: "Removed Icon",
        item: (
            stack: 1,
            category: resource,
            visible_in_inventory: true,
            icon: "items/removed",
        ),
    )"##;
    let err = parse_err::<RawObjectDef>(strip_wrapper(src));
    assert!(err.contains("Unexpected field named `icon`"), "{err}");
}

#[test]
fn object_rejects_unknown_fields() {
    let src = r##"Object(
        name: "Rabbit",
        spawn: (density: 0.1),
    )"##;
    let err = parse_err::<RawObjectDef>(strip_wrapper(src));
    assert!(err.contains("Unexpected field named `spawn`"), "{err}");
}

#[test]
fn object_mining_requires_explicit_drops() {
    let src = r##"Object(
        name: "Stone",
        block: (texture: all("blocks/stone/all")),
        mining: (
            tool: pickaxe,
        ),
    )"##;
    let err = parse_err::<RawObjectDef>(strip_wrapper(src));
    assert!(err.contains("Unexpected missing field `drops`"), "{err}");
}

// ── object.ron — recipes Vec + RawObjectRecipeKind enum ──────────────────────

#[test]
fn object_with_two_recipes_parses() {
    let src = r##"Object(
        name: "Torch",
        item: (
            stack: 64,
            category: utility,
            visible_in_inventory: true,
            inventory_icon: texture("core:texture/items/utility/torch"),
        ),
        recipes: [
            (
                station: Some("#core:tag/station/construction"),
                kind: shaped((
                    pattern: ["C", "S"],
                legend: { "C": "core:object/resources/coal", "S": "core:object/resources/stick" },
                )),
                output: (item: "torch", count: 4),
            ),
            (
                station: None,
                kind: shapeless((ingredients: ["core:object/resources/resin", "core:object/resources/stick"])),
                output: (item: "torch", count: 2),
            ),
        ],
    )"##;
    let o: RawObjectDef = parse("two recipes", strip_wrapper(src));
    assert_eq!(o.recipes.len(), 2);
    match &o.recipes[0].kind {
        RawObjectRecipeKind::Shaped(s) => {
            assert_eq!(s.pattern.len(), 2);
            assert!(s.legend.contains_key("C"));
        }
        _ => panic!("expected shaped recipe"),
    }
    match &o.recipes[1].kind {
        RawObjectRecipeKind::Shapeless(_) => {}
        _ => panic!("expected shapeless recipe"),
    }
    assert!(o.recipes[1].station.is_none());
}

#[test]
fn object_processing_recipe_parses() {
    let src = r##"Object(
        name: "Iron Ingot",
        item: (
            stack: 64,
            category: resource,
            visible_in_inventory: true,
            inventory_icon: texture("core:texture/items/resources/iron_ingot"),
        ),
        recipes: [(
            station: Some("#core:tag/station/furnace"),
            kind: processing((
                inputs: [(item: "core:object/resources/iron_ore_chunk", count: 1, chance: 1.0)],
                duration_seconds: 8.0,
            )),
            output: (item: "iron_ingot", count: 1),
        )],
    )"##;
    let o: RawObjectDef = parse("processing recipe", strip_wrapper(src));
    match &o.recipes[0].kind {
        RawObjectRecipeKind::Processing(p) => {
            assert_eq!(p.inputs.len(), 1);
            assert!((p.duration_seconds - 8.0).abs() < 1e-6);
        }
        _ => panic!("expected processing recipe"),
    }
}

// ── object.ron — station with station_tags ───────────────────────────────────

#[test]
fn object_station_with_tags_parses() {
    let src = r##"Object(
        name: "Construction Workbench",
        block: (texture: all("blocks/workbench/all")),
        item: (
            stack: 1,
            category: station,
            visible_in_inventory: true,
            inventory_icon: block,
        ),
        station: (
            type: workbench,
            station_tags: ["#core:tag/station/construction", "#core:tag/station/tool"],
            slots: Some(9),
        ),
    )"##;
    let o: RawObjectDef = parse("station tags", strip_wrapper(src));
    let s = o.station.expect("station");
    assert_eq!(s.station_tags.len(), 2);
    assert!(matches!(s.station_type, RawObjectStationType::Workbench));
}

// ── object.ron — entity with skeleton ─────────────────────────────────────────

#[test]
fn object_entity_with_skeleton_parses() {
    let src = r#"Object(
        name: "Rabbit",
        entity: (
            model: "voxel/creatures/rabbit/male",
            skeleton: "skeleton/quadruped/small",
            ai: passive_wanderer,
            health: 6,
            move_speed: 1.8,
        ),
    )"#;
    let o: RawObjectDef = parse("entity skeleton", strip_wrapper(src));
    let e = o.entity.expect("entity");
    assert_eq!(e.skeleton.as_deref(), Some("skeleton/quadruped/small"));
}

// ── world.ron — biome (just enough to confirm the schema chains) ─────────────

#[test]
fn world_biome_parses() {
    let src = r#"Biome(
        display_name: "Plains",
        surface: (top: "core:object/terrain/grass", under: "core:object/terrain/dirt",
                  depth_voxels: (2, 5)),
        terrain: (
            base_height: -0.01, amplitude: 0.15, flatness: 0.72,
            hill_field: "core:field/soft_hills", terrace_strength: 0.0,
        ),
        palette: (grass: (0.5, 0.7, 0.3), foliage: (0.3, 0.6, 0.3), fog_bias: (0.0, 0.0, 0.0)),
        placement: (vegetation_tags: [], fauna_tags: [], structure_tags: []),
    )"#;
    let _: RawBiomeProceduralDef = parse("biome", strip_wrapper(src));
}

// ── skeleton.ron ─────────────────────────────────────────────────────────────

#[test]
fn skeleton_parses() {
    let src = r#"Skeleton(
        format_version: 1,
        display_name: "Quadruped Small",
        bones: [
            (name: "root", rest: (translation: (0.0, 0.0, 0.0),
                                  rotation: (0.0, 0.0, 0.0, 1.0))),
            (name: "spine", parent: Some("root"),
             rest: (translation: (0.0, 0.2, 0.0), rotation: (0.0, 0.0, 0.0, 1.0))),
        ],
        slots: [(name: "saddle", bone: "spine")],
    )"#;
    let s: RawSkeletonDef = parse("skeleton", strip_wrapper(src));
    assert_eq!(s.bones.len(), 2);
    assert_eq!(s.slots.len(), 1);
}

// ── weather.ron — clear + storm + minimal ────────────────────────────────────

#[test]
fn weather_clear_parses() {
    let src = r#"WeatherProfile(
        display_name: "Clear Sky",
        rarity: 0.6,
        cloud_coverage: 0.18,
        cloud_density_mul: 0.7,
        wind: ( base_speed: 3.0, gust_speed: 6.0, gust_interval_s: 12.0 ),
    )"#;
    let p: RawWeatherProfileDef = parse("weather clear", strip_wrapper(src));
    assert!((p.rarity - 0.6).abs() < 1e-5);
    assert!(p.precipitation.is_none());
    assert!(p.lightning.is_none());
    assert!((p.transitions.fade_in_s - 8.0).abs() < 1e-5);
}

#[test]
fn weather_thunderstorm_full_parses() {
    let src = r#"WeatherProfile(
        display_name: "Thunderstorm",
        rarity: 0.05,
        biome_bias: { "savanna": 1.5, "tundra": 0.0 },
        cloud_coverage: 0.95,
        cloud_density_mul: 1.4,
        cloud_speed_mul: 2.6,
        cloud_tint: (0.18, 0.18, 0.22),
        fog_multiplier: 1.2,
        fog_tint: (0.32, 0.32, 0.36),
        precipitation: (
            kind: rain,
            intensity: 0.85,
            wind_drift: 0.6,
            splash_density: 0.7,
        ),
        wind: (
            base_speed: 12.0, gust_speed: 22.0, gust_interval_s: 4.5,
            direction_drift_per_s: 0.05,
        ),
        lightning: (
            strikes_per_minute: 2.0,
            flash_intensity: 4.0,
            thunder_delay_per_km: 3.0,
        ),
        post_fx: ( exposure_mul: 0.78, saturation_mul: 0.85, contrast_add: 0.05 ),
        transitions: ( fade_in_s: 8.0, fade_out_s: 12.0,
                       min_duration_s: 60.0, max_duration_s: 240.0 ),
    )"#;
    let p: RawWeatherProfileDef = parse("weather storm", strip_wrapper(src));
    assert_eq!(
        p.precipitation.as_ref().unwrap().kind,
        RawPrecipitationKind::Rain
    );
    assert!(p.lightning.is_some());
    assert_eq!(p.biome_bias.len(), 2);
    assert!((p.cloud_speed_mul - 2.6).abs() < 1e-5);
}

// ── biome ambience ───────────────────────────────────────────────────────────

#[test]
fn biome_ambience_polar_parses() {
    let src = r#"BiomeAmbience(
        display_name: "Polar Ice",
        fog_tint_mul: (0.92, 0.96, 1.05),
        sky_horizon_tint: (0.86, 0.94, 1.0),
        ambient_particles: ( kind: "core:vfx/snow_drift", intensity: 0.2 ),
        post_fx: ( saturation_mul: 0.78, exposure_mul: 0.94, contrast_add: 0.02 ),
        allowed_weather: [
            "core:weather/clear",
            "core:weather/light_snow",
            "core:weather/blizzard",
        ],
        weather_weights: { "blizzard": 1.6, "aurora_calm": 1.2 },
        aurora: (
            latitude_threshold: 0.78,
            color_a: (0.10, 0.92, 0.55),
            color_b: (0.38, 0.42, 0.95),
            intensity: 1.0,
        ),
    )"#;
    let a: RawBiomeAmbienceDef = parse("biome ambience", strip_wrapper(src));
    assert_eq!(a.allowed_weather.len(), 3);
    assert!(a.aurora.is_some());
    assert!((a.fog_tint_mul.2 - 1.05).abs() < 1e-5);
}

#[test]
fn biome_ambience_minimal_parses() {
    let src = r#"BiomeAmbience(
        display_name: "Plains",
    )"#;
    let a: RawBiomeAmbienceDef = parse("biome ambience min", strip_wrapper(src));
    assert!(a.aurora.is_none());
    assert!(a.allowed_weather.is_empty());
    assert!((a.fog_tint_mul.0 - 1.0).abs() < 1e-5);
}

// ── celestial body + star catalog ────────────────────────────────────────────

#[test]
fn celestial_sun_parses() {
    let src = r#"CelestialBody(
        display_name: "Sol Primary",
        kind: star,
        voxel_model: "core:voxel/celestial/sol",
        radius_m: 6.96e8,
        spin: ( axis: (0.0, 1.0, 0.0), period_s: 2160000.0 ),
        surface: (
            emissive_color: (1.0, 0.92, 0.78),
            emissive_intensity: 8.0,
            corona: ( inner: (1.0, 0.9, 0.6), outer: (1.0, 0.5, 0.2), radius_mul: 4.0 ),
        ),
        lod_billboard_distance_m: 1.0e8,
    )"#;
    let b: RawCelestialBodyDef = parse("celestial sun", strip_wrapper(src));
    assert_eq!(b.kind, RawCelestialKind::Star);
    assert!(b.orbit.is_none());
    assert!(b.surface.corona.is_some());
    assert!(b.visible_from_surface);
}

#[test]
fn celestial_moon_with_orbit_parses() {
    let src = r#"CelestialBody(
        display_name: "Luna",
        kind: moon,
        radius_m: 1.737e6,
        orbit: (
            parent: "core:celestial/terra",
            semi_major_axis_m: 3.844e8,
            period_s: 2360592.0,
        ),
        spin: ( axis: (0.0, 1.0, 0.0), period_s: 2360592.0 ),
        surface: ( emissive_color: (0.6, 0.6, 0.62) ),
    )"#;
    let b: RawCelestialBodyDef = parse("celestial moon", strip_wrapper(src));
    let o = b.orbit.expect("orbit");
    assert!((o.semi_major_axis_m - 3.844e8).abs() < 1.0);
    assert!(o.eccentricity.abs() < 1e-9);
}

#[test]
fn star_catalog_parses() {
    let src = r#"StarCatalog(
        display_name: "Milky Way (local)",
        seed: 3_226_499_301,
        star_count: 8000,
        magnitude_range: (-1.5, 6.0),
        spectral_distribution: [
            ( class: O, weight: 0.0001 ),
            ( class: B, weight: 0.001 ),
            ( class: A, weight: 0.01 ),
            ( class: F, weight: 0.03 ),
            ( class: G, weight: 0.08 ),
            ( class: K, weight: 0.13 ),
            ( class: M, weight: 0.76 ),
        ],
        milky_way: (
            density_texture: "core:texture/celestial/milky_way_density",
            tint: (0.85, 0.88, 1.0),
            intensity: 0.6,
        ),
        nebulae: [
            ( name: "Orion", center_lonlat: (1.42, -0.10), radius_rad: 0.08,
              color: (0.55, 0.45, 0.95), intensity: 0.3 ),
        ],
    )"#;
    let c: RawStarCatalogDef = parse("star catalog", strip_wrapper(src));
    assert_eq!(c.star_count, 8000);
    assert_eq!(c.spectral_distribution.len(), 7);
    assert!(c.milky_way.is_some());
    assert_eq!(c.nebulae.len(), 1);
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn strip_wrapper(text: &str) -> &str {
    let trimmed = text.trim_start();
    if let Some(open) = trimmed.find('(') {
        if trimmed[..open]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            && open > 0
        {
            return &trimmed[open..];
        }
    }
    text
}

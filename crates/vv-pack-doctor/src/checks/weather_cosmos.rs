//! Lints for weather profiles, biome ambience, celestial bodies and star
//! catalogs.
//!
//! Validates schemas authored under `defs/world/{weather, biome_ambience,
//! celestial, star_catalogs}/`. Errors block the doctor; warnings flag
//! probable mistakes (V1 bounds, missing references, eccentric orbits).
//!
//! See `docs/v1/13_WEATHER_AND_COSMOS.md` §3 for the field semantics this
//! file enforces.

use std::path::Path;

use vv_content_schema::{
    RawBiomeAmbienceDef, RawCelestialBodyDef, RawStarCatalogDef, RawWeatherProfileDef,
};

use crate::parse::read_typed;
use crate::report::{Diagnostic, Report};
use crate::scan::PackScan;

const CHECK: &str = "weather_cosmos";

pub fn run(scan: &PackScan, report: &mut Report) {
    let defs = scan.pack_root.join("defs").join("world");
    check_weather(&defs.join("weather"), &scan.pack_root, report);
    check_biome_ambience(&defs.join("biome_ambience"), &scan.pack_root, report);
    check_celestial(&defs.join("celestial"), &scan.pack_root, report);
    check_star_catalogs(&defs.join("star_catalogs"), &scan.pack_root, report);
}

// ── weather ──────────────────────────────────────────────────────────────────

fn check_weather(dir: &Path, pack_root: &Path, report: &mut Report) {
    let Some(files) = ron_files_with(dir, ".weather.ron") else {
        return;
    };
    for path in files {
        let rel = crate::parse::pack_relative(pack_root, &path);
        let def: RawWeatherProfileDef = match read_typed(pack_root, &path) {
            Ok(d) => d,
            Err(err) => {
                report.add_parse_error(&err);
                continue;
            }
        };

        if def.display_name.trim().is_empty() {
            push_err(report, &rel, "display_name must not be empty", Some("name"));
        }
        check_unit(report, &rel, "rarity", def.rarity);
        check_unit(report, &rel, "cloud_coverage", def.cloud_coverage);
        check_positive(report, &rel, "cloud_density_mul", def.cloud_density_mul);
        check_non_negative(report, &rel, "cloud_speed_mul", def.cloud_speed_mul);
        check_positive(report, &rel, "fog_multiplier", def.fog_multiplier);

        for (biome, weight) in &def.biome_bias {
            if biome.trim().is_empty() {
                push_err(report, &rel, "biome_bias key is empty", Some("biome_bias"));
            }
            if *weight < 0.0 {
                push_err(
                    report,
                    &rel,
                    &format!("biome_bias[{biome}] = {weight} must be >= 0"),
                    Some("biome_bias"),
                );
            }
        }

        if let Some(tint) = def.cloud_tint {
            check_color(report, &rel, "cloud_tint", tint);
        }
        if let Some(tint) = def.fog_tint {
            check_color(report, &rel, "fog_tint", tint);
        }

        if let Some(p) = &def.precipitation {
            check_unit(report, &rel, "precipitation.intensity", p.intensity);
            check_unit(report, &rel, "precipitation.wind_drift", p.wind_drift);
            check_unit(
                report,
                &rel,
                "precipitation.splash_density",
                p.splash_density,
            );
        }

        let wind = &def.wind;
        check_non_negative(report, &rel, "wind.base_speed", wind.base_speed);
        check_non_negative(report, &rel, "wind.gust_speed", wind.gust_speed);
        if wind.gust_speed < wind.base_speed {
            push_err(
                report,
                &rel,
                &format!(
                    "wind.gust_speed ({}) must be >= wind.base_speed ({})",
                    wind.gust_speed, wind.base_speed
                ),
                Some("wind"),
            );
        }
        check_positive(report, &rel, "wind.gust_interval_s", wind.gust_interval_s);

        if let Some(l) = &def.lightning {
            check_non_negative(
                report,
                &rel,
                "lightning.strikes_per_minute",
                l.strikes_per_minute,
            );
            if l.strikes_per_minute > 60.0 {
                push_warn(
                    report,
                    &rel,
                    &format!(
                        "lightning.strikes_per_minute = {} is unusually high (>60)",
                        l.strikes_per_minute
                    ),
                    Some("lightning"),
                );
            }
            check_non_negative(report, &rel, "lightning.flash_intensity", l.flash_intensity);
            check_non_negative(
                report,
                &rel,
                "lightning.thunder_delay_per_km",
                l.thunder_delay_per_km,
            );
        }

        let t = &def.transitions;
        check_positive(report, &rel, "transitions.fade_in_s", t.fade_in_s);
        check_positive(report, &rel, "transitions.fade_out_s", t.fade_out_s);
        check_positive(report, &rel, "transitions.min_duration_s", t.min_duration_s);
        if t.max_duration_s < t.min_duration_s {
            push_err(
                report,
                &rel,
                &format!(
                    "transitions.max_duration_s ({}) must be >= min_duration_s ({})",
                    t.max_duration_s, t.min_duration_s
                ),
                Some("transitions"),
            );
        }
    }
}

// ── biome ambience ───────────────────────────────────────────────────────────

fn check_biome_ambience(dir: &Path, pack_root: &Path, report: &mut Report) {
    let Some(files) = ron_files_with(dir, ".biome_ambience.ron") else {
        return;
    };
    for path in files {
        let rel = crate::parse::pack_relative(pack_root, &path);
        let def: RawBiomeAmbienceDef = match read_typed(pack_root, &path) {
            Ok(d) => d,
            Err(err) => {
                report.add_parse_error(&err);
                continue;
            }
        };

        if def.display_name.trim().is_empty() {
            push_err(report, &rel, "display_name must not be empty", Some("name"));
        }
        check_non_negative(report, &rel, "fog_tint_mul.r", def.fog_tint_mul.0);
        check_non_negative(report, &rel, "fog_tint_mul.g", def.fog_tint_mul.1);
        check_non_negative(report, &rel, "fog_tint_mul.b", def.fog_tint_mul.2);
        if let Some(t) = def.sky_horizon_tint {
            check_color(report, &rel, "sky_horizon_tint", t);
        }
        check_unit(
            report,
            &rel,
            "ambient_dust_density",
            def.ambient_dust_density,
        );

        if let Some(p) = &def.ambient_particles {
            check_unit(report, &rel, "ambient_particles.intensity", p.intensity);
        }

        for (id, weight) in &def.weather_weights {
            if id.trim().is_empty() {
                push_err(
                    report,
                    &rel,
                    "weather_weights key is empty",
                    Some("weather_weights"),
                );
            }
            if *weight < 0.0 {
                push_err(
                    report,
                    &rel,
                    &format!("weather_weights[{id}] = {weight} must be >= 0"),
                    Some("weather_weights"),
                );
            }
        }

        if let Some(a) = &def.aurora {
            check_unit(
                report,
                &rel,
                "aurora.latitude_threshold",
                a.latitude_threshold,
            );
            check_color(report, &rel, "aurora.color_a", a.color_a);
            check_color(report, &rel, "aurora.color_b", a.color_b);
            check_non_negative(report, &rel, "aurora.intensity", a.intensity);
        }
    }
}

// ── celestial bodies ─────────────────────────────────────────────────────────

fn check_celestial(dir: &Path, pack_root: &Path, report: &mut Report) {
    let Some(files) = ron_files_with(dir, ".celestial.ron") else {
        return;
    };
    for path in files {
        let rel = crate::parse::pack_relative(pack_root, &path);
        let def: RawCelestialBodyDef = match read_typed(pack_root, &path) {
            Ok(d) => d,
            Err(err) => {
                report.add_parse_error(&err);
                continue;
            }
        };

        if def.display_name.trim().is_empty() {
            push_err(report, &rel, "display_name must not be empty", Some("name"));
        }
        if def.radius_m <= 0.0 {
            push_err(
                report,
                &rel,
                &format!("radius_m = {} must be > 0", def.radius_m),
                Some("radius_m"),
            );
        }

        if let Some(o) = &def.orbit {
            if o.semi_major_axis_m <= 0.0 {
                push_err(
                    report,
                    &rel,
                    &format!(
                        "orbit.semi_major_axis_m = {} must be > 0",
                        o.semi_major_axis_m
                    ),
                    Some("orbit"),
                );
            }
            if o.period_s <= 0.0 {
                push_err(
                    report,
                    &rel,
                    &format!("orbit.period_s = {} must be > 0", o.period_s),
                    Some("orbit"),
                );
            }
            if !(0.0..1.0).contains(&o.eccentricity) {
                push_err(
                    report,
                    &rel,
                    &format!("orbit.eccentricity = {} must be in [0, 1)", o.eccentricity),
                    Some("orbit"),
                );
            } else if o.eccentricity > 1e-6 {
                push_warn(
                    report,
                    &rel,
                    "orbit.eccentricity > 0 — V1 solver only supports circular orbits",
                    Some("orbit"),
                );
            }
        }

        if def.spin.period_s <= 0.0 {
            push_err(
                report,
                &rel,
                &format!("spin.period_s = {} must be > 0", def.spin.period_s),
                Some("spin"),
            );
        }
        let axis_len_sq = (def.spin.axis.0 * def.spin.axis.0)
            + (def.spin.axis.1 * def.spin.axis.1)
            + (def.spin.axis.2 * def.spin.axis.2);
        if axis_len_sq < 1e-6 {
            push_err(
                report,
                &rel,
                "spin.axis must not be the zero vector",
                Some("spin"),
            );
        }

        // surface.emissive_color is HDR — allow > 1, only reject negatives.
        let s = &def.surface;
        if s.emissive_color.0 < 0.0 || s.emissive_color.1 < 0.0 || s.emissive_color.2 < 0.0 {
            push_err(
                report,
                &rel,
                "surface.emissive_color components must be >= 0",
                Some("surface"),
            );
        }
        check_non_negative(
            report,
            &rel,
            "surface.emissive_intensity",
            s.emissive_intensity,
        );
        if let Some(c) = &s.corona {
            check_color(report, &rel, "surface.corona.inner", c.inner);
            check_color(report, &rel, "surface.corona.outer", c.outer);
            check_positive(report, &rel, "surface.corona.radius_mul", c.radius_mul);
        }

        if def.lod_billboard_distance_m < 0.0 {
            push_err(
                report,
                &rel,
                &format!(
                    "lod_billboard_distance_m = {} must be >= 0",
                    def.lod_billboard_distance_m
                ),
                Some("lod_billboard_distance_m"),
            );
        }
    }
}

// ── star catalogs ────────────────────────────────────────────────────────────

const STAR_COUNT_V1_CAP: u32 = 8192;

fn check_star_catalogs(dir: &Path, pack_root: &Path, report: &mut Report) {
    let Some(files) = ron_files_with(dir, ".star_catalog.ron") else {
        return;
    };
    for path in files {
        let rel = crate::parse::pack_relative(pack_root, &path);
        let def: RawStarCatalogDef = match read_typed(pack_root, &path) {
            Ok(d) => d,
            Err(err) => {
                report.add_parse_error(&err);
                continue;
            }
        };

        if def.display_name.trim().is_empty() {
            push_err(report, &rel, "display_name must not be empty", Some("name"));
        }
        if def.star_count == 0 {
            push_err(report, &rel, "star_count must be > 0", Some("star_count"));
        }
        if def.star_count > STAR_COUNT_V1_CAP {
            push_warn(
                report,
                &rel,
                &format!(
                    "star_count = {} exceeds V1 cap of {STAR_COUNT_V1_CAP}",
                    def.star_count
                ),
                Some("star_count"),
            );
        }

        let (mag_min, mag_max) = def.magnitude_range;
        if mag_min >= mag_max {
            push_err(
                report,
                &rel,
                &format!("magnitude_range = ({mag_min}, {mag_max}) must satisfy min < max"),
                Some("magnitude_range"),
            );
        }

        let mut sum = 0.0_f32;
        for w in &def.spectral_distribution {
            if w.weight < 0.0 {
                push_err(
                    report,
                    &rel,
                    &format!("spectral_distribution[{:?}] weight must be >= 0", w.class),
                    Some("spectral_distribution"),
                );
            }
            sum += w.weight;
        }
        if !def.spectral_distribution.is_empty() && (sum - 1.0).abs() > 0.05 {
            push_warn(
                report,
                &rel,
                &format!("spectral_distribution weights sum to {sum:.3} — expected ≈ 1.0"),
                Some("spectral_distribution"),
            );
        }

        if let Some(m) = &def.milky_way {
            check_color(report, &rel, "milky_way.tint", m.tint);
            check_non_negative(report, &rel, "milky_way.intensity", m.intensity);
        }

        for n in &def.nebulae {
            if n.name.trim().is_empty() {
                push_err(report, &rel, "nebula name is empty", Some("nebulae"));
            }
            check_color(report, &rel, &format!("nebula[{}].color", n.name), n.color);
            check_non_negative(
                report,
                &rel,
                &format!("nebula[{}].intensity", n.name),
                n.intensity,
            );
            if n.radius_rad <= 0.0 {
                push_err(
                    report,
                    &rel,
                    &format!(
                        "nebula[{}].radius_rad = {} must be > 0",
                        n.name, n.radius_rad
                    ),
                    Some("nebulae"),
                );
            }
        }
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn ron_files_with(dir: &Path, suffix: &str) -> Option<Vec<std::path::PathBuf>> {
    if !dir.exists() {
        return None;
    }
    let mut out = Vec::new();
    walk(dir, suffix, &mut out);
    out.sort();
    Some(out)
}

fn walk(dir: &Path, suffix: &str, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, suffix, out);
        } else if path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n.ends_with(suffix))
        {
            out.push(path);
        }
    }
}

fn check_unit(report: &mut Report, rel: &str, field: &str, v: f32) {
    if !(0.0..=1.0).contains(&v) {
        push_err(
            report,
            rel,
            &format!("{field} = {v} must be in [0, 1]"),
            Some(field),
        );
    }
}

fn check_non_negative(report: &mut Report, rel: &str, field: &str, v: f32) {
    if v < 0.0 {
        push_err(
            report,
            rel,
            &format!("{field} = {v} must be >= 0"),
            Some(field),
        );
    }
}

fn check_positive(report: &mut Report, rel: &str, field: &str, v: f32) {
    if v <= 0.0 {
        push_err(
            report,
            rel,
            &format!("{field} = {v} must be > 0"),
            Some(field),
        );
    }
}

fn check_color(report: &mut Report, rel: &str, field: &str, c: (f32, f32, f32)) {
    let names = ["r", "g", "b"];
    let comps = [c.0, c.1, c.2];
    for (i, v) in comps.iter().enumerate() {
        if !(0.0..=1.0).contains(v) {
            push_err(
                report,
                rel,
                &format!("{field}.{} = {v} must be in [0, 1]", names[i]),
                Some(field),
            );
        }
    }
}

fn push_err(report: &mut Report, rel: &str, message: &str, field: Option<&str>) {
    let mut d = Diagnostic::new(CHECK, message).with_path(rel.to_string());
    if let Some(f) = field {
        d = d.with_field(f.to_string());
    }
    report.error(d);
}

fn push_warn(report: &mut Report, rel: &str, message: &str, field: Option<&str>) {
    let mut d = Diagnostic::new(CHECK, message).with_path(rel.to_string());
    if let Some(f) = field {
        d = d.with_field(f.to_string());
    }
    report.warn(d);
}

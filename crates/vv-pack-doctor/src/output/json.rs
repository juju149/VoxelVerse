//! Minimal hand-written JSON writer for the Pack Doctor report.
//!
//! We avoid pulling in `serde_json` to keep the workspace dependency surface
//! tight - the report shape is fixed, so manual emission stays readable.

use crate::report::{
    BiomeSetSummary, BiomeSummary, CaveSummary, Diagnostic, FeatureSummary, Missing, OreSummary,
    PlanetProfileSummary, PlanetReport, Progression, RenderProfileSummary, Report, Summary, Unused,
};

pub fn render(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    field_str(
        &mut out,
        "pack",
        &report.pack_root.display().to_string(),
        1,
        true,
    );
    field_num(
        &mut out,
        "health_score",
        report.health_score as i64,
        1,
        true,
    );
    field_summary(&mut out, &report.summary, 1, true);
    field_diagnostics(&mut out, "errors", &report.errors, 1, true);
    field_diagnostics(&mut out, "warnings", &report.warnings, 1, true);
    field_unused(&mut out, &report.unused, 1, true);
    field_missing(&mut out, &report.missing, 1, true);
    field_planet(&mut out, &report.planet, 1, true);
    field_progression(&mut out, &report.progression, 1, false);
    out.push_str("}\n");
    out
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn field_str(out: &mut String, key: &str, value: &str, depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    write_string(out, value);
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_num(out: &mut String, key: &str, value: i64, depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(&value.to_string());
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_bool(out: &mut String, key: &str, value: bool, depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(if value { "true" } else { "false" });
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn write_string(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn field_string_array(out: &mut String, key: &str, items: &[String], depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    if items.is_empty() {
        out.push_str("[]");
    } else {
        out.push_str("[\n");
        for (i, item) in items.iter().enumerate() {
            indent(out, depth + 1);
            write_string(out, item);
            if i + 1 != items.len() {
                out.push(',');
            }
            out.push('\n');
        }
        indent(out, depth);
        out.push(']');
    }
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_summary(out: &mut String, summary: &Summary, depth: usize, comma: bool) {
    indent(out, depth);
    out.push_str("\"summary\": {\n");
    field_num(out, "errors", summary.errors as i64, depth + 1, true);
    field_num(out, "warnings", summary.warnings as i64, depth + 1, true);
    field_num(out, "blocks", summary.blocks as i64, depth + 1, true);
    field_num(out, "items", summary.items as i64, depth + 1, true);
    field_num(out, "materials", summary.materials as i64, depth + 1, true);
    field_num(out, "textures", summary.textures as i64, depth + 1, true);
    field_num(out, "recipes", summary.recipes as i64, depth + 1, true);
    field_num(
        out,
        "loot_tables",
        summary.loot_tables as i64,
        depth + 1,
        true,
    );
    field_num(out, "voxels", summary.voxels as i64, depth + 1, true);
    field_num(
        out,
        "shader_modules",
        summary.shader_modules as i64,
        depth + 1,
        true,
    );
    field_num(
        out,
        "techniques",
        summary.techniques as i64,
        depth + 1,
        true,
    );
    field_num(
        out,
        "world_files",
        summary.world_files as i64,
        depth + 1,
        false,
    );
    indent(out, depth);
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_diagnostics(out: &mut String, key: &str, diags: &[Diagnostic], depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    if diags.is_empty() {
        out.push_str("[]");
    } else {
        out.push_str("[\n");
        for (i, d) in diags.iter().enumerate() {
            indent(out, depth + 1);
            out.push_str("{\n");
            field_str(out, "check", d.check, depth + 2, true);
            field_str(out, "message", &d.message, depth + 2, true);
            indent(out, depth + 2);
            out.push_str("\"path\": ");
            match &d.path {
                Some(p) => write_string(out, p),
                None => out.push_str("null"),
            }
            out.push_str(",\n");
            indent(out, depth + 2);
            out.push_str("\"id\": ");
            match &d.id {
                Some(p) => write_string(out, p),
                None => out.push_str("null"),
            }
            out.push_str(",\n");
            indent(out, depth + 2);
            out.push_str("\"field\": ");
            match &d.field {
                Some(f) => write_string(out, f),
                None => out.push_str("null"),
            }
            out.push_str(",\n");
            indent(out, depth + 2);
            out.push_str("\"suggestion\": ");
            match &d.suggestion {
                Some(s) => write_string(out, s),
                None => out.push_str("null"),
            }
            out.push('\n');
            indent(out, depth + 1);
            out.push('}');
            if i + 1 != diags.len() {
                out.push(',');
            }
            out.push('\n');
        }
        indent(out, depth);
        out.push(']');
    }
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_unused(out: &mut String, unused: &Unused, depth: usize, comma: bool) {
    indent(out, depth);
    out.push_str("\"unused\": {\n");
    field_string_array(out, "textures", &unused.textures, depth + 1, true);
    field_string_array(out, "materials", &unused.materials, depth + 1, true);
    field_string_array(out, "items", &unused.items, depth + 1, true);
    field_string_array(out, "blocks", &unused.blocks, depth + 1, true);
    field_string_array(out, "loot_tables", &unused.loot_tables, depth + 1, true);
    field_string_array(out, "voxels", &unused.voxels, depth + 1, true);
    field_string_array(out, "shaders", &unused.shaders, depth + 1, false);
    indent(out, depth);
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_missing(out: &mut String, missing: &Missing, depth: usize, comma: bool) {
    indent(out, depth);
    out.push_str("\"missing\": {\n");
    field_string_array(out, "block_items", &missing.block_items, depth + 1, true);
    field_string_array(out, "loot_tables", &missing.loot_tables, depth + 1, true);
    field_string_array(out, "textures", &missing.textures, depth + 1, true);
    field_string_array(out, "voxels", &missing.voxels, depth + 1, true);
    field_string_array(out, "shaders", &missing.shaders, depth + 1, false);
    indent(out, depth);
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_progression(out: &mut String, progression: &Progression, depth: usize, comma: bool) {
    indent(out, depth);
    out.push_str("\"progression\": {\n");
    field_bool(
        out,
        "basic_loop_reachable",
        progression.basic_loop_reachable,
        depth + 1,
        true,
    );
    field_string_array(out, "notes", &progression.notes, depth + 1, false);
    indent(out, depth);
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_planet(out: &mut String, planet: &PlanetReport, depth: usize, comma: bool) {
    indent(out, depth);
    out.push_str("\"planet\": {\n");
    indent(out, depth + 1);
    out.push_str("\"counts\": {\n");
    field_num(
        out,
        "planet_profiles",
        planet.counts.planet_profiles as i64,
        depth + 2,
        true,
    );
    field_num(
        out,
        "biome_sets",
        planet.counts.biome_sets as i64,
        depth + 2,
        true,
    );
    field_num(out, "biomes", planet.counts.biomes as i64, depth + 2, true);
    field_num(
        out,
        "vegetation_rules",
        planet.counts.vegetation_rules as i64,
        depth + 2,
        true,
    );
    field_num(
        out,
        "prop_scatters",
        planet.counts.prop_scatters as i64,
        depth + 2,
        true,
    );
    field_num(
        out,
        "ore_rules",
        planet.counts.ore_rules as i64,
        depth + 2,
        true,
    );
    field_num(
        out,
        "cave_rules",
        planet.counts.cave_rules as i64,
        depth + 2,
        true,
    );
    field_num(
        out,
        "render_profiles",
        planet.counts.render_profiles as i64,
        depth + 2,
        false,
    );
    indent(out, depth + 1);
    out.push_str("},\n");
    field_planet_profiles(out, &planet.planet_profiles, depth + 1, true);
    field_biome_sets(out, &planet.biome_sets, depth + 1, true);
    field_biomes(out, &planet.biomes, depth + 1, true);
    field_features(out, &planet.features, depth + 1, true);
    field_ores(out, &planet.ores, depth + 1, true);
    field_caves(out, &planet.caves, depth + 1, true);
    field_render_profiles(out, &planet.render_profiles, depth + 1, true);
    field_string_array(out, "budget_notes", &planet.budget_notes, depth + 1, false);
    indent(out, depth);
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_planet_profiles(
    out: &mut String,
    items: &[PlanetProfileSummary],
    depth: usize,
    comma: bool,
) {
    field_object_array(
        out,
        "planet_profiles",
        items.len(),
        depth,
        comma,
        |out, i| {
            let p = &items[i];
            field_str(out, "id", &p.id, depth + 2, true);
            field_str(out, "display_name", &p.display_name, depth + 2, true);
            field_num(
                out,
                "near_voxel_lod_radius",
                p.near_voxel_lod_radius as i64,
                depth + 2,
                true,
            );
            field_num(
                out,
                "far_surface_lod_radius",
                p.far_surface_lod_radius as i64,
                depth + 2,
                true,
            );
            field_num(
                out,
                "upload_budget_chunks_per_frame",
                p.upload_budget_chunks_per_frame as i64,
                depth + 2,
                true,
            );
            field_num(
                out,
                "region_cell_voxels",
                p.region_cell_voxels as i64,
                depth + 2,
                true,
            );
            field_num(
                out,
                "feature_budget_per_chunk",
                p.feature_budget_per_chunk as i64,
                depth + 2,
                true,
            );
            field_num(
                out,
                "vegetation_refs",
                p.vegetation_refs as i64,
                depth + 2,
                true,
            );
            field_num(
                out,
                "prop_scatter_refs",
                p.prop_scatter_refs as i64,
                depth + 2,
                true,
            );
            field_num(out, "ore_refs", p.ore_refs as i64, depth + 2, true);
            field_num(out, "cave_refs", p.cave_refs as i64, depth + 2, false);
        },
    );
}

fn field_biome_sets(out: &mut String, items: &[BiomeSetSummary], depth: usize, comma: bool) {
    field_object_array(out, "biome_sets", items.len(), depth, comma, |out, i| {
        let b = &items[i];
        field_str(out, "id", &b.id, depth + 2, true);
        field_str(out, "display_name", &b.display_name, depth + 2, true);
        field_num(out, "selectors", b.selectors as i64, depth + 2, true);
        field_f32(out, "blend_radius", b.blend_radius, depth + 2, false);
    });
}

fn field_biomes(out: &mut String, items: &[BiomeSummary], depth: usize, comma: bool) {
    field_object_array(out, "biomes", items.len(), depth, comma, |out, i| {
        let b = &items[i];
        field_str(out, "id", &b.id, depth + 2, true);
        field_str(out, "display_name", &b.display_name, depth + 2, true);
        field_str(out, "surface_top", &b.surface_top, depth + 2, true);
        field_str(out, "surface_under", &b.surface_under, depth + 2, true);
        field_f32(out, "amplitude", b.amplitude, depth + 2, true);
        field_f32(out, "flatness", b.flatness, depth + 2, true);
        field_string_array(out, "tags", &b.tags, depth + 2, false);
    });
}

fn field_features(out: &mut String, items: &[FeatureSummary], depth: usize, comma: bool) {
    field_object_array(out, "features", items.len(), depth, comma, |out, i| {
        let f = &items[i];
        field_str(out, "id", &f.id, depth + 2, true);
        field_str(out, "kind", &f.kind, depth + 2, true);
        field_f32(out, "density", f.density, depth + 2, true);
        field_f32(
            out,
            "min_spacing_voxels",
            f.min_spacing_voxels,
            depth + 2,
            true,
        );
        field_num(
            out,
            "variant_count",
            f.variant_count as i64,
            depth + 2,
            false,
        );
    });
}

fn field_ores(out: &mut String, items: &[OreSummary], depth: usize, comma: bool) {
    field_object_array(out, "ores", items.len(), depth, comma, |out, i| {
        let o = &items[i];
        field_str(out, "id", &o.id, depth + 2, true);
        field_str(out, "block", &o.block, depth + 2, true);
        field_f32(out, "density", o.density, depth + 2, true);
        field_u32_pair(out, "depth_voxels", o.depth_voxels, depth + 2, false);
    });
}

fn field_caves(out: &mut String, items: &[CaveSummary], depth: usize, comma: bool) {
    field_object_array(out, "caves", items.len(), depth, comma, |out, i| {
        let c = &items[i];
        field_str(out, "id", &c.id, depth + 2, true);
        field_num(out, "fields", c.fields as i64, depth + 2, true);
        field_u32_pair(out, "depth_voxels", c.depth_voxels, depth + 2, true);
        field_f32_pair(out, "tunnel_radius", c.tunnel_radius, depth + 2, true);
        field_f32_pair(out, "chamber_radius", c.chamber_radius, depth + 2, false);
    });
}

fn field_render_profiles(
    out: &mut String,
    items: &[RenderProfileSummary],
    depth: usize,
    comma: bool,
) {
    field_object_array(
        out,
        "render_profiles",
        items.len(),
        depth,
        comma,
        |out, i| {
            let p = &items[i];
            field_str(out, "id", &p.id, depth + 2, true);
            field_str(out, "label", &p.label, depth + 2, true);
            field_str(out, "quality_class", &p.quality_class, depth + 2, true);
            field_bool(out, "fog", p.fog, depth + 2, true);
            field_str(out, "water", &p.water, depth + 2, true);
            field_num(
                out,
                "enabled_features",
                p.enabled_features as i64,
                depth + 2,
                false,
            );
        },
    );
}

fn field_object_array(
    out: &mut String,
    key: &str,
    len: usize,
    depth: usize,
    comma: bool,
    mut write_item: impl FnMut(&mut String, usize),
) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    if len == 0 {
        out.push_str("[]");
    } else {
        out.push_str("[\n");
        for i in 0..len {
            indent(out, depth + 1);
            out.push_str("{\n");
            write_item(out, i);
            indent(out, depth + 1);
            out.push('}');
            if i + 1 != len {
                out.push(',');
            }
            out.push('\n');
        }
        indent(out, depth);
        out.push(']');
    }
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_f32(out: &mut String, key: &str, value: f32, depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(&format!("{value:.4}"));
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_u32_pair(out: &mut String, key: &str, value: (u32, u32), depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": [");
    out.push_str(&value.0.to_string());
    out.push_str(", ");
    out.push_str(&value.1.to_string());
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_f32_pair(out: &mut String, key: &str, value: (f32, f32), depth: usize, comma: bool) {
    indent(out, depth);
    out.push('"');
    out.push_str(key);
    out.push_str("\": [");
    out.push_str(&format!("{:.4}", value.0));
    out.push_str(", ");
    out.push_str(&format!("{:.4}", value.1));
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

//! Minimal hand-written JSON writer for the Pack Doctor report.
//!
//! We avoid pulling in `serde_json` to keep the workspace dependency surface
//! tight - the report shape is fixed, so manual emission stays readable.

use crate::report::{Diagnostic, Missing, Progression, Report, Summary, Unused};

pub fn render(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    field_str(&mut out, "pack", &report.pack_root.display().to_string(), 1, true);
    field_num(&mut out, "health_score", report.health_score as i64, 1, true);
    field_summary(&mut out, &report.summary, 1, true);
    field_diagnostics(&mut out, "errors", &report.errors, 1, true);
    field_diagnostics(&mut out, "warnings", &report.warnings, 1, true);
    field_unused(&mut out, &report.unused, 1, true);
    field_missing(&mut out, &report.missing, 1, true);
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

fn field_string_array(
    out: &mut String,
    key: &str,
    items: &[String],
    depth: usize,
    comma: bool,
) {
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
    field_num(out, "loot_tables", summary.loot_tables as i64, depth + 1, true);
    field_num(out, "voxels", summary.voxels as i64, depth + 1, true);
    field_num(out, "shader_modules", summary.shader_modules as i64, depth + 1, true);
    field_num(out, "techniques", summary.techniques as i64, depth + 1, true);
    field_num(out, "world_files", summary.world_files as i64, depth + 1, false);
    indent(out, depth);
    out.push('}');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn field_diagnostics(
    out: &mut String,
    key: &str,
    diags: &[Diagnostic],
    depth: usize,
    comma: bool,
) {
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

fn field_progression(
    out: &mut String,
    progression: &Progression,
    depth: usize,
    comma: bool,
) {
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

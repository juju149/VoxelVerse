//! Minimal static HTML report. No framework, no JS - just enough markup to
//! browse errors, warnings, and the unused / missing lists.

use crate::report::{Diagnostic, Report};

pub fn render(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"en\"><head>\n");
    out.push_str("<meta charset=\"utf-8\">\n");
    out.push_str("<title>VoxelVerse Pack Doctor Report</title>\n");
    out.push_str(STYLES);
    out.push_str("</head><body>\n");

    out.push_str("<header>\n");
    out.push_str("<h1>VoxelVerse Pack Doctor</h1>\n");
    push_kv(&mut out, "Pack", &report.pack_root.display().to_string());
    push_kv(
        &mut out,
        "Health score",
        &format!("{}/100", report.health_score),
    );
    out.push_str("</header>\n");

    out.push_str("<section class=\"summary\">\n<h2>Summary</h2>\n<table>\n");
    push_row(&mut out, "Errors", report.summary.errors);
    push_row(&mut out, "Warnings", report.summary.warnings);
    push_row(&mut out, "Blocks", report.summary.blocks);
    push_row(&mut out, "Items", report.summary.items);
    push_row(&mut out, "Materials", report.summary.materials);
    push_row(&mut out, "Textures", report.summary.textures);
    push_row(&mut out, "Recipes", report.summary.recipes);
    push_row(&mut out, "Loot tables", report.summary.loot_tables);
    out.push_str("</table>\n</section>\n");

    push_diagnostics(&mut out, "Errors", &report.errors, "error");
    push_diagnostics(&mut out, "Warnings", &report.warnings, "warning");

    out.push_str("<section><h2>Unused</h2>\n");
    push_list(&mut out, "Textures", &report.unused.textures);
    push_list(&mut out, "Materials", &report.unused.materials);
    push_list(&mut out, "Items", &report.unused.items);
    push_list(&mut out, "Blocks", &report.unused.blocks);
    push_list(&mut out, "Loot tables", &report.unused.loot_tables);
    push_list(&mut out, "Voxels", &report.unused.voxels);
    push_list(&mut out, "Shaders", &report.unused.shaders);
    out.push_str("</section>\n");

    out.push_str("<section><h2>Missing</h2>\n");
    push_list(&mut out, "Block items", &report.missing.block_items);
    push_list(&mut out, "Loot tables", &report.missing.loot_tables);
    push_list(&mut out, "Textures", &report.missing.textures);
    push_list(&mut out, "Voxels", &report.missing.voxels);
    push_list(&mut out, "Shaders", &report.missing.shaders);
    out.push_str("</section>\n");

    out.push_str("<section><h2>Progression</h2>\n");
    push_kv(
        &mut out,
        "Basic loop reachable",
        if report.progression.basic_loop_reachable {
            "yes"
        } else {
            "no"
        },
    );
    if !report.progression.notes.is_empty() {
        out.push_str("<ul>\n");
        for note in &report.progression.notes {
            out.push_str("<li>");
            escape(&mut out, note);
            out.push_str("</li>\n");
        }
        out.push_str("</ul>\n");
    }
    out.push_str("</section>\n");

    out.push_str("</body></html>\n");
    out
}

fn push_kv(out: &mut String, key: &str, value: &str) {
    out.push_str("<p><strong>");
    escape(out, key);
    out.push_str(":</strong> ");
    escape(out, value);
    out.push_str("</p>\n");
}

fn push_row(out: &mut String, key: &str, value: usize) {
    out.push_str("<tr><th>");
    escape(out, key);
    out.push_str("</th><td>");
    out.push_str(&value.to_string());
    out.push_str("</td></tr>\n");
}

fn push_diagnostics(out: &mut String, title: &str, diags: &[Diagnostic], class: &str) {
    out.push_str("<section><h2>");
    escape(out, title);
    out.push_str(&format!(" ({})", diags.len()));
    out.push_str("</h2>\n");
    if diags.is_empty() {
        out.push_str("<p class=\"empty\">None.</p>\n");
    } else {
        out.push_str("<table class=\"diagnostics ");
        out.push_str(class);
        out.push_str("\">\n<thead><tr><th>Check</th><th>Message</th><th>Where</th><th>Field</th><th>Fix</th></tr></thead>\n<tbody>\n");
        for d in diags {
            out.push_str("<tr><td>");
            escape(out, d.check);
            out.push_str("</td><td>");
            escape(out, &d.message);
            out.push_str("</td><td>");
            match (&d.id, &d.path) {
                (Some(id), Some(path)) => {
                    escape(out, path);
                    out.push_str(" <small>");
                    escape(out, id);
                    out.push_str("</small>");
                }
                (Some(id), None) => escape(out, id),
                (None, Some(path)) => escape(out, path),
                (None, None) => out.push('-'),
            }
            out.push_str("</td><td>");
            if let Some(f) = &d.field {
                escape(out, f);
            } else {
                out.push('-');
            }
            out.push_str("</td><td>");
            if let Some(s) = &d.suggestion {
                escape(out, s);
            } else {
                out.push('-');
            }
            out.push_str("</td></tr>\n");
        }
        out.push_str("</tbody></table>\n");
    }
    out.push_str("</section>\n");
}

fn push_list(out: &mut String, title: &str, items: &[String]) {
    out.push_str("<details");
    if !items.is_empty() {
        out.push_str(" open");
    }
    out.push_str("><summary>");
    escape(out, title);
    out.push_str(&format!(" ({})", items.len()));
    out.push_str("</summary>\n");
    if items.is_empty() {
        out.push_str("<p class=\"empty\">None.</p>\n");
    } else {
        out.push_str("<ul>\n");
        for it in items {
            out.push_str("<li><code>");
            escape(out, it);
            out.push_str("</code></li>\n");
        }
        out.push_str("</ul>\n");
    }
    out.push_str("</details>\n");
}

fn escape(out: &mut String, value: &str) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
}

const STYLES: &str = r#"<style>
  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; max-width: 1080px; margin: 2rem auto; padding: 0 1rem; color: #222; }
  header { border-bottom: 1px solid #ddd; padding-bottom: 1rem; margin-bottom: 1rem; }
  h1 { margin: 0 0 0.5rem; }
  h2 { margin-top: 2rem; }
  table { border-collapse: collapse; width: 100%; }
  th, td { padding: 0.4rem 0.6rem; text-align: left; border-bottom: 1px solid #eee; }
  .summary th { width: 12rem; }
  .diagnostics.error tbody tr { background: #fff4f4; }
  .diagnostics.warning tbody tr { background: #fff9e6; }
  code { background: #f4f4f4; padding: 1px 4px; border-radius: 3px; }
  .empty { color: #888; }
  details { margin: 0.5rem 0; }
  summary { cursor: pointer; font-weight: 600; }
</style>
"#;

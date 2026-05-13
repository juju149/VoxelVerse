//! Tag declarations and usages.
//!
//! Tags are arbitrary strings attached to objects (`tags: ["soil", "terrain"]`).
//! Other files refer to them via `#tag.<name>`, `#station.<name>`, etc.
//! This check verifies every referenced tag was actually declared somewhere
//! and warns about declared tags that no file ever references.

use std::collections::{BTreeMap, BTreeSet};

use crate::index::{parse_tag_ref, PackIndex};
use crate::report::{Diagnostic, Report};

const CHECK: &str = "tags";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    // 1) Gather every referenced tag from objects + world files.
    let mut referenced: BTreeMap<String, Vec<TagUsage>> = BTreeMap::new();
    for obj in &index.scan.objects {
        // mining.tool isn't a tag; surface/biome refs are strings, so we walk
        // through any string field looking for the `#xxx.yyy` pattern.
        scan_object_for_tag_refs(obj, &mut referenced);
    }
    for file in &index.scan.world_files {
        scan_value_for_tag_refs(&file.value, &file.rel_path, &file.id, &mut referenced);
    }

    // 2) Build the set of declared tag identifiers. Tags carry their full
    // text in the object's `tags:` list, with sub-namespaces using dots.
    let declared: BTreeSet<String> = index.tags_declared.iter().cloned().collect();

    // 3) Flag unknown refs.
    for (tag, usages) in &referenced {
        if !declared.contains(tag) {
            for u in usages {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("unknown tag '#{}' — no object declares it", tag),
                    )
                    .with_path(u.path.clone())
                    .with_id(u.id.clone())
                    .with_suggestion(format!(
                        "add `{}` to the `tags:` list of the relevant object",
                        tag
                    )),
                );
            }
        }
    }

    // 4) Flag declared-but-unused.
    for tag in &declared {
        if !referenced.contains_key(tag) {
            report.warn(
                Diagnostic::new(
                    CHECK,
                    format!("tag '{}' is declared but never referenced via #{}", tag, tag),
                )
                .with_suggestion(
                    "remove the tag from `tags:` or reference it from a file that needs it"
                        .to_string(),
                ),
            );
        }
    }
}

struct TagUsage {
    path: String,
    id: String,
}

fn scan_object_for_tag_refs(
    obj: &crate::scan::ParsedObject,
    out: &mut BTreeMap<String, Vec<TagUsage>>,
) {
    for recipe in &obj.def.recipes {
        if let Some(station) = &recipe.station {
            record_tag_ref(station, obj.rel_path.clone(), obj.id.clone(), out);
        }
    }
}

fn scan_value_for_tag_refs(
    value: &ron::Value,
    rel_path: &str,
    id: &str,
    out: &mut BTreeMap<String, Vec<TagUsage>>,
) {
    match value {
        ron::Value::String(s) => {
            record_tag_ref(s, rel_path.to_string(), id.to_string(), out);
        }
        ron::Value::Seq(seq) => {
            for v in seq {
                scan_value_for_tag_refs(v, rel_path, id, out);
            }
        }
        ron::Value::Map(map) => {
            for (_, v) in map.iter() {
                scan_value_for_tag_refs(v, rel_path, id, out);
            }
        }
        ron::Value::Option(Some(inner)) => scan_value_for_tag_refs(inner, rel_path, id, out),
        _ => {}
    }
}

fn record_tag_ref(
    s: &str,
    path: String,
    id: String,
    out: &mut BTreeMap<String, Vec<TagUsage>>,
) {
    let Some((kind, name)) = parse_tag_ref(s) else { return };
    let full = format!("{kind}.{name}");
    out.entry(full).or_default().push(TagUsage { path, id });
}

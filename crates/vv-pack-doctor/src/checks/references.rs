//! Cross-object reference resolution.
//!
//! Walks every parsed object and checks that all short item references
//! (mining drops, loot, recipe ingredients, recipe legends, recipe outputs,
//! weapon ammo) point at an object that actually exists in the pack. World
//! files (biome/ore/...) are also inspected at the value level so a missing
//! `block: dirty_stone` produces an actionable diagnostic even if its
//! enclosing schema has drifted.

use vv_content_schema::{
    RawObjectCount, RawObjectDef, RawObjectRecipeKind, RawObjectRecipeSection,
};

use crate::index::PackIndex;
use crate::report::{Diagnostic, Report};
use crate::scan::{ParsedObject, ParsedWorldFile, WorldCategory};

const CHECK: &str = "references";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    for obj in &index.scan.objects {
        check_object(obj, index, report);
    }
    for file in &index.scan.world_files {
        check_world_file(file, index, report);
    }
}

fn check_object(obj: &ParsedObject, index: &PackIndex<'_>, report: &mut Report) {
    let RawObjectDef {
        mining,
        loot,
        recipes,
        weapon,
        ..
    } = &obj.def;

    if let Some(mining) = mining {
        if let Some(drops) = &mining.drops {
            for (i, drop) in drops.iter().enumerate() {
                if drop.item == "self" {
                    continue;
                }
                check_strict_object_ref(
                    &drop.item,
                    obj,
                    report,
                    &format!("mining.drops[{i}].item"),
                );
                if index.resolve_object(&drop.item).is_none() {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!(
                                "mining drop #{} references unknown item '{}'",
                                i + 1,
                                drop.item
                            ),
                        )
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field(format!("mining.drops[{i}].item"))
                        .with_suggestion(format!(
                            "create defs/objects/.../{}.object.ron or fix the spelling",
                            drop.item
                        )),
                    );
                }
                check_count(
                    &drop.count,
                    obj,
                    report,
                    &format!("mining.drops[{i}].count"),
                );
            }
        }
    }

    if let Some(loot) = loot {
        for (i, drop) in loot.when_killed.iter().enumerate() {
            check_strict_object_ref(
                &drop.item,
                obj,
                report,
                &format!("loot.when_killed[{i}].item"),
            );
            if index.resolve_object(&drop.item).is_none() {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "loot drop #{} references unknown item '{}'",
                            i + 1,
                            drop.item
                        ),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field(format!("loot.when_killed[{i}].item")),
                );
            }
            check_count(
                &drop.count,
                obj,
                report,
                &format!("loot.when_killed[{i}].count"),
            );
        }
    }

    if let Some(weapon) = weapon {
        if let Some(ammo) = &weapon.ammo {
            check_strict_object_ref(ammo, obj, report, "weapon.ammo");
            if index.resolve_object(ammo).is_none() {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!("weapon ammo references unknown item '{}'", ammo),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("weapon.ammo"),
                );
            }
        }
    }

    for (ri, recipe) in recipes.iter().enumerate() {
        check_recipe_refs(obj, ri, recipe, index, report);
    }
}

fn check_recipe_refs(
    obj: &ParsedObject,
    ri: usize,
    recipe: &RawObjectRecipeSection,
    index: &PackIndex<'_>,
    report: &mut Report,
) {
    if index.resolve_object(&recipe.output.item).is_none() {
        report.error(
            Diagnostic::new(
                CHECK,
                format!(
                    "recipe output references unknown item '{}'",
                    recipe.output.item
                ),
            )
            .with_path(obj.rel_path.clone())
            .with_id(obj.id.clone())
            .with_field(format!("recipes[{ri}].output.item")),
        );
    }
    check_strict_object_ref(
        &recipe.output.item,
        obj,
        report,
        &format!("recipes[{ri}].output.item"),
    );

    match &recipe.kind {
        RawObjectRecipeKind::Shaped(shaped) => {
            for (sym, item) in &shaped.legend {
                check_strict_object_ref(
                    item,
                    obj,
                    report,
                    &format!("recipes[{ri}].kind.shaped.legend.{sym}"),
                );
                if index.resolve_object(item).is_none() {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!("recipe legend '{}' references unknown item '{}'", sym, item),
                        )
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field(format!("recipes[{ri}].kind.shaped.legend.{sym}")),
                    );
                }
            }
        }
        RawObjectRecipeKind::Shapeless(shapeless) => {
            for (i, ingredient) in shapeless.ingredients.iter().enumerate() {
                check_strict_object_ref(
                    ingredient,
                    obj,
                    report,
                    &format!("recipes[{ri}].kind.shapeless.ingredients[{i}]"),
                );
                if index.resolve_object(ingredient).is_none() {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!(
                                "shapeless ingredient #{} references unknown item '{}'",
                                i + 1,
                                ingredient
                            ),
                        )
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field(format!("recipes[{ri}].kind.shapeless.ingredients[{i}]")),
                    );
                }
            }
        }
        RawObjectRecipeKind::Processing(processing) => {
            for (i, input) in processing.inputs.iter().enumerate() {
                check_strict_object_ref(
                    &input.item,
                    obj,
                    report,
                    &format!("recipes[{ri}].kind.processing.inputs[{i}].item"),
                );
                if index.resolve_object(&input.item).is_none() {
                    report.error(
                        Diagnostic::new(
                            CHECK,
                            format!(
                                "processing input #{} references unknown item '{}'",
                                i + 1,
                                input.item
                            ),
                        )
                        .with_path(obj.rel_path.clone())
                        .with_id(obj.id.clone())
                        .with_field(format!("recipes[{ri}].kind.processing.inputs[{i}].item")),
                    );
                }
            }
        }
    }
}

fn check_strict_object_ref(value: &str, obj: &ParsedObject, report: &mut Report, field: &str) {
    if value == "self" || is_strict_object_ref(value) {
        return;
    }
    report.error(
        Diagnostic::new(
            CHECK,
            format!(
                "short object reference '{}' is forbidden in V1; use `namespace:object/...`",
                value
            ),
        )
        .with_path(obj.rel_path.clone())
        .with_id(obj.id.clone())
        .with_field(field.to_string())
        .with_suggestion(format!(
            "replace '{}' with a fully-qualified object reference such as `core:object/<category>/{}`",
            value,
            value
        )),
    );
}

fn is_strict_object_ref(value: &str) -> bool {
    let Some((namespace, path)) = value.split_once(':') else {
        return false;
    };
    !namespace.is_empty() && path.starts_with("object/") && path.len() > "object/".len()
}

fn check_count(count: &RawObjectCount, obj: &ParsedObject, report: &mut Report, field: &str) {
    if let RawObjectCount::Range(min, max) = count {
        if min > max {
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!("drop count range is inverted: ({min}, {max})"),
                )
                .with_path(obj.rel_path.clone())
                .with_id(obj.id.clone())
                .with_field(field.to_string())
                .with_suggestion("write the range as (min, max) with min <= max".to_string()),
            );
        }
    }
}

fn check_world_file(file: &ParsedWorldFile, index: &PackIndex<'_>, report: &mut Report) {
    // The world schema is partly fluid: walk the parsed value and resolve any
    // string that "looks like" a short object reference.  We only flag fields
    // whose name suggests an object/block link, to avoid noisy false
    // positives.
    let object_link_fields = [
        "block",
        "blocks",
        "top",
        "under",
        "side",
        "bottom",
        "item",
        "trunk",
        "leaves",
        "air_block",
        "use",
        "replace",
        "replaces",
        "surface",
    ];
    walk(&file.value, &mut Vec::new(), &mut |path, value| {
        let Some(s) = value.as_str_value() else {
            return;
        };
        let Some(last) = path.last() else { return };
        if s.starts_with('#') {
            return; // tag — checked elsewhere
        }
        if s == "any" || s == "self" {
            return;
        }

        if last == "model" {
            // Voxel asset reference. Strip namespace prefix if present.
            let stripped = s.strip_prefix("core:").unwrap_or(s);
            let candidate = if stripped.starts_with("voxel/") {
                format!("media/{}.vox", stripped)
            } else {
                format!("media/voxel/{}.vox", stripped)
            };
            if !index.voxel_exists(&candidate) {
                report.missing.voxels.push(candidate.clone());
                report.error(
                    Diagnostic::new(
                        "references",
                        format!("voxel model '{}' is missing on disk", s),
                    )
                    .with_path(file.rel_path.clone())
                    .with_id(file.id.clone())
                    .with_field(path.join("."))
                    .with_suggestion(format!("create {}", candidate)),
                );
            }
            return;
        }

        if !object_link_fields.contains(&last.as_str()) {
            return;
        }

        if index.resolve_object(s).is_some() {
            return;
        }
        let cats = [
            WorldCategory::Biome,
            WorldCategory::Noise,
            WorldCategory::Climate,
            WorldCategory::BiomeSet,
            WorldCategory::Terrain,
            WorldCategory::Vegetation,
            WorldCategory::Caves,
            WorldCategory::Ores,
            WorldCategory::Structures,
            WorldCategory::PropScatter,
        ];
        if cats.iter().any(|c| index.resolve_world(*c, s).is_some()) {
            return;
        }
        report.error(
            Diagnostic::new(
                "references",
                format!("world file references unknown '{}'", s),
            )
            .with_path(file.rel_path.clone())
            .with_id(file.id.clone())
            .with_field(path.join("."))
            .with_suggestion(format!(
                "declare an object, biome, ore or world entity whose short name is '{}'",
                s.rsplit('/').next().unwrap_or(s)
            )),
        );
    });
}

trait ValueExt {
    fn as_str_value(&self) -> Option<&str>;
}

impl ValueExt for ron::Value {
    fn as_str_value(&self) -> Option<&str> {
        match self {
            ron::Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

fn walk(
    value: &ron::Value,
    path: &mut Vec<String>,
    visit: &mut impl FnMut(&[String], &ron::Value),
) {
    visit(path, value);
    match value {
        ron::Value::Map(map) => {
            for (k, v) in map.iter() {
                let key = match k {
                    ron::Value::String(s) => s.clone(),
                    ron::Value::Char(c) => c.to_string(),
                    other => format!("{other:?}"),
                };
                path.push(key);
                walk(v, path, visit);
                path.pop();
            }
        }
        ron::Value::Seq(seq) => {
            for (i, v) in seq.iter().enumerate() {
                path.push(format!("[{i}]"));
                walk(v, path, visit);
                path.pop();
            }
        }
        ron::Value::Option(Some(inner)) => walk(inner, path, visit),
        _ => {}
    }
}

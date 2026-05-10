//! Raw block-state schema.
//!
//! Sprint 0 scope: schema and semantic validation only. Variant compilation,
//! transformations, and runtime lookup arrive in subsequent sprint steps.
//!
//! Determinism is non-negotiable: properties are stored in a `BTreeMap` so
//! that iteration order is the lexicographic order of property names. Two
//! identical packs always produce the exact same sequence of compiled
//! variants and therefore the same `VoxelId`s build-to-build.
//!
//! Built-in property kinds know their allowed values; the compiler can apply
//! automatic mesh / collision / material transforms for them. The `Enum`
//! kind escapes to author-defined value sets but does not benefit from
//! built-in transformations — modders must wire those manually.

use serde::Deserialize;
use std::collections::BTreeMap;

/// Per-block declaration of orthogonal state properties (e.g. `axis`,
/// `facing`, `waterlogged`). The cartesian product of property values
/// produces the block's variants at compile time.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct RawBlockStates {
    /// Property name → kind + default. Stored in a `BTreeMap` so the
    /// compilation order is deterministic across hosts and runs.
    #[serde(default)]
    pub properties: BTreeMap<String, RawBlockStateProperty>,
}

/// A single state property: its kind constrains its allowed value set.
///
/// Built-in kinds (`Axis`, `FacingHorizontal`, `Facing`, `Half`,
/// `StairShape`, `Bool`) are semantically known to the compiler — it can
/// generate transformations automatically. The `Enum` kind allows arbitrary
/// author-defined value sets but produces no automatic transforms.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawBlockStateProperty {
    /// Local-axis orientation. Allowed values: `x | y | z`.
    Axis { default: String },
    /// Horizontal facing on the world plane. Allowed values:
    /// `north | south | east | west`.
    FacingHorizontal { default: String },
    /// Six-direction facing. Allowed values:
    /// `north | south | east | west | up | down`.
    Facing { default: String },
    /// Vertical half. Allowed values: `top | bottom`.
    Half { default: String },
    /// Stair connection shape. Allowed values:
    /// `straight | inner_left | inner_right | outer_left | outer_right`.
    StairShape { default: String },
    /// Boolean state (e.g. `waterlogged`, `powered`, `open`).
    Bool { default: bool },
    /// Author-defined enumerated values. No automatic transforms applied.
    Enum {
        values: Vec<String>,
        default: String,
    },
}

impl RawBlockStateProperty {
    /// The list of allowed values for built-in kinds. `Enum` returns its
    /// own `values` (non-built-in). `Bool` returns the canonical
    /// `["false", "true"]` for diagnostics only.
    pub fn allowed_values(&self) -> Vec<&str> {
        match self {
            Self::Axis { .. } => vec!["x", "y", "z"],
            Self::FacingHorizontal { .. } => vec!["north", "south", "east", "west"],
            Self::Facing { .. } => vec!["north", "south", "east", "west", "up", "down"],
            Self::Half { .. } => vec!["top", "bottom"],
            Self::StairShape { .. } => vec![
                "straight",
                "inner_left",
                "inner_right",
                "outer_left",
                "outer_right",
            ],
            Self::Bool { .. } => vec!["false", "true"],
            Self::Enum { values, .. } => values.iter().map(String::as_str).collect(),
        }
    }

    /// A short tag identifying the kind, used in diagnostics.
    pub fn kind_tag(&self) -> &'static str {
        match self {
            Self::Axis { .. } => "axis",
            Self::FacingHorizontal { .. } => "facing_horizontal",
            Self::Facing { .. } => "facing",
            Self::Half { .. } => "half",
            Self::StairShape { .. } => "stair_shape",
            Self::Bool { .. } => "bool",
            Self::Enum { .. } => "enum",
        }
    }

    /// Validate the property's default (and value-set sanity for `Enum`).
    /// Errors are appended to `errors` with `ctx` prepended for traceability.
    pub fn validate_into(&self, name: &str, ctx: &str, errors: &mut Vec<String>) {
        match self {
            Self::Axis { default }
            | Self::FacingHorizontal { default }
            | Self::Facing { default }
            | Self::Half { default }
            | Self::StairShape { default } => {
                let allowed = self.allowed_values();
                if !allowed.iter().any(|v| *v == default.as_str()) {
                    errors.push(format!(
                        "{ctx}: state '{name}' ({}) default '{}' not in [{}]",
                        self.kind_tag(),
                        default,
                        allowed.join(", ")
                    ));
                }
            }
            Self::Bool { .. } => {
                // bool defaults are typed; nothing to validate.
            }
            Self::Enum { values, default } => {
                if values.is_empty() {
                    errors.push(format!(
                        "{ctx}: state '{name}' (enum) has empty `values`"
                    ));
                    return;
                }
                let mut seen = std::collections::HashSet::new();
                for v in values {
                    if !seen.insert(v.as_str()) {
                        errors.push(format!(
                            "{ctx}: state '{name}' (enum) duplicate value '{}'",
                            v
                        ));
                    }
                }
                if !values.iter().any(|v| v == default) {
                    errors.push(format!(
                        "{ctx}: state '{name}' (enum) default '{}' not in declared values [{}]",
                        default,
                        values.join(", ")
                    ));
                }
            }
        }
    }
}

impl RawBlockStates {
    /// Validate every declared property. `ctx` should describe the
    /// containing block (e.g. `"block 'core:block/.../oak_log'"`).
    pub fn validate_into(&self, ctx: &str, errors: &mut Vec<String>) {
        for (name, prop) in &self.properties {
            prop.validate_into(name, ctx, errors);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    pub fn len(&self) -> usize {
        self.properties.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis(default: &str) -> RawBlockStateProperty {
        RawBlockStateProperty::Axis {
            default: default.into(),
        }
    }

    fn enum_prop(values: &[&str], default: &str) -> RawBlockStateProperty {
        RawBlockStateProperty::Enum {
            values: values.iter().map(|s| (*s).into()).collect(),
            default: default.into(),
        }
    }

    #[test]
    fn empty_states_accepted() {
        let s = RawBlockStates::default();
        let mut errors = Vec::new();
        s.validate_into("block 'x'", &mut errors);
        assert!(errors.is_empty());
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn valid_axis_accepted() {
        let mut s = RawBlockStates::default();
        s.properties.insert("axis".into(), axis("y"));
        let mut errors = Vec::new();
        s.validate_into("block 'log'", &mut errors);
        assert!(errors.is_empty(), "got: {errors:?}");
    }

    #[test]
    fn invalid_axis_rejected() {
        let mut s = RawBlockStates::default();
        s.properties.insert("axis".into(), axis("w"));
        let mut errors = Vec::new();
        s.validate_into("block 'log'", &mut errors);
        assert_eq!(errors.len(), 1);
        let e = &errors[0];
        assert!(e.contains("axis"), "msg: {e}");
        assert!(e.contains("'w'"), "msg: {e}");
        assert!(e.contains("[x, y, z]"), "msg: {e}");
    }

    #[test]
    fn enum_with_absent_default_rejected() {
        let mut s = RawBlockStates::default();
        s.properties
            .insert("color".into(), enum_prop(&["red", "green", "blue"], "purple"));
        let mut errors = Vec::new();
        s.validate_into("block 'paint'", &mut errors);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("'purple'"));
        assert!(errors[0].contains("[red, green, blue]"));
    }

    #[test]
    fn enum_with_empty_values_rejected() {
        let mut s = RawBlockStates::default();
        s.properties.insert("k".into(), enum_prop(&[], "x"));
        let mut errors = Vec::new();
        s.validate_into("block 'x'", &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("empty"));
    }

    #[test]
    fn enum_with_duplicate_values_rejected() {
        let mut s = RawBlockStates::default();
        s.properties
            .insert("k".into(), enum_prop(&["a", "b", "a"], "a"));
        let mut errors = Vec::new();
        s.validate_into("block 'x'", &mut errors);
        assert!(errors.iter().any(|e| e.contains("duplicate")));
    }

    #[test]
    fn property_iteration_order_is_deterministic() {
        // Insert in non-alphabetical order on purpose; BTreeMap should
        // surface them in lexicographic order regardless.
        let mut s = RawBlockStates::default();
        s.properties.insert("waterlogged".into(), RawBlockStateProperty::Bool { default: false });
        s.properties.insert("axis".into(), axis("y"));
        s.properties.insert("half".into(), RawBlockStateProperty::Half { default: "bottom".into() });

        let order: Vec<&str> = s.properties.keys().map(String::as_str).collect();
        assert_eq!(order, vec!["axis", "half", "waterlogged"]);

        // Re-iterate to ensure stability.
        let order2: Vec<&str> = s.properties.keys().map(String::as_str).collect();
        assert_eq!(order, order2);
    }

    #[test]
    fn ron_roundtrip_axis_property() {
        let src = r#"(
            properties: {
                "axis": axis(default: "y"),
                "waterlogged": bool(default: false),
            },
        )"#;
        let parsed: RawBlockStates =
            ron::from_str(src).expect("ron parses block states");
        assert_eq!(parsed.properties.len(), 2);
        match parsed.properties.get("axis").unwrap() {
            RawBlockStateProperty::Axis { default } => assert_eq!(default, "y"),
            other => panic!("expected axis, got {other:?}"),
        }
    }

    #[test]
    fn ron_roundtrip_enum_property() {
        let src = r#"(
            properties: {
                "color": enum(values: ["red", "green", "blue"], default: "red"),
            },
        )"#;
        let parsed: RawBlockStates =
            ron::from_str(src).expect("ron parses enum state");
        match parsed.properties.get("color").unwrap() {
            RawBlockStateProperty::Enum { values, default } => {
                assert_eq!(values.len(), 3);
                assert_eq!(default, "red");
            }
            other => panic!("expected enum, got {other:?}"),
        }
    }
}

use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn block_base_color(&mut self, doc: &RawDocument<BlockDef>) -> [f32; 4] {
        self.parse_hex_color(
            doc,
            "render.material.base_color",
            &doc.value.render.material.base_color,
        )
    }

    pub(super) fn parse_hex_color<T>(
        &mut self,
        doc: &RawDocument<T>,
        field: &str,
        color: &vv_schema::common::HexColor,
    ) -> [f32; 4] {
        match parse_hex_color(&color.0) {
            Some(color) => color,
            None => {
                self.invalid_value(
                    doc,
                    field,
                    &color.0,
                    "expected #RRGGBB or #RRGGBBAA hex color",
                );
                [1.0, 0.0, 1.0, 1.0]
            }
        }
    }

    pub(super) fn clamp_unit<T>(&mut self, doc: &RawDocument<T>, field: &str, value: f32) -> f32 {
        self.clamp_range(doc, field, value, 0.0, 1.0)
    }

    pub(super) fn positive_scale<T>(
        &mut self,
        doc: &RawDocument<T>,
        field: &str,
        value: f32,
    ) -> f32 {
        if value > 0.0 && value.is_finite() {
            value
        } else {
            self.invalid_value(
                doc,
                field,
                &value.to_string(),
                "expected a positive finite value",
            );
            1.0
        }
    }

    pub(super) fn grid_size<T>(&mut self, doc: &RawDocument<T>, field: &str, value: u32) -> u32 {
        if (1..=64).contains(&value) {
            value
        } else {
            self.invalid_value(
                doc,
                field,
                &value.to_string(),
                "expected a grid size between 1 and 64",
            );
            value.clamp(1, 64)
        }
    }

    pub(super) fn clamp_range<T>(
        &mut self,
        doc: &RawDocument<T>,
        field: &str,
        value: f32,
        min: f32,
        max: f32,
    ) -> f32 {
        if value.is_finite() && value >= min && value <= max {
            value
        } else {
            self.invalid_value(
                doc,
                field,
                &value.to_string(),
                &format!("expected a finite value between {min} and {max}"),
            );
            value.clamp(min, max)
        }
    }

    pub(super) fn invalid_value<T>(
        &mut self,
        doc: &RawDocument<T>,
        field: &str,
        value: &str,
        reason: &str,
    ) {
        self.diagnostics.push(CompileDiagnostic::InvalidValue {
            owner: "block".to_owned(),
            path: doc.source_path.clone(),
            field: field.to_owned(),
            value: value.to_owned(),
            reason: reason.to_owned(),
        });
    }
}

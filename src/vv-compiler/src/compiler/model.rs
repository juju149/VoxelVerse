use super::helpers::*;
use super::prelude::*;

impl ContentCompiler {
    pub(super) fn compile_render_details(
        &mut self,
        doc: &RawDocument<BlockDef>,
        render: &BlockRenderDef,
    ) -> SmallVec<[CompiledBlockDetail; 8]> {
        let mut merged = render.details.clone();

        if let Some(model) = &render.model {
            for detail in &model.details {
                merged.push(self.model_detail_to_legacy(doc, model, detail));
            }

            for instance in &model.instances {
                merged.push(self.model_instance_to_legacy(doc, model, instance));
            }
        }

        self.compile_details(doc, &merged)
    }

    fn model_detail_to_legacy(
        &mut self,
        doc: &RawDocument<BlockDef>,
        model: &BlockProceduralModelDef,
        detail: &BlockModelDetailDef,
    ) -> BlockDetailDef {
        let seed = if detail.seed != 0 {
            detail.seed
        } else if model.seed != 0 {
            stable_hash32(&format!(
                "{}:{}:{}",
                model.seed, detail.color, detail.density
            ))
        } else {
            stable_hash32(&format!(
                "{}:{}:{}",
                doc.relative_path.display(),
                detail.color,
                detail.density
            ))
        };

        BlockDetailDef {
            kind: legacy_detail_kind(detail.kind),
            color: self.resolve_model_color(doc, model, &detail.color, "#FFFFFF80"),
            density: detail.density,
            min_size: detail.size.min,
            max_size: detail.size.max,
            slope_bias: detail.slope_bias,
            faces: if detail.faces.is_empty() {
                vec![BlockDetailFace::All]
            } else {
                detail.faces.clone()
            },
            seed,
        }
    }

    fn model_instance_to_legacy(
        &mut self,
        doc: &RawDocument<BlockDef>,
        model: &BlockProceduralModelDef,
        instance: &BlockInstanceDef,
    ) -> BlockDetailDef {
        let color_role = if !instance.color.is_empty() {
            instance.color.as_str()
        } else {
            instance.colors.first().map(String::as_str).unwrap_or("")
        };

        let seed = if instance.seed != 0 {
            instance.seed
        } else if model.seed != 0 {
            stable_hash32(&format!(
                "{}:{}:{}",
                model.seed, color_role, instance.density
            ))
        } else {
            stable_hash32(&format!(
                "{}:{}:{}",
                doc.relative_path.display(),
                color_role,
                instance.density
            ))
        };

        BlockDetailDef {
            kind: legacy_instance_kind(instance.kind),
            color: self.resolve_model_color(doc, model, color_role, "#FFFFFF80"),
            density: instance.density,
            min_size: instance.size.min,
            max_size: instance.size.max,
            slope_bias: 0.5,
            faces: if instance.faces.is_empty() {
                vec![BlockDetailFace::All]
            } else {
                instance.faces.clone()
            },
            seed,
        }
    }

    fn resolve_model_color(
        &self,
        doc: &RawDocument<BlockDef>,
        model: &BlockProceduralModelDef,
        role_or_hex: &str,
        fallback: &str,
    ) -> HexColor {
        if role_or_hex.starts_with('#') {
            return HexColor(role_or_hex.to_owned());
        }

        if role_or_hex.is_empty() {
            return HexColor(fallback.to_owned());
        }

        if let Some(color) = model.palette.colors.get(role_or_hex) {
            return color.clone();
        }

        self.warn_value(
            doc,
            "render.model.palette.colors",
            role_or_hex,
            "unknown model color role, using fallback magenta-tinted debug color",
        );

        HexColor("#FF00FFAA".to_owned())
    }
}

fn legacy_detail_kind(kind: BlockModelDetailKind) -> BlockDetailKind {
    match kind {
        BlockModelDetailKind::Pebbles => BlockDetailKind::Pebble,
        BlockModelDetailKind::Roots => BlockDetailKind::Root,
        BlockModelDetailKind::LeafLobes => BlockDetailKind::LeafLobe,
        BlockModelDetailKind::Grain => BlockDetailKind::Grain,
        BlockModelDetailKind::Speckles => BlockDetailKind::Speckle,
        BlockModelDetailKind::Stains => BlockDetailKind::Stain,
        BlockModelDetailKind::Cracks => BlockDetailKind::Crack,
        BlockModelDetailKind::Flowers => BlockDetailKind::LeafLobe,
        BlockModelDetailKind::Crystals => BlockDetailKind::Pebble,
        BlockModelDetailKind::Spikes => BlockDetailKind::Pebble,
        BlockModelDetailKind::Droplets => BlockDetailKind::Stain,
    }
}

fn legacy_instance_kind(kind: BlockInstanceKind) -> BlockDetailKind {
    match kind {
        BlockInstanceKind::LeafCard => BlockDetailKind::LeafLobe,
        BlockInstanceKind::FlowerCard => BlockDetailKind::LeafLobe,
        BlockInstanceKind::CrystalPrism => BlockDetailKind::Pebble,
        BlockInstanceKind::SpikeCone => BlockDetailKind::Pebble,
        BlockInstanceKind::PebbleBlob => BlockDetailKind::Pebble,
        BlockInstanceKind::RootCurve => BlockDetailKind::Root,
        BlockInstanceKind::DropletBlob => BlockDetailKind::Stain,
    }
}

use super::helpers::*;
use super::prelude::*;

impl super::ContentCompiler {
    pub(super) fn validate_universes(
        &mut self,
        load_order: &PackLoadOrder,
        index: &ReferenceIndex,
    ) {
        for pack in load_order.packs() {
            for doc in &pack.content.universes {
                let reference = doc.value.default_planet_type.0.clone();

                self.resolve_key(
                    "universe",
                    &doc.source_path,
                    &reference,
                    ReferenceKind::PlanetType,
                    &index.planet_types,
                );
            }
        }
    }

    pub(super) fn validate_climate(&mut self, load_order: &PackLoadOrder, index: &ReferenceIndex) {
        for pack in load_order.packs() {
            for doc in &pack.content.climate_tags {
                let tags = doc
                    .value
                    .temperature
                    .iter()
                    .map(|range| &range.tag)
                    .chain(doc.value.humidity.iter().map(|range| &range.tag))
                    .chain(doc.value.altitude.iter().map(|range| &range.tag))
                    .chain(doc.value.slope.iter().map(|range| &range.tag))
                    .chain(doc.value.latitude.iter().map(|range| &range.tag))
                    .chain(doc.value.depth.iter().map(|range| &range.tag))
                    .chain(
                        doc.value
                            .derived
                            .iter()
                            .flat_map(|rule| rule.requires.iter()),
                    )
                    .chain(
                        doc.value
                            .derived
                            .iter()
                            .flat_map(|rule| rule.produces.iter()),
                    );

                for tag in tags {
                    self.resolve_tag("climate", doc, tag, index);
                }
            }

            for doc in &pack.content.climate_transitions {
                for transition in &doc.value.transitions {
                    self.resolve_tag("climate_transition", doc, &transition.from, index);
                    self.resolve_tag("climate_transition", doc, &transition.to, index);
                }
            }
        }
    }

    pub(super) fn validate_block_render(&mut self, doc: &RawDocument<BlockDef>) {
        let render = &doc.value.render;

        if let BlockShape::Custom { model } = &render.shape.kind {
            self.parse_resource_ref("block", doc, model, ReferenceKind::Resource);
        }

        if matches!(render.shape.kind, BlockShape::Cross) && render.meshing.occludes {
            self.invalid_value(
                doc,
                "render.meshing.occludes",
                "true",
                "cross-shaped blocks should not fully occlude neighboring faces",
            );
        }

        if matches!(doc.value.physics.phase, MaterialPhase::Liquid)
            && matches!(render.meshing.render_mode, RenderMode::Opaque)
        {
            self.invalid_value(
                doc,
                "render.meshing.render_mode",
                "opaque",
                "liquid blocks should usually use transparent or additive rendering",
            );
        }

        self.warn_suspicious_base_color(doc);
    }

    fn warn_suspicious_base_color(&self, doc: &RawDocument<BlockDef>) {
        let render = &doc.value.render;
        let raw = &render.material.base_color.0;
        let Some(rgba) = parse_hex_color(raw) else {
            return;
        };

        let r = rgba[0];
        let g = rgba[1];
        let b = rgba[2];
        let is_emissive =
            render.lighting.emits_light > 0 || render.lighting.emission.is_some();

        // Pure white (or near-white) base on a non-emissive block usually means
        // the author forgot to author the hue and is relying on tint or palette
        // alone. Fine for emissive blocks (lava, crystals) where bright is the point.
        if !is_emissive && r.min(g).min(b) >= 0.94 {
            self.warn_value(
                doc,
                "render.material.base_color",
                raw,
                "near-white base color on a non-emissive block — author the hue \
                 explicitly instead of relying on tint or palette to color it",
            );
        }

        // Excessive saturation (HSV-style: max - min over max). Above 0.92
        // tends to look fluorescent under the neutral lighting pipeline.
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        if max > 0.05 {
            let saturation = (max - min) / max;
            if saturation > 0.92 && !is_emissive {
                self.warn_value(
                    doc,
                    "render.material.base_color",
                    raw,
                    "highly saturated base color — desaturate slightly to read \
                     naturally under neutral lighting",
                );
            }
        }
    }
}

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
    }
}

use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn compile_tag(
        &mut self,
        doc: &RawDocument<TagDef>,
        index: &ReferenceIndex,
    ) -> CompiledTag {
        let domain = match doc.value.kind {
            TagContentKind::Block => TagDomain::Block,
            TagContentKind::Item => TagDomain::Item,
            TagContentKind::Entity => TagDomain::Entity,
            TagContentKind::Placeable => TagDomain::Placeable,
            TagContentKind::Any => TagDomain::Any,
        };
        let values = doc
            .value
            .values
            .iter()
            .filter_map(|value| self.resolve_tagged_content("tag", doc, &value.0, domain, index))
            .collect();
        let extends = self.resolve_tags("tag", doc, &doc.value.extends, index);
        CompiledTag {
            domain,
            values,
            extends,
        }
    }
}

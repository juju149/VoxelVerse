use super::helpers::*;
use super::prelude::*;

use super::*;

impl ContentCompiler {
    pub(super) fn resolve_tags<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        tags: &[TagRef],
        index: &ReferenceIndex,
    ) -> Vec<TagId> {
        tags.iter()
            .filter_map(|tag| self.resolve_tag(owner, doc, tag, index))
            .collect()
    }

    pub(super) fn resolve_block<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &BlockRef,
        index: &ReferenceIndex,
    ) -> Option<BlockId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Block,
            &index.blocks,
        )
    }

    pub(super) fn resolve_item<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &ItemRef,
        index: &ReferenceIndex,
    ) -> Option<ItemId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Item,
            &index.items,
        )
    }

    pub(super) fn resolve_entity<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &EntityRef,
        index: &ReferenceIndex,
    ) -> Option<EntityId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Entity,
            &index.entities,
        )
    }

    pub(super) fn resolve_placeable<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &PlaceableRef,
        index: &ReferenceIndex,
    ) -> Option<PlaceableId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Placeable,
            &index.placeables,
        )
    }

    pub(super) fn resolve_loot_table<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &LootTableRef,
        index: &ReferenceIndex,
    ) -> Option<LootTableId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::LootTable,
            &index.loot_tables,
        )
    }

    pub(super) fn resolve_planet_type<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &vv_schema::worldgen::planet::PlanetTypeRef,
        index: &ReferenceIndex,
    ) -> Option<PlanetTypeId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::PlanetType,
            &index.planet_types,
        )
    }

    pub(super) fn resolve_tag<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &TagRef,
        index: &ReferenceIndex,
    ) -> Option<TagId> {
        self.resolve_key(
            owner,
            &doc.source_path,
            &reference.0,
            ReferenceKind::Tag,
            &index.tags,
        )
    }

    pub(super) fn resolve_texture_ref<T>(
        &mut self,
        doc: &RawDocument<T>,
        reference: Option<&ResourceRef>,
        texture_ids: &HashMap<ContentKey, TextureId>,
    ) -> Option<TextureId> {
        let reference = reference?;
        let key = self.parse_texture_ref("block", doc, reference)?;
        texture_ids.get(&key).copied().or_else(|| {
            self.diagnostics.push(CompileDiagnostic::MissingReference {
                owner: "block".to_owned(),
                path: doc.source_path.clone(),
                reference: reference.0.clone(),
                expected: ReferenceKind::Texture,
            });
            None
        })
    }

    pub(super) fn parse_texture_ref<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &ResourceRef,
    ) -> Option<ContentKey> {
        match ContentKey::from_str(&reference.0) {
            Ok(key) => Some(key),
            Err(err) => {
                self.diagnostics.push(CompileDiagnostic::InvalidReference {
                    owner: owner.to_owned(),
                    path: doc.source_path.clone(),
                    reference: reference.0.clone(),
                    expected: ReferenceKind::Texture,
                    reason: err.to_string(),
                });
                None
            }
        }
    }

    pub(super) fn parse_resource_ref<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &ResourceRef,
        expected: ReferenceKind,
    ) -> Option<ContentKey> {
        match ContentKey::from_str(&reference.0) {
            Ok(key) => Some(key),
            Err(err) => {
                self.diagnostics.push(CompileDiagnostic::InvalidReference {
                    owner: owner.to_owned(),
                    path: doc.source_path.clone(),
                    reference: reference.0.clone(),
                    expected,
                    reason: err.to_string(),
                });
                None
            }
        }
    }

    pub(super) fn parse_content_ref<T>(
        &mut self,
        doc: &RawDocument<T>,
        field: &str,
        reference: &str,
    ) -> Option<ContentKey> {
        match ContentKey::from_str(reference) {
            Ok(key) => Some(key),
            Err(err) => {
                self.diagnostics.push(CompileDiagnostic::InvalidValue {
                    owner: "block".to_owned(),
                    path: doc.source_path.clone(),
                    field: field.to_owned(),
                    value: reference.to_owned(),
                    reason: err.to_string(),
                });
                None
            }
        }
    }

    pub(super) fn material_key<T>(
        &mut self,
        doc: &RawDocument<T>,
        material_name: &str,
    ) -> ContentKey {
        if material_name.contains(':') {
            match ContentKey::from_str(material_name) {
                Ok(key) => key,
                Err(err) => {
                    self.diagnostics.push(CompileDiagnostic::InvalidReference {
                        owner: "block".to_owned(),
                        path: doc.source_path.clone(),
                        reference: material_name.to_owned(),
                        expected: ReferenceKind::Material,
                        reason: err.to_string(),
                    });
                    ContentKey::new(&doc.pack_namespace, "standard_opaque")
                        .expect("fallback material key is valid")
                }
            }
        } else {
            ContentKey::new(&doc.pack_namespace, material_name).unwrap_or_else(|err| {
                self.diagnostics.push(CompileDiagnostic::InvalidReference {
                    owner: "block".to_owned(),
                    path: doc.source_path.clone(),
                    reference: material_name.to_owned(),
                    expected: ReferenceKind::Material,
                    reason: err.to_string(),
                });
                ContentKey::new(&doc.pack_namespace, "standard_opaque")
                    .expect("fallback material key is valid")
            })
        }
    }

    pub(super) fn resolve_key<I>(
        &mut self,
        owner: &str,
        path: &std::path::Path,
        reference: &str,
        kind: ReferenceKind,
        index: &HashMap<ContentKey, I>,
    ) -> Option<I>
    where
        I: Copy,
    {
        match ContentKey::from_str(reference) {
            Ok(key) => index.get(&key).copied().or_else(|| {
                self.diagnostics.push(CompileDiagnostic::MissingReference {
                    owner: owner.to_owned(),
                    path: path.to_path_buf(),
                    reference: reference.to_owned(),
                    expected: kind,
                });
                None
            }),
            Err(err) => {
                self.diagnostics.push(CompileDiagnostic::InvalidReference {
                    owner: owner.to_owned(),
                    path: path.to_path_buf(),
                    reference: reference.to_owned(),
                    expected: kind,
                    reason: err.to_string(),
                });
                None
            }
        }
    }

    pub(super) fn resolve_tagged_content<T>(
        &mut self,
        owner: &str,
        doc: &RawDocument<T>,
        reference: &str,
        domain: TagDomain,
        index: &ReferenceIndex,
    ) -> Option<TaggedContent> {
        match domain {
            TagDomain::Block => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Block,
                    &index.blocks,
                )
                .map(TaggedContent::Block),
            TagDomain::Item => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Item,
                    &index.items,
                )
                .map(TaggedContent::Item),
            TagDomain::Entity => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Entity,
                    &index.entities,
                )
                .map(TaggedContent::Entity),
            TagDomain::Placeable => self
                .resolve_key(
                    owner,
                    &doc.source_path,
                    reference,
                    ReferenceKind::Placeable,
                    &index.placeables,
                )
                .map(TaggedContent::Placeable),
            TagDomain::Any => {
                let key = match ContentKey::from_str(reference) {
                    Ok(key) => key,
                    Err(err) => {
                        self.diagnostics.push(CompileDiagnostic::InvalidReference {
                            owner: owner.to_owned(),
                            path: doc.source_path.clone(),
                            reference: reference.to_owned(),
                            expected: ReferenceKind::Tag,
                            reason: err.to_string(),
                        });
                        return None;
                    }
                };
                index
                    .blocks
                    .get(&key)
                    .copied()
                    .map(TaggedContent::Block)
                    .or_else(|| index.items.get(&key).copied().map(TaggedContent::Item))
                    .or_else(|| index.entities.get(&key).copied().map(TaggedContent::Entity))
                    .or_else(|| {
                        index
                            .placeables
                            .get(&key)
                            .copied()
                            .map(TaggedContent::Placeable)
                    })
                    .or_else(|| {
                        self.diagnostics.push(CompileDiagnostic::MissingReference {
                            owner: owner.to_owned(),
                            path: doc.source_path.clone(),
                            reference: reference.to_owned(),
                            expected: ReferenceKind::Tag,
                        });
                        None
                    })
            }
        }
    }
}

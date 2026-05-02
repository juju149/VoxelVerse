use super::helpers::*;
use super::prelude::*;

impl super::ContentCompiler {
    pub(super) fn compile_block(
        &mut self,
        doc: &RawDocument<BlockDef>,
        index: &ReferenceIndex,
        _texture_ids: &HashMap<ContentKey, TextureId>,
        visual_id: BlockVisualId,
    ) -> CompiledBlock {
        self.validate_drop_spec("block", doc, &doc.value.drops, index);
        self.validate_block_render(doc);

        let render = &doc.value.render;
        let base_color = self.block_base_color(doc);

        let roughness =
            self.clamp_unit(doc, "render.material.roughness", render.material.roughness);
        let metallic = self.clamp_unit(doc, "render.material.metallic", render.material.metallic);
        let alpha = self.clamp_unit(doc, "render.material.alpha", render.material.alpha);

        let emission = render
            .lighting
            .emission
            .as_ref()
            .map(|color| self.parse_hex_color(doc, "render.lighting.emission", color));

        let material = self.compile_compiled_block_visual(doc);

        CompiledBlock {
            display_key: doc.value.display_key.as_ref().map(|key| key.0.clone()),
            stack_max: doc.value.stack_max,
            tags: self.resolve_tags("block", doc, &doc.value.tags, index),

            mining: CompiledBlockMining {
                hardness: doc.value.mining.hardness,
                tool: compiled_tool_kind(doc.value.mining.tool),
                tool_tier_min: doc.value.mining.tool_tier_min,
                drop_xp: doc.value.mining.drop_xp,
            },

            physics: CompiledBlockPhysics {
                phase: match doc.value.physics.phase {
                    MaterialPhase::Solid => CompiledMaterialPhase::Solid,
                    MaterialPhase::Liquid => CompiledMaterialPhase::Liquid,
                    MaterialPhase::Passable => CompiledMaterialPhase::Passable,
                },
                density: doc.value.physics.density,
                friction: doc.value.physics.friction,
                drag: doc.value.physics.drag,
            },

            render: CompiledBlockRender {
                visual_id,
                color: [base_color[0], base_color[1], base_color[2]],
                roughness,
                metallic,
                emission,
                alpha,
                render_mode: compiled_render_mode(&render.meshing.render_mode),
                emits_light: render.lighting.emits_light,
                tint: compiled_tint_mode(&render.material.tint),
                shape: compiled_block_shape(&render.shape.kind),

                meshing: CompiledBlockMeshing {
                    occludes: render.meshing.occludes,
                    greedy_merge: render.meshing.greedy_merge,
                    casts_shadow: render.meshing.casts_shadow,
                    receives_ao: render.meshing.receives_ao,
                },

                material,
                texture_layout: CompiledTextureLayout::Single,
                textures: CompiledBlockTextures::default(),
            },

            drops: self.compile_drop_spec("block", doc, &doc.value.drops, index),
        }
    }

    pub(super) fn compile_texture_registry(
        &mut self,
        _block_docs: &[(ContentKey, &RawDocument<BlockDef>)],
        _content: &mut CompiledContent,
    ) -> HashMap<ContentKey, TextureId> {
        HashMap::new()
    }

    pub(super) fn compile_block_visual(
        &mut self,
        block_key: &ContentKey,
        doc: &RawDocument<BlockDef>,
        content: &mut CompiledContent,
        material_ids: &mut HashMap<ContentKey, MaterialId>,
    ) -> BlockVisualId {
        let compiled = self.compile_compiled_block_visual(doc);
        let material_id = self.material_id(doc, &compiled.material_key, content, material_ids);

        let palette_offset = content.block_visual_palettes.len() as u32;
        let mut palette = compiled.palette.clone();

        if palette.is_empty() {
            palette.push(compiled.base_color);
        }

        content
            .block_visual_palettes
            .extend(palette.iter().copied());

        let palette_len = palette.len() as u32;

        let flags = BlockVisualFlags::from_bits(
            u32::from(!matches!(
                doc.value.render.meshing.render_mode,
                RenderMode::Opaque
            )) * BlockVisualFlags::TRANSPARENT
                | u32::from(
                    compiled.emission.is_some() || doc.value.render.lighting.emits_light > 0,
                ) * BlockVisualFlags::EMISSIVE
                | u32::from(compiled.variation.biome_tint_strength > 0.0)
                    * BlockVisualFlags::BIOME_TINTED
                | u32::from(doc.value.render.meshing.occludes) * BlockVisualFlags::OCCLUDES
                | u32::from(doc.value.render.meshing.receives_ao) * BlockVisualFlags::RECEIVES_AO,
        );

        let geometry_profile_id = geometry_profile_id(&doc.value.render.shape.profile) as f32;
        let surface_program = compiled_surface_program(&doc.value.render.program);

        let face_depth = self.clamp_unit(
            doc,
            "render.shape.face_depth",
            doc.value.render.shape.face_depth,
        );

        let roundness = self.clamp_unit(
            doc,
            "render.shape.roundness",
            doc.value.render.shape.roundness,
        );

        let faces = self.runtime_faces(doc, &compiled);
        let details = [RuntimeBlockDetail::default(); BLOCK_VISUAL_DETAIL_COUNT];

        let runtime = RuntimeBlockVisual {
            base_color: compiled.base_color,
            emission: compiled.emission.unwrap_or([0.0, 0.0, 0.0, 0.0]),

            surface: [
                compiled.roughness,
                compiled.metallic,
                compiled.alpha,
                face_depth,
            ],

            shape: [
                compiled.bevel,
                compiled.normal_strength,
                geometry_profile_id,
                roundness,
            ],

            variation_a: [
                compiled.variation.per_voxel_tint,
                compiled.variation.per_face_tint,
                compiled.variation.macro_noise_scale,
                compiled.variation.macro_noise_strength,
            ],

            variation_b: [
                compiled.variation.micro_noise_scale,
                compiled.variation.micro_noise_strength,
                compiled.variation.edge_darkening,
                compiled.variation.ao_influence,
            ],

            response: [
                compiled.variation.biome_tint_strength,
                compiled.variation.wetness_response,
                compiled.variation.snow_response,
                compiled.variation.dust_response,
            ],

            palette: [palette_offset, palette_len, material_id.raw(), flags.0],

            // Layout actuel conservé pour éviter de casser le shader :
            // x = grid_size legacy
            // y = face_blend legacy
            // z = detail_count
            // w = surface_program_id
            procedural: [10, 0, 0, surface_program.runtime_id()],

            faces,
            details,
        };

        content.block_visuals.push(block_key.clone(), runtime)
    }

    pub(super) fn material_id(
        &mut self,
        doc: &RawDocument<BlockDef>,
        key: &ContentKey,
        content: &mut CompiledContent,
        material_ids: &mut HashMap<ContentKey, MaterialId>,
    ) -> MaterialId {
        if let Some(id) = material_ids.get(key) {
            return *id;
        }

        let id = content.materials.push(
            key.clone(),
            CompiledMaterialShader {
                shader_key: key.clone(),
            },
        );

        material_ids.insert(key.clone(), id);

        if !is_known_material_kind(key.name()) {
            self.diagnostics.push(CompileDiagnostic::InvalidReference {
                owner: "block".to_owned(),
                path: doc.source_path.clone(),
                reference: key.to_string(),
                expected: ReferenceKind::Material,
                reason: "unknown voxel material shader kind".to_owned(),
            });
        }

        id
    }

    pub(super) fn compile_compiled_block_visual(
        &mut self,
        doc: &RawDocument<BlockDef>,
    ) -> CompiledBlockVisual {
        let render = &doc.value.render;
        let base_color = self.block_base_color(doc);

        let emission = render
            .lighting
            .emission
            .as_ref()
            .map(|color| self.parse_hex_color(doc, "render.lighting.emission", color));

        let mut palette = SmallVec::<[[f32; 4]; 8]>::new();

        for (index, color) in render.material.palette.iter().enumerate() {
            palette.push(self.parse_hex_color(
                doc,
                &format!("render.material.palette[{index}]"),
                color,
            ));
        }

        let variation = self.compile_visual_variation(doc, render.variation, render.environment);
        let material_ref = material_ref_from_kind(&render.material.kind);

        CompiledBlockVisual {
            material_key: self.material_key(doc, &material_ref),
            base_color,
            palette,

            roughness: self.clamp_unit(doc, "render.material.roughness", render.material.roughness),

            metallic: self.clamp_unit(doc, "render.material.metallic", render.material.metallic),

            emission,

            alpha: self.clamp_unit(doc, "render.material.alpha", render.material.alpha),

            bevel: self.clamp_range(doc, "render.shape.bevel", render.shape.bevel, 0.0, 0.20),

            normal_strength: self.clamp_unit(
                doc,
                "render.shape.normal_strength",
                render.shape.normal_strength,
            ),

            variation,

            surface_program: compiled_surface_program(&render.program),

            procedural: BlockProceduralConfig::new(10, false),

            faces: CompiledBlockFaceVisuals {
                top: self.compile_face_visual(doc, "render.faces.top", render.faces.top.as_ref()),
                side: self.compile_face_visual(
                    doc,
                    "render.faces.side",
                    render.faces.side.as_ref(),
                ),
                bottom: self.compile_face_visual(
                    doc,
                    "render.faces.bottom",
                    render.faces.bottom.as_ref(),
                ),
                north: self.compile_face_visual(
                    doc,
                    "render.faces.north",
                    render.faces.north.as_ref(),
                ),
                south: self.compile_face_visual(
                    doc,
                    "render.faces.south",
                    render.faces.south.as_ref(),
                ),
                east: self.compile_face_visual(
                    doc,
                    "render.faces.east",
                    render.faces.east.as_ref(),
                ),
                west: self.compile_face_visual(
                    doc,
                    "render.faces.west",
                    render.faces.west.as_ref(),
                ),
            },

            details: SmallVec::new(),
        }
    }

    pub(super) fn compile_visual_variation(
        &mut self,
        doc: &RawDocument<BlockDef>,
        variation: RawBlockVisualVariation,
        environment: RawBlockEnvironmentResponseDef,
    ) -> CompiledBlockVisualVariation {
        CompiledBlockVisualVariation {
            per_voxel_tint: self.clamp_unit(
                doc,
                "render.variation.per_voxel_tint",
                variation.per_voxel_tint,
            ),
            per_face_tint: self.clamp_unit(
                doc,
                "render.variation.per_face_tint",
                variation.per_face_tint,
            ),
            macro_noise_scale: self.positive_scale(
                doc,
                "render.variation.macro_noise_scale",
                variation.macro_noise_scale,
            ),
            macro_noise_strength: self.clamp_unit(
                doc,
                "render.variation.macro_noise_strength",
                variation.macro_noise_strength,
            ),
            micro_noise_scale: self.positive_scale(
                doc,
                "render.variation.micro_noise_scale",
                variation.micro_noise_scale,
            ),
            micro_noise_strength: self.clamp_unit(
                doc,
                "render.variation.micro_noise_strength",
                variation.micro_noise_strength,
            ),
            edge_darkening: self.clamp_unit(
                doc,
                "render.variation.edge_darkening",
                variation.edge_darkening,
            ),
            ao_influence: self.clamp_unit(
                doc,
                "render.variation.ao_influence",
                variation.ao_influence,
            ),
            biome_tint_strength: self.clamp_unit(
                doc,
                "render.environment.biome_tint_strength",
                environment.biome_tint_strength,
            ),
            wetness_response: self.clamp_unit(
                doc,
                "render.environment.wetness_response",
                environment.wetness_response,
            ),
            snow_response: self.clamp_unit(
                doc,
                "render.environment.snow_response",
                environment.snow_response,
            ),
            dust_response: self.clamp_unit(
                doc,
                "render.environment.dust_response",
                environment.dust_response,
            ),
        }
    }

    pub(super) fn compile_procedural(
        &mut self,
        _doc: &RawDocument<BlockDef>,
    ) -> BlockProceduralConfig {
        BlockProceduralConfig::new(10, false)
    }

    pub(super) fn compile_face_visual(
        &mut self,
        doc: &RawDocument<BlockDef>,
        field: &str,
        face: Option<&RawBlockFaceVisual>,
    ) -> Option<CompiledBlockFaceVisual> {
        let face = face?;

        Some(CompiledBlockFaceVisual {
            color_bias: face
                .color_bias
                .as_ref()
                .map(|color| self.parse_hex_color(doc, &format!("{field}.color_bias"), color))
                .unwrap_or([1.0, 1.0, 1.0, 1.0]),

            detail_bias: SmallVec::new(),
        })
    }

    pub(super) fn compile_detail(&mut self, _doc: &RawDocument<BlockDef>) -> CompiledBlockDetail {
        CompiledBlockDetail {
            kind: String::new(),
            density: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            min_size: 0.0,
            max_size: 0.0,
            slope_bias: 0.0,
        }
    }

    pub(super) fn runtime_faces(
        &mut self,
        _doc: &RawDocument<BlockDef>,
        compiled: &CompiledBlockVisual,
    ) -> [RuntimeBlockFaceVisual; BLOCK_VISUAL_FACE_COUNT] {
        [
            Self::runtime_face(compiled.faces.top.as_ref()),
            Self::runtime_face(compiled.faces.bottom.as_ref()),
            Self::runtime_face(
                compiled
                    .faces
                    .north
                    .as_ref()
                    .or(compiled.faces.side.as_ref()),
            ),
            Self::runtime_face(
                compiled
                    .faces
                    .south
                    .as_ref()
                    .or(compiled.faces.side.as_ref()),
            ),
            Self::runtime_face(
                compiled
                    .faces
                    .east
                    .as_ref()
                    .or(compiled.faces.side.as_ref()),
            ),
            Self::runtime_face(
                compiled
                    .faces
                    .west
                    .as_ref()
                    .or(compiled.faces.side.as_ref()),
            ),
        ]
    }

    fn runtime_face(face: Option<&CompiledBlockFaceVisual>) -> RuntimeBlockFaceVisual {
        let Some(face) = face else {
            return RuntimeBlockFaceVisual {
                color_bias: [1.0, 1.0, 1.0, 1.0],
                detail_mask: 0,
                ..RuntimeBlockFaceVisual::default()
            };
        };

        RuntimeBlockFaceVisual {
            color_bias: face.color_bias,
            detail_mask: 0,
            ..RuntimeBlockFaceVisual::default()
        }
    }

    pub(super) fn compile_block_textures(
        &mut self,
        _doc: &RawDocument<BlockDef>,
        _texture_ids: &HashMap<ContentKey, TextureId>,
    ) -> CompiledBlockTextures {
        CompiledBlockTextures::default()
    }
}

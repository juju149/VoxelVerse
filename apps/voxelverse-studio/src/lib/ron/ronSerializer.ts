import type { BlockDef, ExportFile, MaterialFaceDef, MaterialParam, MaterialPatternLayer, PackProject } from "../../types/studio";

function q(value: string) {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function paramValue(param: MaterialParam) {
  if (typeof param.value === "string") {
    return q(param.value);
  }
  return String(param.value);
}

function serializeParam(param: MaterialParam) {
  const range = typeof param.min === "number" && typeof param.max === "number"
    ? `, min: ${param.min}, max: ${param.max}`
    : "";
  return `            Param(name: ${q(param.name)}, kind: ${param.kind}, value: ${paramValue(param)}${range})`;
}

function serializeLayer(layer: MaterialPatternLayer) {
  const color = layer.color ? `, color: ${q(layer.color)}` : "";
  const threshold = typeof layer.threshold === "number" ? `, threshold: ${layer.threshold}` : "";
  return `            PatternLayer(
                id: ${q(layer.id)},
                kind: ${layer.kind},
                blend: ${layer.blend},
                domain: ${layer.domain},
                mask: ${layer.mask},
                enabled: ${layer.enabled},
                strength: ${layer.strength},
                scale: ${layer.scale},
                contrast: ${layer.contrast},
                softness: ${layer.softness},
                warp: ${layer.warp},
                offset: (${layer.offsetX}, ${layer.offsetY})${threshold}${color},
            )`;
}

function serializeBlueprint(material: MaterialFaceDef) {
  const nodes = material.blueprint.nodes.map((node) => {
    const params = Object.entries(node.params)
      .map(([key, value]) => `${key}: ${typeof value === "string" ? q(value) : value}`)
      .join(", ");
    return `            BlueprintNode(id: ${q(node.id)}, kind: ${node.kind}, label: ${q(node.label)}, position: (${node.position.x}, ${node.position.y}), params: (${params}))`;
  }).join(",\n");
  const links = material.blueprint.links
    .map((link) => `            BlueprintLink(from: ${q(link.from)}, to: ${q(link.to)})`)
    .join(",\n");

  return `    blueprint: MaterialBlueprint(
        nodes: [
${nodes}
        ],
        links: [
${links}
        ],
    ),`;
}

export function serializeMaterialFace(material: MaterialFaceDef) {
  if (material.rawRonOverride?.trim()) {
    return material.rawRonOverride;
  }

  const params = material.recipe.params.map(serializeParam).join(",\n");
  const layers = material.recipe.patternLayers.map(serializeLayer).join(",\n");

  return `MaterialFaceDef(
    id: ${q(material.id)},
    display_name: ${q(material.displayName)},
    material_kind: ${q(material.materialKind)},
    resolution_preview: ${material.resolutionPreview},
    seed: ${material.seed},
${serializeBlueprint(material)}
    recipe: ProceduralMaterialRecipe(
        style: ${material.recipe.style},
        palette: (
            base: ${q(material.recipe.baseColor)},
            shadow: ${q(material.recipe.shadowColor)},
            highlight: ${q(material.recipe.highlightColor)},
        ),
        pattern_layers: [
${layers}
        ],
        surface: (
            roughness: ${material.recipe.surface.roughness},
            height_strength: ${material.recipe.surface.heightStrength},
            normal_strength: ${material.recipe.surface.normalStrength},
            edge_softness: ${material.recipe.surface.edgeSoftness},
        ),
        stylization: (
            color_steps: ${material.recipe.stylization.colorSteps},
            smoothing: ${material.recipe.stylization.smoothing},
            saturation: ${material.recipe.stylization.saturation},
            value_boost: ${material.recipe.stylization.valueBoost},
            micro_detail: ${material.recipe.stylization.microDetail},
        ),
        variation: (
            enabled: ${material.recipe.variation.enabled},
            per_block_strength: ${material.recipe.variation.perBlockStrength},
            color_jitter: ${material.recipe.variation.colorJitter},
            pattern_jitter: ${material.recipe.variation.patternJitter},
        ),
        exposed_params: [
${params}
        ],
    ),
)`;
}

function materialRefs(block: BlockDef) {
  const refs = block.render.materials;
  return [
    refs.all ? `        all: Some(MaterialFaceRef(${q(refs.all)})),` : "        all: None,",
    refs.top ? `        top: Some(MaterialFaceRef(${q(refs.top)})),` : "        top: None,",
    refs.side ? `        side: Some(MaterialFaceRef(${q(refs.side)})),` : "        side: None,",
    refs.bottom ? `        bottom: Some(MaterialFaceRef(${q(refs.bottom)})),` : "        bottom: None,",
    refs.north ? `        north: Some(MaterialFaceRef(${q(refs.north)})),` : "        north: None,",
    refs.south ? `        south: Some(MaterialFaceRef(${q(refs.south)})),` : "        south: None,",
    refs.east ? `        east: Some(MaterialFaceRef(${q(refs.east)})),` : "        east: None,",
    refs.west ? `        west: Some(MaterialFaceRef(${q(refs.west)})),` : "        west: None,",
  ].join("\n");
}

export function serializeBlock(block: BlockDef) {
  if (block.rawRonOverride?.trim()) {
    return block.rawRonOverride;
  }

  const tags = block.tags.map(q).join(", ");
  const drops = block.gameplay.drops.map(q).join(", ");
  const tint = block.render.tint ? `Some(${q(block.render.tint)})` : "None";

  return `BlockDef(
    id: ${q(block.id)},
    display_name: ${q(block.displayName)},
    seed: ${block.seed},
    geometry: (
        kind: ${block.geometry.kind},
        collision_shape: ${block.geometry.collisionShape},
        custom_model: ${block.geometry.customModel ? `Some(${q(block.geometry.customModel)})` : "None"},
    ),
    render: (
        materials: (
${materialRefs(block)}
        ),
        tint: ${tint},
        ambient_occlusion: ${block.render.ambientOcclusion},
        transparent: ${block.render.transparent},
        cull_faces: ${block.render.cullFaces},
        light_emission: ${block.render.lightEmission},
    ),
    gameplay: (
        walk_through: ${block.gameplay.walkThrough},
        hardness: ${block.gameplay.hardness},
        break_speed_preset: ${block.gameplay.breakSpeedPreset.replace("-", "_")},
        drops: [${drops}],
    ),
    tags: [${tags}],
)`;
}

export function serializePack(project: PackProject) {
  return `PackDef(
    schema_version: ${project.schemaVersion},
    id: ${q(project.id)},
    namespace: ${q(project.namespace)},
    display_name: ${q(project.name)},
    pack_seed: ${project.packSeed},
    source_of_truth: "procedural_ron",
    materials: ${project.materials.length},
    blocks: ${project.blocks.length},
)`;
}

function materialPath(material: MaterialFaceDef) {
  const path = material.id.split(":")[1] ?? material.id;
  return `materials/${path}.ron`;
}

function blockPath(block: BlockDef) {
  const path = block.id.split(":")[1] ?? block.id;
  return `blocks/${path}.ron`;
}

export function exportProjectFiles(project: PackProject): ExportFile[] {
  return [
    { path: "pack.ron", content: serializePack(project) },
    ...project.materials.map((material) => ({ path: materialPath(material), content: serializeMaterialFace(material) })),
    ...project.blocks.map((block) => ({ path: blockPath(block), content: serializeBlock(block) })),
  ];
}

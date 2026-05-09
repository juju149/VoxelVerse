import type { BlockDef, ExportFile, MaterialFaceDef, MaterialParam, MaterialPatternLayer, PackProject, ProceduralGraph } from "../../types/studio";

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

// ---------------------------------------------------------------------------
// ProceduralGraph serializer (Phase 2)
// ---------------------------------------------------------------------------

function serializeParamValue(value: string | number | boolean): string {
  if (typeof value === "string") return q(value);
  if (typeof value === "boolean") return String(value);
  return String(value);
}

export function serializeGraph(graph: ProceduralGraph): string {
  const nodes = graph.nodes.map((node) => {
    const paramsStr = Object.entries(node.params)
      .map(([k, v]) => `            ${k}: ${serializeParamValue(v)}`)
      .join(",\n");
    const exposedStr = node.exposedParams.map(q).join(", ");
    return `        GraphNode(
            id: ${q(node.id)},
            kind: ${node.kind},
            position: (${Math.round(node.position.x)}, ${Math.round(node.position.y)}),
            params: {
${paramsStr}
            },
            exposed_params: [${exposedStr}],
        )`;
  }).join(",\n");

  const conns = graph.connections.map((c) => `        GraphConnection(
            id: ${q(c.id)},
            from: ${q(`${c.fromNode}.${c.fromPort}`)},
            to: ${q(`${c.toNode}.${c.toPort}`)},
        )`).join(",\n");

  return `ProceduralGraph(
    version: ${graph.version},
    nodes: [
${nodes}
    ],
    connections: [
${conns}
    ],
)`;
}

// ---------------------------------------------------------------------------
// MaterialFaceDef serializer
// ---------------------------------------------------------------------------

export function serializeMaterialFace(material: MaterialFaceDef) {
  if (material.rawRonOverride?.trim()) {
    return material.rawRonOverride;
  }

  const graphSection = material.graph
    ? `\n    graph: Some(${serializeGraph(material.graph).replace(/\n/g, "\n    ")}),`
    : "\n    graph: None,";

  const params = material.recipe.params.map(serializeParam).join(",\n");
  const layers = material.recipe.patternLayers.map(serializeLayer).join(",\n");

  return `MaterialFaceDef(
    id: ${q(material.id)},
    display_name: ${q(material.displayName)},
    category: ${q(material.category)},
    resolution_preview: ${material.resolutionPreview},
    seed: ${material.seed},${graphSection}
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

/**
 * Derives a baked texture ref path from a material face ID.
 * e.g. "core:terrain/grass_top" → "core:baked/terrain/grass_top_albedo"
 */
function bakedRef(materialId: string, channel: "albedo" | "normal" | "roughness") {
  const [namespace, path] = materialId.split(":");
  if (!namespace || !path) return `${q(`${materialId}_${channel}`)}`;
  return q(`${namespace}:baked/${path}_${channel}`);
}

function visualFace(materialId: string | undefined) {
  if (!materialId) return "None";
  return `Some((
            albedo: ${bakedRef(materialId, "albedo")},
            normal: ${bakedRef(materialId, "normal")},
            roughness: ${bakedRef(materialId, "roughness")},
        ))`;
}

export function serializeBlock(block: BlockDef) {
  if (block.rawRonOverride?.trim()) {
    return block.rawRonOverride;
  }

  const refs = block.render.materials;
  // Resolve per-face material IDs ("all" overrides individual faces).
  const top = refs.top ?? refs.all;
  const bottom = refs.bottom ?? refs.all;
  const side = refs.side ?? refs.all;
  const north = refs.north ?? side;
  const south = refs.south ?? side;
  const east = refs.east ?? side;
  const west = refs.west ?? side;

  const solid = block.geometry.collisionShape === "solid_cube";
  const [cr, cg, cb] = block.color;
  const shape = block.geometry.kind === "cross_plant" ? "cross_plane" : "cube";
  const drops = block.gameplay.drops.map(q).join(", ");
  const tags = block.tags.map(q).join(", ");

  // Emit RawBlockDef — the format the engine reads directly.
  // Gameplay extras (drops, tags, seed) follow in a comment block so
  // no information is lost for the future compiler.
  return `RawBlockDef(
    display_name: ${q(block.displayName)},
    solid: ${solid},
    color: (${cr.toFixed(3)}, ${cg.toFixed(3)}, ${cb.toFixed(3)}),
    hardness: ${block.gameplay.hardness},
    visual: Some((
        shape: ${shape},
        top: ${visualFace(top)},
        bottom: ${visualFace(bottom)},
        side: ${visualFace(side)},
        north: ${visualFace(north)},
        south: ${visualFace(south)},
        east: ${visualFace(east)},
        west: ${visualFace(west)},
    )),
)
// --- Studio metadata (not parsed by the engine) ---
// id: ${q(block.id)}
// seed: ${block.seed}
// break_speed: ${block.gameplay.breakSpeedPreset}
// walk_through: ${block.gameplay.walkThrough}
// light_emission: ${block.render.lightEmission}
// drops: [${drops}]
// tags: [${tags}]
`;
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
  const graphFiles: ExportFile[] = project.materials
    .filter((m) => m.graph)
    .map((m) => ({ path: graphPath(m), content: serializeGraph(m.graph!) }));

  return [
    { path: "pack.ron", content: serializePack(project) },
    ...project.materials.map((material) => ({ path: materialPath(material), content: serializeMaterialFace(material) })),
    ...graphFiles,
    ...project.blocks.map((block) => ({ path: blockPath(block), content: serializeBlock(block) })),
  ];
}

function graphPath(material: MaterialFaceDef): string {
  const [namespace, path = "unknown"] = material.id.split(":");
  return `packs/${namespace}/graphs/${path}.ron`;
}

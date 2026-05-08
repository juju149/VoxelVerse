import type { FaceMaterialRefs, PackProject, ValidationIssue } from "../../types/studio";

const idPattern = /^[a-z0-9_]+:[a-z0-9_/-]+$/;

export function validatePack(project: PackProject): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const materialIds = new Set<string>();
  const blockIds = new Set<string>();

  for (const material of project.materials) {
    if (!idPattern.test(material.id)) {
      issues.push({
        id: `${material.id}:invalid-id`,
        severity: "error",
        message: "Material ID is invalid.",
        path: `materials/${material.id}`,
        details: "Use namespace:path, lowercase letters, numbers, underscores, dashes and slashes.",
        fixable: true,
        fixKind: "normalize-id",
        targetId: material.id,
      });
    }
    if (materialIds.has(material.id)) {
      issues.push({
        id: `${material.id}:duplicate`,
        severity: "error",
        message: "Duplicate material ID.",
        path: `materials/${material.id}`,
        fixable: false,
      });
    }
    materialIds.add(material.id);
    if (!Number.isInteger(material.seed) || material.seed < 0) {
      issues.push({
        id: `${material.id}:seed`,
        severity: "error",
        message: "Material seed must be a positive integer.",
        path: `materials/${material.id}.seed`,
        fixable: true,
        fixKind: "clamp-seed",
        targetId: material.id,
      });
    }
    if (!material.recipe.patternLayers.some((layer) => layer.enabled)) {
      issues.push({
        id: `${material.id}:no-pattern`,
        severity: "warning",
        message: "Material has no enabled pattern layer.",
        path: `materials/${material.id}.recipe.pattern_layers`,
        details: "It will render as a flat color, which is valid but less expressive.",
        fixable: false,
      });
    }
    if (!material.blueprint.nodes.some((node) => node.kind === "output")) {
      issues.push({
        id: `${material.id}:blueprint-output`,
        severity: "error",
        message: "Blueprint has no Output node.",
        path: `materials/${material.id}.blueprint`,
        details: "A material blueprint needs one output node to define the exported material.",
        fixable: false,
      });
    }
    const blueprintNodeIds = new Set(material.blueprint.nodes.map((node) => node.id));
    for (const link of material.blueprint.links) {
      const from = link.from.split(".")[0];
      const to = link.to.split(".")[0];
      if (!blueprintNodeIds.has(from) || !blueprintNodeIds.has(to)) {
        issues.push({
          id: `${material.id}:blueprint-link:${link.id}`,
          severity: "error",
          message: "Blueprint link points to a missing node.",
          path: `materials/${material.id}.blueprint.links`,
          details: `${link.from} -> ${link.to}`,
          fixable: false,
        });
      }
    }
    for (const layer of material.recipe.patternLayers) {
      if (!layer.id.trim()) {
        issues.push({
          id: `${material.id}:layer-id`,
          severity: "error",
          message: "Pattern layer has an empty ID.",
          path: `materials/${material.id}.recipe.pattern_layers`,
          fixable: false,
        });
      }
      if (layer.scale <= 0 || layer.strength < 0 || layer.strength > 1 || layer.contrast < 0 || layer.contrast > 1) {
        issues.push({
          id: `${material.id}:${layer.id}:range`,
          severity: "error",
          message: "Pattern layer values are outside valid ranges.",
          path: `materials/${material.id}.recipe.pattern_layers.${layer.id}`,
          details: "Scale must be positive. Strength and contrast must stay between 0 and 1.",
          fixable: false,
        });
      }
    }
    if (material.recipe.stylization.colorSteps < 2) {
      issues.push({
        id: `${material.id}:stylization`,
        severity: "error",
        message: "Stylization color steps must be at least 2.",
        path: `materials/${material.id}.recipe.stylization.color_steps`,
        fixable: false,
      });
    }
  }

  for (const block of project.blocks) {
    if (!idPattern.test(block.id)) {
      issues.push({
        id: `${block.id}:invalid-id`,
        severity: "error",
        message: "Block ID is invalid.",
        path: `blocks/${block.id}`,
        details: "Use namespace:path, for example core:grass.",
        fixable: true,
        fixKind: "normalize-id",
        targetId: block.id,
      });
    }
    if (blockIds.has(block.id)) {
      issues.push({
        id: `${block.id}:duplicate`,
        severity: "error",
        message: "Duplicate block ID.",
        path: `blocks/${block.id}`,
        fixable: false,
      });
    }
    blockIds.add(block.id);

    validateBlockMaterials(block.id, block.render.materials, materialIds, issues);

    if (!Number.isFinite(block.gameplay.hardness) || block.gameplay.hardness < 0 || block.gameplay.hardness > 100) {
      issues.push({
        id: `${block.id}:hardness`,
        severity: "error",
        message: "Hardness must be between 0 and 100.",
        path: `blocks/${block.id}.gameplay.hardness`,
        fixable: true,
        fixKind: "clamp-hardness",
        targetId: block.id,
      });
    }
    if (!Number.isInteger(block.seed) || block.seed < 0) {
      issues.push({
        id: `${block.id}:seed`,
        severity: "error",
        message: "Block seed must be a positive integer.",
        path: `blocks/${block.id}.seed`,
        fixable: true,
        fixKind: "clamp-seed",
        targetId: block.id,
      });
    }
    if (block.render.lightEmission < 0 || block.render.lightEmission > 15) {
      issues.push({
        id: `${block.id}:light`,
        severity: "error",
        message: "Light emission must be between 0 and 15.",
        path: `blocks/${block.id}.render.light_emission`,
        fixable: false,
      });
    }
  }

  if (project.hasUnsavedChanges) {
    issues.push({
      id: "project:unsaved",
      severity: "warning",
      message: "Project has unsaved local changes.",
      path: project.path,
      fixable: false,
    });
  }

  return issues;
}

function validateBlockMaterials(
  blockId: string,
  refs: FaceMaterialRefs,
  materialIds: Set<string>,
  issues: ValidationIssue[],
) {
  const faces: (keyof FaceMaterialRefs)[] = ["all", "top", "side", "bottom", "north", "south", "east", "west"];
  const assigned = faces.filter((face) => refs[face]);
  if (assigned.length === 0) {
    issues.push({
      id: `${blockId}:materials:empty`,
      severity: "error",
      message: "Block has no material assigned.",
      path: `blocks/${blockId}.render.materials`,
      details: "Assign All Material for simple blocks, or Top/Side/Bottom for terrain blocks.",
      fixable: true,
      fixKind: "assign-all-material",
      targetId: blockId,
    });
    return;
  }
  for (const face of assigned) {
    const ref = refs[face];
    if (ref && !materialIds.has(ref)) {
      issues.push({
        id: `${blockId}:${face}:missing`,
        severity: "error",
        message: `Block references a missing ${face} material.`,
        path: `blocks/${blockId}.render.materials.${face}`,
        details: `${ref} was not found.`,
        fixable: face === "bottom" && materialIds.has("core:dirt/base"),
        fixKind: face === "bottom" ? "use-dirt-bottom" : undefined,
        targetId: blockId,
      });
    }
  }
}

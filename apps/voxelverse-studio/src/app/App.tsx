import { useMemo, useState } from "react";
import { AppShell } from "../components/layout/AppShell";
import { MaterialsPage } from "../pages/MaterialsPage";
import { BlocksPage } from "../pages/BlocksPage";
import { initialProject } from "../data/initialProject";
import { createMaterialFromPreset } from "../lib/presets/materialPresets";
import { createBlockFromPreset } from "../lib/presets/blockPresets";
import { exportProjectFiles } from "../lib/ron/ronSerializer";
import { randomSeed } from "../lib/procedural/seed";
import { validatePack } from "../lib/validation/packValidator";
import type {
  BlockDef,
  BlockKind,
  FaceMaterialRefs,
  MaterialFaceDef,
  MaterialKind,
  MaterialStylePreset,
  PackProject,
  StudioRoute,
  ValidationIssue,
} from "../types/studio";

const storageKey = "voxelverse-studio-project-v4";

function loadProject(): PackProject {
  const saved = window.localStorage.getItem(storageKey);
  if (!saved) {
    return initialProject;
  }
  try {
    const parsed = JSON.parse(saved) as PackProject;
    if (parsed.schemaVersion !== 4 || !Array.isArray(parsed.materials) || !Array.isArray(parsed.blocks)) {
      window.localStorage.removeItem(storageKey);
      return initialProject;
    }
    return { ...parsed, validationIssues: validatePack(parsed) };
  } catch {
    return initialProject;
  }
}

function downloadFile(path: string, content: string) {
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = path.replace(/\//g, "__");
  link.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 500);
}

function markDirty(project: PackProject): PackProject {
  return { ...project, hasUnsavedChanges: true, validationIssues: validatePack({ ...project, hasUnsavedChanges: true }) };
}

export function App() {
  const [route, setRoute] = useState<StudioRoute>("materials");
  const [project, setProject] = useState(loadProject);
  const [selectedMaterialId, setSelectedMaterialId] = useState(project.materials[0]?.id ?? "");
  const [selectedBlockId, setSelectedBlockId] = useState(project.blocks[0]?.id ?? "");
  const [statusMessage, setStatusMessage] = useState("Ready");

  const selectedMaterial = useMemo(
    () => project.materials.find((material) => material.id === selectedMaterialId) ?? project.materials[0],
    [project.materials, selectedMaterialId],
  );
  const selectedBlock = useMemo(
    () => project.blocks.find((block) => block.id === selectedBlockId) ?? project.blocks[0],
    [project.blocks, selectedBlockId],
  );

  function updateProject(next: PackProject, message?: string) {
    setProject(markDirty(next));
    if (message) {
      setStatusMessage(message);
    }
  }

  function updateMaterial(material: MaterialFaceDef, message?: string) {
    updateProject({
      ...project,
      materials: project.materials.map((item) => item.id === selectedMaterialId ? material : item),
    }, message ?? "Material updated");
    if (material.id !== selectedMaterialId) {
      setSelectedMaterialId(material.id);
    }
  }

  function updateBlock(block: BlockDef, message?: string) {
    updateProject({
      ...project,
      blocks: project.blocks.map((item) => item.id === selectedBlockId ? block : item),
    }, message ?? "Block updated");
    if (block.id !== selectedBlockId) {
      setSelectedBlockId(block.id);
    }
  }

  function createMaterial(kind: MaterialKind, style: MaterialStylePreset) {
    const material = createMaterialFromPreset(kind, style, project.namespace);
    const unique = ensureUniqueMaterialId(material, project.materials);
    updateProject({ ...project, materials: [...project.materials, unique] }, "Material created");
    setSelectedMaterialId(unique.id);
  }

  function createBlock(kind: BlockKind, faces: FaceMaterialRefs) {
    const block = createBlockFromPreset(kind, faces, project.namespace);
    const unique = ensureUniqueBlockId(block, project.blocks);
    updateProject({ ...project, blocks: [...project.blocks, unique] }, "Block created");
    setSelectedBlockId(unique.id);
  }

  function saveProject() {
    const saved: PackProject = {
      ...project,
      hasUnsavedChanges: false,
      lastSavedAt: new Date().toISOString(),
    };
    saved.validationIssues = validatePack(saved);
    window.localStorage.setItem(storageKey, JSON.stringify(saved));
    setProject(saved);
    setStatusMessage("All changes saved locally");
  }

  function validateCurrentProject() {
    const issues = validatePack(project);
    setProject({ ...project, validationIssues: issues });
    setStatusMessage(issues.length === 0 ? "Pack valid" : `${issues.length} validation issue(s)`);
  }

  function exportProject() {
    const files = exportProjectFiles(project);
    for (const file of files) {
      downloadFile(file.path, file.content);
    }
    setStatusMessage(`Exported ${files.length} .ron file(s)`);
  }

  function applyFix(issue: ValidationIssue) {
    if (!issue.fixKind || !issue.targetId) {
      return;
    }
    let next = project;
    if (issue.fixKind === "use-dirt-bottom") {
      next = {
        ...project,
        blocks: project.blocks.map((block) => block.id === issue.targetId
          ? { ...block, render: { ...block.render, materials: { ...block.render.materials, bottom: "core:dirt/base" } } }
          : block),
      };
    } else if (issue.fixKind === "clamp-hardness") {
      next = {
        ...project,
        blocks: project.blocks.map((block) => block.id === issue.targetId
          ? { ...block, gameplay: { ...block.gameplay, hardness: Math.max(0, Math.min(100, block.gameplay.hardness || 0)) } }
          : block),
      };
    } else if (issue.fixKind === "clamp-seed") {
      next = {
        ...project,
        materials: project.materials.map((material) => material.id === issue.targetId
          ? { ...material, seed: Math.abs(Math.floor(material.seed || randomSeed())) }
          : material),
        blocks: project.blocks.map((block) => block.id === issue.targetId
          ? { ...block, seed: Math.abs(Math.floor(block.seed || randomSeed())) }
          : block),
      };
    } else if (issue.fixKind === "normalize-id") {
      next = normalizeId(project, issue.targetId);
    } else if (issue.fixKind === "assign-all-material") {
      next = {
        ...project,
        blocks: project.blocks.map((block) => block.id === issue.targetId
          ? { ...block, render: { ...block.render, materials: { all: project.materials[0]?.id ?? "" } } }
          : block),
      };
    }
    updateProject(next, "Fix applied");
  }

  if (!selectedMaterial || !selectedBlock) {
    return (
      <div className="grid h-screen place-items-center bg-background p-8 text-center text-sm text-muted-foreground">
        No pack data. Create a material or block to start.
      </div>
    );
  }

  return (
    <AppShell
      project={project}
      route={route}
      statusMessage={statusMessage}
      onRouteChange={setRoute}
      onSave={saveProject}
      onValidate={validateCurrentProject}
      onExport={exportProject}
    >
      {route === "materials" ? (
        <MaterialsPage
          project={project}
          selectedMaterial={selectedMaterial}
          onSelectMaterial={setSelectedMaterialId}
          onCreateMaterial={createMaterial}
          onUpdateMaterial={updateMaterial}
        />
      ) : (
        <BlocksPage
          project={project}
          selectedBlock={selectedBlock}
          onSelectBlock={setSelectedBlockId}
          onCreateBlock={createBlock}
          onUpdateBlock={updateBlock}
          onFixIssue={applyFix}
        />
      )}
    </AppShell>
  );
}

function ensureUniqueMaterialId(material: MaterialFaceDef, materials: MaterialFaceDef[]) {
  const ids = new Set(materials.map((item) => item.id));
  if (!ids.has(material.id)) {
    return material;
  }
  const [namespace, path] = material.id.split(":");
  let index = 2;
  while (ids.has(`${namespace}:${path}_${index}`)) {
    index += 1;
  }
  return { ...material, id: `${namespace}:${path}_${index}`, displayName: `${material.displayName} ${index}` };
}

function ensureUniqueBlockId(block: BlockDef, blocks: BlockDef[]) {
  const ids = new Set(blocks.map((item) => item.id));
  if (!ids.has(block.id)) {
    return block;
  }
  const [namespace, path] = block.id.split(":");
  let index = 2;
  while (ids.has(`${namespace}:${path}_${index}`)) {
    index += 1;
  }
  return { ...block, id: `${namespace}:${path}_${index}`, displayName: `${block.displayName} ${index}` };
}

function normalizeId(project: PackProject, targetId: string) {
  const normalize = (value: string) => {
    const [namespaceRaw, pathRaw = namespaceRaw] = value.includes(":") ? value.split(":") : [project.namespace, value];
    const clean = pathRaw.toLowerCase().replace(/[^a-z0-9_/-]+/g, "_").replace(/^_+|_+$/g, "");
    return `${namespaceRaw.toLowerCase().replace(/[^a-z0-9_]+/g, "_")}:${clean || "item"}`;
  };
  return {
    ...project,
    materials: project.materials.map((material) => material.id === targetId ? { ...material, id: normalize(material.id) } : material),
    blocks: project.blocks.map((block) => block.id === targetId ? { ...block, id: normalize(block.id) } : block),
  };
}

import { useState } from "react";
import type { BlockDef, BlockKind, FaceMaterialRefs, PackProject, ValidationIssue } from "../types/studio";
import { BlockBuilder } from "../components/blocks/BlockBuilder";
import { BlockCubePreview } from "../components/blocks/BlockCubePreview";
import { BlockLibrary } from "../components/blocks/BlockLibrary";
import { BlockVariationPreview } from "../components/blocks/BlockVariationPreview";
import { BlockWizard } from "../components/blocks/BlockWizard";
import { ValidationPanel } from "../components/validation/ValidationPanel";

type BlocksPageProps = {
  project: PackProject;
  selectedBlock: BlockDef;
  onSelectBlock: (id: string) => void;
  onCreateBlock: (kind: BlockKind, faces: FaceMaterialRefs) => void;
  onUpdateBlock: (block: BlockDef, message?: string) => void;
  onFixIssue: (issue: ValidationIssue) => void;
};

export function BlocksPage({
  project,
  selectedBlock,
  onSelectBlock,
  onCreateBlock,
  onUpdateBlock,
  onFixIssue,
}: BlocksPageProps) {
  const [wizardOpen, setWizardOpen] = useState(false);
  const [variationOffset, setVariationOffset] = useState(0);

  return (
    <div className="grid h-full grid-cols-[300px_minmax(460px,1fr)_420px] gap-4 p-4">
      <BlockLibrary
        blocks={project.blocks}
        selectedId={selectedBlock.id}
        onSelect={onSelectBlock}
        onNew={() => setWizardOpen(true)}
        onQuickCreate={() => onCreateBlock("cube", defaultFaces(project))}
      />

      <div className="min-h-0 overflow-auto">
        <BlockBuilder
          block={selectedBlock}
          materials={project.materials}
          onChange={onUpdateBlock}
        />
      </div>

      <div className="min-h-0 space-y-4 overflow-auto">
        <BlockCubePreview
          project={project}
          block={selectedBlock}
          variationOffset={variationOffset}
          onNextVariation={() => setVariationOffset((value) => value + 9)}
        />
        <BlockVariationPreview project={project} block={selectedBlock} offset={variationOffset} />
        <ValidationPanel issues={project.validationIssues} onFix={onFixIssue} />
      </div>

      <BlockWizard
        open={wizardOpen}
        materials={project.materials}
        onClose={() => setWizardOpen(false)}
        onCreate={(kind, faces) => {
          onCreateBlock(kind, faces);
          setWizardOpen(false);
        }}
      />
    </div>
  );
}

function defaultFaces(project: PackProject): FaceMaterialRefs {
  const has = (id: string) => project.materials.some((material) => material.id === id);
  return {
    top: has("core:grass/top") ? "core:grass/top" : project.materials[0]?.id ?? "",
    side: has("core:grass/side") ? "core:grass/side" : project.materials[0]?.id ?? "",
    bottom: has("core:dirt/base") ? "core:dirt/base" : project.materials[0]?.id ?? "",
  };
}

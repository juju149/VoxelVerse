import { useState } from "react";
import type { MaterialFaceDef, MaterialKind, MaterialStylePreset, PackProject } from "../types/studio";
import { MaterialBlueprintEditor } from "../components/materials/MaterialBlueprintEditor";
import { MaterialLibrary } from "../components/materials/MaterialLibrary";
import { MaterialPreview } from "../components/materials/MaterialPreview";
import { MaterialRonEditor } from "../components/materials/MaterialRonEditor";
import { MaterialWizard } from "../components/materials/MaterialWizard";
import { Button } from "../components/ui/button";

type MaterialsPageProps = {
  project: PackProject;
  selectedMaterial: MaterialFaceDef;
  onSelectMaterial: (id: string) => void;
  onCreateMaterial: (kind: MaterialKind, style: MaterialStylePreset) => void;
  onUpdateMaterial: (material: MaterialFaceDef, message?: string) => void;
};

type Tab = "look" | "ron";

export function MaterialsPage({
  project,
  selectedMaterial,
  onSelectMaterial,
  onCreateMaterial,
  onUpdateMaterial,
}: MaterialsPageProps) {
  const [wizardOpen, setWizardOpen] = useState(false);
  const [tab, setTab] = useState<Tab>("look");

  return (
    <div className="grid h-full grid-cols-[300px_minmax(680px,1fr)_360px] gap-4 p-4">
      <MaterialLibrary
        materials={project.materials}
        selectedId={selectedMaterial.id}
        onSelect={onSelectMaterial}
        onNew={() => setWizardOpen(true)}
        onQuickCreate={(kind) => onCreateMaterial(kind, "soft_natural")}
      />

      <div className="min-h-0 overflow-hidden rounded-lg border bg-card">
        <div className="flex items-center justify-between gap-4 border-b p-3">
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold">{selectedMaterial.displayName}</div>
            <div className="truncate text-xs text-muted-foreground">{selectedMaterial.id} procedural material recipe</div>
          </div>
          <div className="flex w-56 shrink-0 gap-2">
          {(["look", "ron"] as const).map((item) => (
            <Button
              key={item}
              variant={tab === item ? "secondary" : "ghost"}
              size="sm"
              className="flex-1 capitalize"
              onClick={() => setTab(item)}
            >
              {item === "ron" ? "RON" : "Look"}
            </Button>
          ))}
          </div>
        </div>
        <div className="h-[calc(100%-65px)] overflow-auto p-5">
          {tab === "look" ? (
            <MaterialBlueprintEditor material={selectedMaterial} onChange={onUpdateMaterial} />
          ) : (
            <MaterialRonEditor material={selectedMaterial} onChange={onUpdateMaterial} />
          )}
        </div>
      </div>

      <div className="min-h-0 overflow-auto">
        <MaterialPreview
          material={selectedMaterial}
          onGenerate={() => onUpdateMaterial({
            ...selectedMaterial,
            previewVersion: selectedMaterial.previewVersion + 1,
          }, "Preview generated")}
          onRandomizeSeed={() => onUpdateMaterial({
            ...selectedMaterial,
            seed: Math.floor(100 + Math.random() * 999999),
            previewVersion: selectedMaterial.previewVersion + 1,
          }, "Seed randomized")}
          onSave={() => onUpdateMaterial(selectedMaterial, "Material saved to local project")}
        />
      </div>

      <MaterialWizard
        open={wizardOpen}
        onClose={() => setWizardOpen(false)}
        onCreate={(kind, style) => {
          onCreateMaterial(kind, style);
          setWizardOpen(false);
        }}
      />
    </div>
  );
}

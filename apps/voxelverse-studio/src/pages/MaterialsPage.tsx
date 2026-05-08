import { useState } from "react";
import type { MaterialFaceDef, PackProject } from "../types/studio";
import { MaterialBlueprintEditor } from "../components/materials/MaterialBlueprintEditor";
import { MaterialLibrary } from "../components/materials/MaterialLibrary";
import { MaterialPreview } from "../components/materials/MaterialPreview";
import { MaterialRonEditor } from "../components/materials/MaterialRonEditor";
import { MaterialTemplateGallery } from "../components/materials/MaterialTemplateGallery";
import { Button } from "../components/ui/button";

type MaterialsPageProps = {
  project: PackProject;
  selectedMaterial: MaterialFaceDef;
  onSelectMaterial: (id: string) => void;
  /** Create a blank material with no pattern layers. */
  onCreateEmpty: () => void;
  /** Create a material from a template key. */
  onCreateFromTemplate: (templateKey: string) => void;
  onUpdateMaterial: (material: MaterialFaceDef, message?: string) => void;
};

type Tab = "look" | "ron";

export function MaterialsPage({
  project,
  selectedMaterial,
  onSelectMaterial,
  onCreateEmpty,
  onCreateFromTemplate,
  onUpdateMaterial,
}: MaterialsPageProps) {
  const [templateGalleryOpen, setTemplateGalleryOpen] = useState(false);
  const [tab, setTab] = useState<Tab>("look");

  return (
    <div className="grid h-full grid-cols-[300px_minmax(680px,1fr)_360px] gap-4 p-4">
      <MaterialLibrary
        materials={project.materials}
        selectedId={selectedMaterial.id}
        onSelect={onSelectMaterial}
        onNew={onCreateEmpty}
        onNewFromTemplate={() => setTemplateGalleryOpen(true)}
      />

      <div className="min-h-0 overflow-hidden rounded-lg border bg-card">
        <div className="flex items-center justify-between gap-4 border-b p-3">
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold">{selectedMaterial.displayName}</div>
            <div className="truncate text-xs text-muted-foreground">
              {selectedMaterial.id} · {selectedMaterial.category}
            </div>
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
            <MaterialBlueprintEditor
              material={selectedMaterial}
              onChange={onUpdateMaterial}
              initialMode={selectedMaterial.category === "custom" && selectedMaterial.recipe.patternLayers.length === 0 ? "simple" : "advanced"}
            />
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

      <MaterialTemplateGallery
        open={templateGalleryOpen}
        onClose={() => setTemplateGalleryOpen(false)}
        onSelect={(templateKey) => {
          onCreateFromTemplate(templateKey);
          setTemplateGalleryOpen(false);
        }}
      />
    </div>
  );
}

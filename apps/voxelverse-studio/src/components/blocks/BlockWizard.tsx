import { useMemo, useState } from "react";
import type { BlockKind, FaceMaterialRefs, MaterialFaceDef } from "../../types/studio";
import { blockTemplates } from "../../lib/presets/blockPresets";
import { Button } from "../ui/button";
import { Dialog } from "../ui/dialog";
import { Label } from "../ui/label";
import { Select } from "../ui/select";

type BlockWizardProps = {
  open: boolean;
  materials: MaterialFaceDef[];
  onClose: () => void;
  onCreate: (kind: BlockKind, faces: FaceMaterialRefs) => void;
};

export function BlockWizard({ open, materials, onClose, onCreate }: BlockWizardProps) {
  const [kind, setKind] = useState<BlockKind>("cube");
  const defaults = useMemo(() => defaultFaces(materials), [materials]);
  const [faces, setFaces] = useState<FaceMaterialRefs>(defaults);

  return (
    <Dialog open={open} title="New Block" onClose={onClose}>
      <div className="space-y-5">
        <section className="space-y-3">
          <h3 className="text-sm font-medium">Choose block type</h3>
          <div className="grid grid-cols-2 gap-2">
            {blockTemplates.map((template) => (
              <button
                key={template.kind}
                type="button"
                onClick={() => setKind(template.kind)}
                className={`rounded-md border p-3 text-left text-sm ${kind === template.kind ? "border-primary bg-primary/10" : "bg-background/55"}`}
              >
                <span className="block font-medium">{template.label}</span>
                <span className="text-xs text-muted-foreground">{template.description}</span>
              </button>
            ))}
          </div>
        </section>

        <section className="grid grid-cols-3 gap-3">
          {(["top", "side", "bottom"] as const).map((face) => (
            <div key={face} className="space-y-2">
              <Label>{face}</Label>
              <Select value={faces[face]} onChange={(event) => setFaces({ ...faces, [face]: event.target.value })}>
                {materials.map((material) => <option key={material.id} value={material.id}>{material.displayName}</option>)}
              </Select>
            </div>
          ))}
        </section>

        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>Cancel</Button>
          <Button onClick={() => onCreate(kind, faces)}>Create</Button>
        </div>
      </div>
    </Dialog>
  );
}

function defaultFaces(materials: MaterialFaceDef[]): FaceMaterialRefs {
  const has = (id: string) => materials.some((material) => material.id === id);
  return {
    top: has("core:grass/top") ? "core:grass/top" : materials[0]?.id ?? "",
    side: has("core:grass/side") ? "core:grass/side" : materials[0]?.id ?? "",
    bottom: has("core:dirt/base") ? "core:dirt/base" : materials[0]?.id ?? "",
  };
}

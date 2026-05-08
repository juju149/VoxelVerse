import { useState } from "react";
import type { MaterialKind, MaterialStylePreset } from "../../types/studio";
import { materialPresetChoices, materialStyles } from "../../lib/presets/materialPresets";
import { Button } from "../ui/button";
import { Dialog } from "../ui/dialog";

type MaterialWizardProps = {
  open: boolean;
  onClose: () => void;
  onCreate: (kind: MaterialKind, style: MaterialStylePreset) => void;
};

export function MaterialWizard({ open, onClose, onCreate }: MaterialWizardProps) {
  const [kind, setKind] = useState<MaterialKind>("grass_top");
  const [style, setStyle] = useState<MaterialStylePreset>("soft_natural");

  return (
    <Dialog open={open} title="New Material" onClose={onClose}>
      <div className="space-y-5">
        <section className="space-y-3">
          <h3 className="text-sm font-medium">Choose material type</h3>
          <div className="grid grid-cols-2 gap-2">
            {materialPresetChoices.map((choice) => (
              <button
                key={choice.kind}
                type="button"
                onClick={() => setKind(choice.kind)}
                className={`rounded-md border p-3 text-left text-sm ${kind === choice.kind ? "border-primary bg-primary/10" : "bg-background/55"}`}
              >
                <span className="block font-medium">{choice.label}</span>
                <span className="text-xs text-muted-foreground">{choice.description}</span>
              </button>
            ))}
          </div>
        </section>

        <section className="space-y-3">
          <h3 className="text-sm font-medium">Choose style preset</h3>
          <div className="grid grid-cols-4 gap-2">
            {materialStyles.map((item) => (
              <button
                key={item.id}
                type="button"
                onClick={() => setStyle(item.id)}
                className={`rounded-md border px-3 py-2 text-sm ${style === item.id ? "border-primary bg-primary/10" : "bg-background/55"}`}
              >
                {item.label}
              </button>
            ))}
          </div>
        </section>

        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>Cancel</Button>
          <Button onClick={() => onCreate(kind, style)}>Create</Button>
        </div>
      </div>
    </Dialog>
  );
}

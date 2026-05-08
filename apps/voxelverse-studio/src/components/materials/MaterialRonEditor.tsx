import { useEffect, useState } from "react";
import type { MaterialFaceDef } from "../../types/studio";
import { normalizeRonEdit } from "../../lib/ron/ronParser";
import { serializeMaterialFace } from "../../lib/ron/ronSerializer";
import { Button } from "../ui/button";

type MaterialRonEditorProps = {
  material: MaterialFaceDef;
  onChange: (material: MaterialFaceDef, message?: string) => void;
};

export function MaterialRonEditor({ material, onChange }: MaterialRonEditorProps) {
  const generated = serializeMaterialFace({ ...material, rawRonOverride: undefined });
  const [value, setValue] = useState(material.rawRonOverride ?? generated);

  useEffect(() => {
    setValue(material.rawRonOverride ?? generated);
  }, [generated, material.rawRonOverride]);

  return (
    <div className="space-y-3" title="The raw modding file saved in the pack.">
      <textarea
        className="h-[520px] w-full resize-none rounded-md border bg-background p-3 font-mono text-xs leading-relaxed text-foreground outline-none focus:ring-2 focus:ring-ring"
        value={value}
        onChange={(event) => setValue(event.target.value)}
        spellCheck={false}
      />
      <div className="flex gap-2">
        <Button onClick={() => onChange({ ...material, rawRonOverride: normalizeRonEdit(value) }, "RON override applied to export")}>
          Apply to Export
        </Button>
        <Button variant="secondary" onClick={() => onChange({ ...material, rawRonOverride: undefined }, "RON regenerated from material recipe")}>
          Regenerate
        </Button>
      </div>
    </div>
  );
}

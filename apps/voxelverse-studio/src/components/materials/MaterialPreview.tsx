import { useMemo } from "react";
import { Dices, Save, WandSparkles } from "lucide-react";
import type { MaterialFaceDef } from "../../types/studio";
import { generateMaterialPreviewDataUrl } from "../../lib/procedural/evaluator";
import { Button } from "../ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";

type MaterialPreviewProps = {
  material: MaterialFaceDef;
  onGenerate: () => void;
  onRandomizeSeed: () => void;
  onSave: () => void;
};

export function MaterialPreview({ material, onGenerate, onRandomizeSeed, onSave }: MaterialPreviewProps) {
  const preview = useMemo(
    () => generateMaterialPreviewDataUrl(material, { seedOverride: material.seed + material.previewVersion }),
    [material],
  );

  return (
    <Card>
      <CardHeader>
        <div className="space-y-2">
          <div className="flex items-center justify-between gap-3">
            <CardTitle>Preview</CardTitle>
            <div className="rounded-md border bg-muted px-2 py-1 text-xs text-muted-foreground" title="Controls the random look. Same seed = same result.">
              Seed {material.seed}
            </div>
          </div>
          <CardDescription>Live result from the RON recipe.</CardDescription>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-lg border bg-background/55 p-3">
          <img className="aspect-square w-full rounded-md border object-cover" src={preview} alt={`${material.displayName} face preview`} />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <div className="mb-2 text-xs font-medium text-muted-foreground">4x4 repeat</div>
            <div
              className="aspect-square rounded-md border"
              style={{ backgroundImage: `url(${preview})`, backgroundSize: "25% 25%" }}
            />
          </div>
          <div>
            <div className="mb-2 text-xs font-medium text-muted-foreground">Cube check</div>
            <div className="grid aspect-square place-items-center overflow-hidden rounded-md border bg-background/50">
              <div className="cube-preview-compact">
                <div className="cube-face cube-face-top" style={{ backgroundImage: `url(${preview})`, backgroundSize: "cover" }} />
                <div className="cube-face cube-face-side" style={{ backgroundImage: `url(${preview})`, backgroundSize: "cover" }} />
                <div className="cube-face cube-face-front" style={{ backgroundImage: `url(${preview})`, backgroundSize: "cover" }} />
              </div>
            </div>
          </div>
        </div>

        <div className="grid gap-2">
          <Button className="w-full" onClick={onGenerate}>
            <WandSparkles className="h-4 w-4" />
            Generate Preview
          </Button>
          <Button className="w-full" variant="secondary" onClick={onRandomizeSeed}>
            <Dices className="h-4 w-4" />
            Randomize Seed
          </Button>
          <Button className="w-full" variant="outline" onClick={onSave}>
            <Save className="h-4 w-4" />
            Save Material
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

import type { BlockDef, PackProject } from "../../types/studio";
import { facePreview } from "./BlockCubePreview";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";

type BlockVariationPreviewProps = {
  project: PackProject;
  block: BlockDef;
  offset: number;
};

export function BlockVariationPreview({ project, block, offset }: BlockVariationPreviewProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>9 Seeded Variations</CardTitle>
        <CardDescription>Same block, deterministic preview seeds.</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-3">
          {Array.from({ length: 9 }, (_, index) => {
            const top = facePreview(project, block, "top", offset + index);
            const side = facePreview(project, block, "side", offset + index);
            return (
              <div key={index} className="grid aspect-square place-items-center rounded-md border bg-background/55">
                <div className="cube-preview-compact">
                  <div className="cube-face cube-face-top" style={{ backgroundImage: `url(${top})`, backgroundSize: "cover" }} />
                  <div className="cube-face cube-face-side" style={{ backgroundImage: `url(${side})`, backgroundSize: "cover" }} />
                  <div className="cube-face cube-face-front" style={{ backgroundImage: `url(${side})`, backgroundSize: "cover" }} />
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}

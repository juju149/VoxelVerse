import { Dices } from "lucide-react";
import type { BlockDef, PackProject } from "../../types/studio";
import { generateMaterialPreviewDataUrl } from "../../lib/procedural/evaluator";
import { finalMaterialSeed } from "../../lib/procedural/seed";
import { Button } from "../ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";

type BlockCubePreviewProps = {
  project: PackProject;
  block: BlockDef;
  variationOffset: number;
  onNextVariation: () => void;
};

export function BlockCubePreview({ project, block, variationOffset, onNextVariation }: BlockCubePreviewProps) {
  const top = facePreview(project, block, "top", variationOffset);
  const side = facePreview(project, block, "side", variationOffset);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Live Cube Preview</CardTitle>
        <CardDescription>{block.id}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid place-items-center rounded-lg border bg-background/55 py-8">
          <div className="cube-preview">
            <div className="cube-face cube-face-top" style={{ backgroundImage: `url(${top})`, backgroundSize: "cover" }} />
            <div className="cube-face cube-face-side" style={{ backgroundImage: `url(${side})`, backgroundSize: "cover" }} />
            <div className="cube-face cube-face-front" style={{ backgroundImage: `url(${side})`, backgroundSize: "cover" }} />
          </div>
        </div>
        <Button variant="secondary" onClick={onNextVariation}>
          <Dices className="h-4 w-4" />
          Show Variation
        </Button>
      </CardContent>
    </Card>
  );
}

export function facePreview(project: PackProject, block: BlockDef, face: keyof BlockDef["render"]["materials"], offset: number) {
  const refs = block.render.materials;
  const materialId = refs[face] ?? (face === "top" ? refs.all : refs.side ?? refs.all) ?? refs.all;
  const material = project.materials.find((item) => item.id === materialId) ?? project.materials[0];
  if (!material) {
    return "";
  }
  const seed = finalMaterialSeed(project.packSeed, block.id, block.seed + offset, material.id, face, project.seedPolicy.previewPositionSeed + offset);
  return generateMaterialPreviewDataUrl(material, { seedOverride: seed, size: 96 });
}

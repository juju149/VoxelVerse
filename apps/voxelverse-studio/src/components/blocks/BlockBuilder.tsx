import { useEffect, useState } from "react";
import type { BlockDef, FaceMaterialRefs, MaterialFaceDef } from "../../types/studio";
import { normalizeRonEdit } from "../../lib/ron/ronParser";
import { serializeBlock } from "../../lib/ron/ronSerializer";
import { Button } from "../ui/button";
import { Collapsible } from "../ui/collapsible";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Select } from "../ui/select";
import { Slider } from "../ui/slider";
import { Switch } from "../ui/switch";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";

function rgbToHex(color: [number, number, number]): string {
  const ch = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255).toString(16).padStart(2, "0");
  return `#${ch(color[0])}${ch(color[1])}${ch(color[2])}`;
}

function hexToRgb01(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.replace("#", ""), 16);
  return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

type BlockBuilderProps = {
  block: BlockDef;
  materials: MaterialFaceDef[];
  onChange: (block: BlockDef, message?: string) => void;
};

export function BlockBuilder({ block, materials, onChange }: BlockBuilderProps) {
  const generatedRon = serializeBlock({ ...block, rawRonOverride: undefined });
  const [rawRon, setRawRon] = useState(block.rawRonOverride ?? generatedRon);

  useEffect(() => {
    setRawRon(block.rawRonOverride ?? generatedRon);
  }, [block.rawRonOverride, generatedRon]);

  function update(patch: Partial<BlockDef>, message = "Block updated") {
    onChange({ ...block, ...patch, rawRonOverride: undefined }, message);
  }

  function setMaterial(face: keyof FaceMaterialRefs, value: string) {
    update({
      render: {
        ...block.render,
        materials: { ...block.render.materials, [face]: value || undefined },
      },
    }, "Material assigned");
  }

  function useSimpleMaterial(value: string) {
    update({
      render: {
        ...block.render,
        materials: value
          ? { all: value }
          : { top: materials[0]?.id ?? "", side: materials[0]?.id ?? "", bottom: materials[0]?.id ?? "" },
      },
    }, value ? "Single material applied" : "Face slots enabled");
  }

  return (
    <Card className="min-h-full">
      <CardHeader>
        <CardTitle>Block Builder</CardTitle>
        <CardDescription>Pick a shape, assign materials, export a complete block .ron.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="grid grid-cols-2 gap-4">
          <Field label="Name">
            <Input value={block.displayName} onChange={(event) => update({ displayName: event.target.value })} />
          </Field>
          <Field label="ID">
            <Input value={block.id} onChange={(event) => update({ id: event.target.value })} />
          </Field>
          <Field label="Block shape">
            <Select
              value={block.geometry.kind}
              onChange={(event) => update({
                geometry: { ...block.geometry, kind: event.target.value as BlockDef["geometry"]["kind"] },
              })}
            >
              <option value="cube">Full Block</option>
              <option value="slab">Slab</option>
              <option value="stairs">Stairs</option>
              <option value="cross_plant">Plant</option>
              <option value="liquid">Liquid</option>
              <option value="custom">Custom</option>
            </Select>
          </Field>
          <Field label="Category">
            <Input value={block.category} onChange={(event) => update({ category: event.target.value })} />
          </Field>
        </div>

        <section className="space-y-3 rounded-lg border bg-background/45 p-3">
          <div>
            <h3 className="text-sm font-medium">Materials</h3>
            <p className="text-xs text-muted-foreground">Use All for simple blocks, or override top/side/bottom when needed.</p>
          </div>
          <Field label="All Material">
            <Select value={block.render.materials.all ?? ""} onChange={(event) => useSimpleMaterial(event.target.value)}>
              <option value="">Use face slots</option>
              {materials.map((material) => <option key={material.id} value={material.id}>{material.displayName}</option>)}
            </Select>
          </Field>
          <div className="grid grid-cols-3 gap-3">
            {(["top", "side", "bottom"] as const).map((face) => (
              <Field key={face} label={`${face.charAt(0).toUpperCase()}${face.slice(1)}`}>
                <Select
                  value={block.render.materials[face] ?? ""}
                  onChange={(event) => setMaterial(face, event.target.value)}
                  disabled={Boolean(block.render.materials.all)}
                >
                  <option value="">Not set</option>
                  {materials.map((material) => <option key={material.id} value={material.id}>{material.displayName}</option>)}
                </Select>
              </Field>
            ))}
          </div>
        </section>

        <div className="grid grid-cols-2 gap-4">
          <Field label="Can walk through?">
            <Switch
              checked={block.gameplay.walkThrough}
              onCheckedChange={(walkThrough) => update({ gameplay: { ...block.gameplay, walkThrough } })}
            />
          </Field>
          <Field label="Break Speed">
            <Select
              value={block.gameplay.breakSpeedPreset}
              onChange={(event) => update({ gameplay: { ...block.gameplay, breakSpeedPreset: event.target.value as BlockDef["gameplay"]["breakSpeedPreset"] } })}
            >
              <option value="soft">Soft</option>
              <option value="normal">Normal</option>
              <option value="hard">Hard</option>
              <option value="very-hard">Very Hard</option>
            </Select>
          </Field>
          <Field label="Drops">
            <Input
              value={block.gameplay.drops.join(", ")}
              onChange={(event) => update({ gameplay: { ...block.gameplay, drops: event.target.value.split(",").map((item) => item.trim()).filter(Boolean) } })}
            />
          </Field>
          <Field label="Tags">
            <Input value={block.tags.join(", ")} onChange={(event) => update({ tags: event.target.value.split(",").map((item) => item.trim()).filter(Boolean) })} />
          </Field>
        </div>

        <Collapsible title="Advanced block .ron fields">
          <div className="space-y-4">
            <Field label={`Exact hardness ${block.gameplay.hardness.toFixed(2)}`}>
              <Slider
                min={0}
                max={10}
                step={0.05}
                value={block.gameplay.hardness}
                onChange={(event) => update({ gameplay: { ...block.gameplay, hardness: Number(event.currentTarget.value) } })}
              />
            </Field>
            <div className="grid grid-cols-2 gap-3">
              <Field label="Collision">
                <Select
                  value={block.geometry.collisionShape}
                  onChange={(event) => update({ geometry: { ...block.geometry, collisionShape: event.target.value as BlockDef["geometry"]["collisionShape"] } })}
                >
                  <option value="solid_cube">Solid cube</option>
                  <option value="partial">Partial</option>
                  <option value="cross">Cross</option>
                  <option value="fluid">Fluid</option>
                  <option value="none">None</option>
                </Select>
              </Field>
              <Field label="Light emission">
                <Input
                  type="number"
                  min={0}
                  max={15}
                  value={block.render.lightEmission}
                  onChange={(event) => update({ render: { ...block.render, lightEmission: Number(event.target.value) } })}
                />
              </Field>
            </div>
            <Switch
              checked={block.render.transparent}
              onCheckedChange={(transparent) => update({ render: { ...block.render, transparent } })}
              label="Transparent"
            />
            <Switch
              checked={block.render.ambientOcclusion}
              onCheckedChange={(ambientOcclusion) => update({ render: { ...block.render, ambientOcclusion } })}
              label="Ambient occlusion"
            />
            <Field label="Block seed">
              <Input type="number" value={block.seed} onChange={(event) => update({ seed: Number(event.target.value) })} />
            </Field>
            <Field label="LOD Color (RGB 0–1)">
              <div className="flex gap-2">
                <Input
                  type="color"
                  className="w-14 p-1"
                  value={rgbToHex(block.color)}
                  onChange={(event) => update({ color: hexToRgb01(event.target.value) })}
                />
                <Input
                  value={block.color.map((v) => v.toFixed(3)).join(", ")}
                  readOnly
                  className="font-mono text-xs"
                />
              </div>
              <p className="mt-1 text-xs text-muted-foreground">Used for distant LOD rendering. Click the swatch to change.</p>
            </Field>
            <Field label="Raw .ron">
              <textarea
                className="h-52 w-full resize-none rounded-md border bg-background p-3 font-mono text-xs outline-none focus:ring-2 focus:ring-ring"
                value={rawRon}
                onChange={(event) => setRawRon(event.target.value)}
                spellCheck={false}
              />
            </Field>
            <div className="flex gap-2">
              <Button onClick={() => onChange({ ...block, rawRonOverride: normalizeRonEdit(rawRon) }, "Block RON override applied")}>
                Apply RON to Export
              </Button>
              <Button variant="secondary" onClick={() => onChange({ ...block, rawRonOverride: undefined }, "Block RON regenerated")}>
                Regenerate RON
              </Button>
            </div>
          </div>
        </Collapsible>
      </CardContent>
    </Card>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2">
      <Label>{label}</Label>
      {children}
    </div>
  );
}

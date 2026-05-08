import { useCallback, useEffect, useMemo, useState } from "react";
import {
  addEdge,
  Background,
  Controls,
  Handle,
  MarkerType,
  MiniMap,
  Position,
  ReactFlow,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { Plus, SlidersHorizontal, Trash2 } from "lucide-react";
import type { MaterialBlueprintNode, MaterialBlueprintNodeKind, MaterialFaceDef, ParamValue } from "../../types/studio";
import { compileRecipeFromBlueprint, createBlueprintNode } from "../../lib/blueprint/materialBlueprint";
import { cn } from "../../lib/cn";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Select } from "../ui/select";
import { Slider } from "../ui/slider";
import { Switch } from "../ui/switch";

type MaterialBlueprintEditorProps = {
  material: MaterialFaceDef;
  onChange: (material: MaterialFaceDef, message?: string) => void;
};

type BlueprintNodeData = {
  label: string;
  kind: MaterialBlueprintNodeKind;
};

const nodeTypes = { materialNode: MaterialNode };

export function MaterialBlueprintEditor({ material, onChange }: MaterialBlueprintEditorProps) {
  const initialNodes = useMemo(() => toFlowNodes(material.blueprint.nodes), [material.blueprint.nodes]);
  const initialEdges = useMemo(() => toFlowEdges(material.blueprint.links), [material.blueprint.links]);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [selectedId, setSelectedId] = useState(material.blueprint.nodes[0]?.id ?? "");
  const [menu, setMenu] = useState<{ x: number; y: number; flowX: number; flowY: number } | null>(null);

  useEffect(() => setNodes(toFlowNodes(material.blueprint.nodes)), [material.blueprint.nodes, setNodes]);
  useEffect(() => setEdges(toFlowEdges(material.blueprint.links)), [material.blueprint.links, setEdges]);

  const selected = material.blueprint.nodes.find((node) => node.id === selectedId);

  const commitBlueprint = useCallback((nextNodes: MaterialBlueprintNode[], nextEdges = material.blueprint.links, message = "Blueprint updated") => {
    const blueprint = { nodes: nextNodes, links: nextEdges };
    const recipe = compileRecipeFromBlueprint(blueprint, material.recipe);
    onChange({ ...material, blueprint, recipe, rawRonOverride: undefined, previewVersion: material.previewVersion + 1 }, message);
  }, [material, onChange]);

  const onConnect = useCallback((connection: Connection) => {
    const edgeId = `${connection.source}-${connection.target}-${Date.now()}`;
    const nextEdge: Edge = {
      ...connection,
      id: edgeId,
      animated: true,
      markerEnd: { type: MarkerType.ArrowClosed },
      style: { stroke: "#8b5cf6", strokeWidth: 2 },
    };
    setEdges((current) => addEdge(nextEdge, current));
    commitBlueprint(material.blueprint.nodes, [
      ...material.blueprint.links,
      { id: edgeId, from: `${connection.source}.out`, to: `${connection.target}.in` },
    ], "Blueprint link added");
  }, [commitBlueprint, material.blueprint.links, material.blueprint.nodes, setEdges]);

  function updateNode(node: MaterialBlueprintNode) {
    commitBlueprint(material.blueprint.nodes.map((item) => item.id === node.id ? node : item), material.blueprint.links, "Node parameters updated");
  }

  function addNode(kind: MaterialBlueprintNodeKind) {
    if (!menu) {
      return;
    }
    const node = createBlueprintNode(kind, menu.flowX, menu.flowY, material.blueprint.nodes.length + 1);
    commitBlueprint([...material.blueprint.nodes, node], [
      ...material.blueprint.links,
      { id: `${node.id}-output`, from: `${node.id}.out`, to: "output.in" },
    ], "Blueprint node added");
    setSelectedId(node.id);
    setMenu(null);
  }

  function deleteSelectedNode() {
    if (!selected || selected.kind === "output") {
      return;
    }
    commitBlueprint(
      material.blueprint.nodes.filter((node) => node.id !== selected.id),
      material.blueprint.links.filter((link) => !link.from.startsWith(`${selected.id}.`) && !link.to.startsWith(`${selected.id}.`)),
      "Blueprint node removed",
    );
    setSelectedId("output");
  }

  return (
    <div className="grid h-full min-h-[720px] grid-cols-[minmax(0,1fr)_320px] overflow-hidden rounded-md border bg-[#0b0d12]">
      <div className="relative min-h-0">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={(changes) => {
            onNodesChange(changes);
          }}
          onNodeDragStop={(_, node) => {
            commitBlueprint(material.blueprint.nodes.map((item) => item.id === node.id ? { ...item, position: node.position } : item), material.blueprint.links, "Blueprint node moved");
          }}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={(_, node) => setSelectedId(node.id)}
          onNodeContextMenu={(event, node) => {
            event.preventDefault();
            setSelectedId(node.id);
          }}
          onPaneContextMenu={(event) => {
            event.preventDefault();
            const target = event.currentTarget as HTMLElement;
            const bounds = target.getBoundingClientRect();
            setMenu({ x: event.clientX - bounds.left, y: event.clientY - bounds.top, flowX: event.clientX - bounds.left, flowY: event.clientY - bounds.top });
          }}
          fitView
          panOnDrag
          zoomOnScroll
          selectionOnDrag
          proOptions={{ hideAttribution: true }}
        >
          <Background color="#252a34" gap={24} size={1} />
          <MiniMap pannable zoomable nodeColor={(node) => nodeColor(String(node.data.kind))} />
          <Controls />
        </ReactFlow>
        {menu ? (
          <div className="absolute z-20 w-56 rounded-lg border bg-card p-2 shadow-2xl" style={{ left: menu.x, top: menu.y }}>
            <div className="mb-2 px-2 py-1 text-xs font-medium text-muted-foreground">Add node</div>
            {(["pattern", "palette", "stylization", "surface", "variation", "output"] as const).map((kind) => (
              <button
                key={kind}
                type="button"
                className="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-sm hover:bg-muted"
                onClick={() => addNode(kind)}
              >
                <Plus className="h-4 w-4 text-primary" />
                {nodeLabel(kind)}
              </button>
            ))}
          </div>
        ) : null}
      </div>

      <aside className="min-h-0 overflow-auto border-l bg-card/95 p-4">
        <div className="mb-4 flex items-center justify-between gap-3">
          <div>
            <div className="text-sm font-semibold">Node Parameters</div>
            <div className="text-xs text-muted-foreground">Right-click canvas to add nodes.</div>
          </div>
          <Button variant="ghost" size="icon" onClick={deleteSelectedNode} disabled={!selected || selected.kind === "output"}>
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
        {selected ? (
          <NodeInspector node={selected} onChange={updateNode} />
        ) : (
          <div className="rounded-md border border-dashed p-4 text-sm text-muted-foreground">Select a node to edit its parameters.</div>
        )}
      </aside>
    </div>
  );
}

function MaterialNode({ data, selected }: NodeProps<Node<BlueprintNodeData>>) {
  return (
    <div className={cn(
      "min-w-[190px] overflow-hidden rounded-md border bg-[#11151c] shadow-xl",
      selected ? "border-amber-400 shadow-amber-500/20" : "border-slate-700",
    )}>
      <div className="flex items-center gap-2 px-3 py-2 text-xs font-semibold text-white" style={{ backgroundColor: nodeColor(data.kind) }}>
        <SlidersHorizontal className="h-3.5 w-3.5" />
        {data.label}
      </div>
      <div className="space-y-2 p-3 text-xs text-slate-300">
        <div className="flex justify-between gap-3">
          <span>In</span>
          <span>Out</span>
        </div>
      </div>
      <Handle type="target" position={Position.Left} id="in" className="!h-3 !w-3 !border-slate-900 !bg-cyan-400" />
      <Handle type="source" position={Position.Right} id="out" className="!h-3 !w-3 !border-slate-900 !bg-cyan-400" />
    </div>
  );
}

function NodeInspector({ node, onChange }: { node: MaterialBlueprintNode; onChange: (node: MaterialBlueprintNode) => void }) {
  function setParam(name: string, value: ParamValue) {
    onChange({ ...node, params: { ...node.params, [name]: value } });
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>Node name</Label>
        <Input value={node.label} onChange={(event) => onChange({ ...node, label: event.target.value })} />
      </div>
      {node.kind === "pattern" ? <PatternParams node={node} setParam={setParam} /> : null}
      {node.kind === "palette" ? (
        <>
          <ColorParam label="Base" value={String(node.params.baseColor ?? "#7BAA32")} onChange={(value) => setParam("baseColor", value)} />
          <ColorParam label="Shadow" value={String(node.params.shadowColor ?? "#5F8D29")} onChange={(value) => setParam("shadowColor", value)} />
          <ColorParam label="Highlight" value={String(node.params.highlightColor ?? "#9ACB4E")} onChange={(value) => setParam("highlightColor", value)} />
        </>
      ) : null}
      {node.kind === "stylization" ? (
        <>
          <NumberParam label="Color steps" value={Number(node.params.colorSteps ?? 5)} min={2} max={10} step={1} onChange={(value) => setParam("colorSteps", value)} />
          <NumberParam label="Smoothing" value={Number(node.params.smoothing ?? 0.48)} min={0} max={1} step={0.01} onChange={(value) => setParam("smoothing", value)} />
          <NumberParam label="Saturation" value={Number(node.params.saturation ?? 1)} min={0} max={1.6} step={0.01} onChange={(value) => setParam("saturation", value)} />
          <NumberParam label="Value boost" value={Number(node.params.valueBoost ?? 1)} min={0.5} max={1.5} step={0.01} onChange={(value) => setParam("valueBoost", value)} />
          <NumberParam label="Micro detail" value={Number(node.params.microDetail ?? 0.08)} min={0} max={0.25} step={0.01} onChange={(value) => setParam("microDetail", value)} />
        </>
      ) : null}
      {node.kind === "surface" ? (
        <>
          <NumberParam label="Roughness" value={Number(node.params.roughness ?? 0.78)} min={0} max={1} step={0.01} onChange={(value) => setParam("roughness", value)} />
          <NumberParam label="Height" value={Number(node.params.heightStrength ?? 0.14)} min={0} max={1} step={0.01} onChange={(value) => setParam("heightStrength", value)} />
          <NumberParam label="Normal" value={Number(node.params.normalStrength ?? 0.22)} min={0} max={1} step={0.01} onChange={(value) => setParam("normalStrength", value)} />
          <NumberParam label="Edge softness" value={Number(node.params.edgeSoftness ?? 0.42)} min={0} max={1} step={0.01} onChange={(value) => setParam("edgeSoftness", value)} />
        </>
      ) : null}
      {node.kind === "variation" ? (
        <>
          <Switch checked={Boolean(node.params.enabled ?? true)} onCheckedChange={(checked) => setParam("enabled", checked)} label="Enabled" />
          <NumberParam label="Per block" value={Number(node.params.perBlockStrength ?? 0.18)} min={0} max={0.5} step={0.01} onChange={(value) => setParam("perBlockStrength", value)} />
          <NumberParam label="Color jitter" value={Number(node.params.colorJitter ?? 0.08)} min={0} max={0.25} step={0.01} onChange={(value) => setParam("colorJitter", value)} />
          <NumberParam label="Pattern jitter" value={Number(node.params.patternJitter ?? 0.12)} min={0} max={0.4} step={0.01} onChange={(value) => setParam("patternJitter", value)} />
        </>
      ) : null}
      {node.kind === "output" ? (
        <div className="rounded-md border bg-background/60 p-3 text-sm text-muted-foreground">Output collects every connected procedural node and drives the exported RON.</div>
      ) : null}
    </div>
  );
}

function PatternParams({ node, setParam }: { node: MaterialBlueprintNode; setParam: (name: string, value: ParamValue) => void }) {
  return (
    <>
      <Switch checked={Boolean(node.params.enabled ?? true)} onCheckedChange={(checked) => setParam("enabled", checked)} label="Enabled" />
      <SelectParam label="Pattern" value={String(node.params.kind ?? "soft_blotches")} onChange={(value) => setParam("kind", value)} options={[
        "soft_noise", "soft_blotches", "organic_cells", "rounded_pebbles", "edge_band", "patch_cells", "rings", "stripes", "dots", "bands", "cracks", "flat",
      ]} />
      <SelectParam label="Blend" value={String(node.params.blend ?? "overlay")} onChange={(value) => setParam("blend", value)} options={[
        "mix", "overlay", "multiply", "screen", "shadow", "highlight", "add", "subtract",
      ]} />
      <SelectParam label="Domain" value={String(node.params.domain ?? "warped_uv")} onChange={(value) => setParam("domain", value)} options={[
        "uv", "warped_uv", "radial", "vertical", "horizontal",
      ]} />
      <SelectParam label="Mask" value={String(node.params.mask ?? "none")} onChange={(value) => setParam("mask", value)} options={[
        "none", "top_band", "bottom_band", "vertical_gradient", "center_soft", "edge_wear",
      ]} />
      <NumberParam label="Strength" value={Number(node.params.strength ?? 0.2)} min={0} max={1} step={0.01} onChange={(value) => setParam("strength", value)} />
      <NumberParam label="Scale" value={Number(node.params.scale ?? 6)} min={1} max={24} step={0.25} onChange={(value) => setParam("scale", value)} />
      <NumberParam label="Contrast" value={Number(node.params.contrast ?? 0.18)} min={0} max={1} step={0.01} onChange={(value) => setParam("contrast", value)} />
      <NumberParam label="Softness" value={Number(node.params.softness ?? 0.42)} min={0} max={1} step={0.01} onChange={(value) => setParam("softness", value)} />
      <NumberParam label="Warp" value={Number(node.params.warp ?? 0.12)} min={0} max={1} step={0.01} onChange={(value) => setParam("warp", value)} />
      <NumberParam label="Threshold" value={Number(node.params.threshold ?? 0.5)} min={0} max={1} step={0.01} onChange={(value) => setParam("threshold", value)} />
      <ColorParam label="Layer color" value={String(node.params.color ?? "#9ACB4E")} onChange={(value) => setParam("color", value)} />
    </>
  );
}

function NumberParam({ label, value, min, max, step, onChange }: { label: string; value: number; min: number; max: number; step: number; onChange: (value: number) => void }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-2">
        <Label>{label}</Label>
        <span className="text-xs text-muted-foreground">{value.toFixed(step >= 1 ? 0 : 2)}</span>
      </div>
      <Slider min={min} max={max} step={step} value={value} onChange={(event) => onChange(Number(event.currentTarget.value))} />
    </div>
  );
}

function SelectParam({ label, value, options, onChange }: { label: string; value: string; options: string[]; onChange: (value: string) => void }) {
  return (
    <div className="space-y-2">
      <Label>{label}</Label>
      <Select value={value} onChange={(event) => onChange(event.target.value)}>
        {options.map((option) => <option key={option} value={option}>{option}</option>)}
      </Select>
    </div>
  );
}

function ColorParam({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return (
    <div className="space-y-2">
      <Label>{label}</Label>
      <div className="flex gap-2">
        <Input type="color" className="w-14 p-1" value={value} onChange={(event) => onChange(event.target.value)} />
        <Input value={value} onChange={(event) => onChange(event.target.value)} />
      </div>
    </div>
  );
}

function toFlowNodes(nodes: MaterialBlueprintNode[]): Node<BlueprintNodeData>[] {
  return nodes.map((node) => ({
    id: node.id,
    type: "materialNode",
    position: node.position,
    data: { label: node.label, kind: node.kind },
  }));
}

function toFlowEdges(links: { id: string; from: string; to: string }[]): Edge[] {
  return links.map((link) => ({
    id: link.id,
    source: link.from.split(".")[0],
    target: link.to.split(".")[0],
    animated: true,
    markerEnd: { type: MarkerType.ArrowClosed },
    style: { stroke: "#8b5cf6", strokeWidth: 2 },
  }));
}

function nodeColor(kind: string) {
  const colors: Record<string, string> = {
    palette: "#2563eb",
    pattern: "#0f766e",
    stylization: "#7c3aed",
    surface: "#64748b",
    variation: "#b45309",
    output: "#be123c",
  };
  return colors[kind] ?? "#475569";
}

function nodeLabel(kind: MaterialBlueprintNodeKind) {
  const labels: Record<MaterialBlueprintNodeKind, string> = {
    palette: "Palette",
    pattern: "Pattern Layer",
    stylization: "Stylization",
    surface: "Surface",
    variation: "Variation",
    output: "Output",
  };
  return labels[kind];
}

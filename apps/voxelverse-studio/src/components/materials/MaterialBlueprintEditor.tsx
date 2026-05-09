import { useCallback, useEffect, useMemo, useState, type Dispatch, type SetStateAction } from "react";
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
  type OnNodesChange,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { AlertTriangle, Plus, SlidersHorizontal, Trash2 } from "lucide-react";
import type { GraphConnection, GraphNode, GraphNodeKind, MaterialBlueprintNode, MaterialBlueprintNodeKind, MaterialEditorMode, MaterialFaceDef, ParamValue, ProceduralGraph } from "../../types/studio";
import { compileRecipeFromBlueprint, createBlueprintNode } from "../../lib/blueprint/materialBlueprint";
import { getNodeDef, NODE_CATEGORIES, NODES_BY_CATEGORY } from "../../lib/graph/nodeDefs";
import { validateGraph, type GraphError } from "../../lib/graph/graphValidator";
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
  /** Initial editor mode; defaults to "simple" for new empty materials. */
  initialMode?: MaterialEditorMode;
};

type BlueprintNodeData = {
  label: string;
  kind: MaterialBlueprintNodeKind;
};

const nodeTypes = { materialNode: MaterialNode };

export function MaterialBlueprintEditor({ material, onChange, initialMode = "advanced" }: MaterialBlueprintEditorProps) {
  const initialNodes = useMemo(() => toFlowNodes(material.blueprint.nodes), [material.blueprint.nodes]);
  const initialEdges = useMemo(() => toFlowEdges(material.blueprint.links), [material.blueprint.links]);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [selectedId, setSelectedId] = useState(material.blueprint.nodes[0]?.id ?? "");
  const [menu, setMenu] = useState<{ x: number; y: number; flowX: number; flowY: number } | null>(null);
  const [editorMode, setEditorMode] = useState<MaterialEditorMode>(initialMode);

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
    <div className="space-y-3">
      {/* Mode toggle */}
      <div className="flex items-center justify-between gap-3">
        <div className="text-xs text-muted-foreground">
          {editorMode === "simple"
            ? "Key parameters only — switch to Advanced for the full graph."
            : "Full node graph — right-click canvas to add nodes."}
        </div>
        <div className="flex shrink-0 gap-1 rounded-md border p-0.5">
          {(["simple", "advanced"] as const).map((mode) => (
            <button
              key={mode}
              type="button"
              onClick={() => setEditorMode(mode)}
              className={cn(
                "rounded px-3 py-1 text-xs font-medium transition-colors capitalize",
                editorMode === mode ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground",
              )}
            >
              {mode}
            </button>
          ))}
        </div>
      </div>

      {editorMode === "simple" ? (
        <SimpleModePanel material={material} onChange={onChange} />
      ) : material.graph ? (
        <ProceduralGraphPanel
          material={material}
          onChange={onChange}
        />
      ) : (
        <AdvancedGraphPanel
          material={material}
          onChange={onChange}
          nodes={nodes}
          edges={edges}
          setNodes={setNodes}
          setEdges={setEdges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          selectedId={selectedId}
          setSelectedId={setSelectedId}
          menu={menu}
          setMenu={setMenu}
          commitBlueprint={commitBlueprint}
          selected={selected}
          deleteSelectedNode={deleteSelectedNode}
          addNode={addNode}
          updateNode={updateNode}
        />
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// ProceduralGraph panel — real typed node graph (Phase 2)
// ---------------------------------------------------------------------------

type GraphNodeData = { label: string; kind: GraphNodeKind; color: string };
const graphNodeTypes = { graphNode: GraphNodeComponent };

function graphNodesToFlow(nodes: GraphNode[]): Node<GraphNodeData>[] {
  return nodes.map((n) => {
    const def = getNodeDef(n.kind);
    return {
      id: n.id,
      type: "graphNode",
      position: n.position,
      data: { label: n.label ?? def?.label ?? n.kind, kind: n.kind, color: def?.color ?? "#475569" },
    };
  });
}

function graphConnectionsToFlow(connections: GraphConnection[]): Edge[] {
  return connections.map((c) => ({
    id: c.id,
    source: c.fromNode,
    sourceHandle: c.fromPort,
    target: c.toNode,
    targetHandle: c.toPort,
    animated: true,
    markerEnd: { type: MarkerType.ArrowClosed },
    style: { stroke: "#8b5cf6", strokeWidth: 2 },
  }));
}

function ProceduralGraphPanel({
  material,
  onChange,
}: {
  material: MaterialFaceDef;
  onChange: (material: MaterialFaceDef, message?: string) => void;
}) {
  const graph = material.graph!;
  const initialNodes = useMemo(() => graphNodesToFlow(graph.nodes), [graph.nodes]);
  const initialEdges = useMemo(() => graphConnectionsToFlow(graph.connections), [graph.connections]);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; flowX: number; flowY: number } | null>(null);
  const errors = useMemo(() => validateGraph(graph), [graph]);

  useEffect(() => setNodes(graphNodesToFlow(graph.nodes)), [graph.nodes, setNodes]);
  useEffect(() => setEdges(graphConnectionsToFlow(graph.connections)), [graph.connections, setEdges]);

  const selected = graph.nodes.find((n) => n.id === selectedId) ?? null;

  function commitGraph(nextGraph: ProceduralGraph, message = "Graph updated") {
    onChange({ ...material, graph: nextGraph, previewVersion: material.previewVersion + 1 }, message);
  }

  const onConnect = useCallback((connection: Connection) => {
    if (!connection.source || !connection.target || !connection.sourceHandle || !connection.targetHandle) return;
    const connId = `${connection.source}-${connection.sourceHandle}-${connection.target}-${connection.targetHandle}-${Date.now()}`;
    const nextConn: GraphConnection = {
      id: connId,
      fromNode: connection.source,
      fromPort: connection.sourceHandle,
      toNode: connection.target,
      toPort: connection.targetHandle,
    };
    setEdges((curr) => addEdge({ ...connection, id: connId, animated: true, markerEnd: { type: MarkerType.ArrowClosed }, style: { stroke: "#8b5cf6", strokeWidth: 2 } }, curr));
    commitGraph({ ...graph, connections: [...graph.connections, nextConn] }, "Connection added");
  }, [graph, commitGraph, setEdges]);

  function updateGraphNode(node: GraphNode) {
    commitGraph({ ...graph, nodes: graph.nodes.map((n) => n.id === node.id ? node : node) }, "Node params updated");
  }

  function deleteSelectedNode() {
    if (!selectedId) return;
    const node = graph.nodes.find((n) => n.id === selectedId);
    if (!node || node.kind === "material_output") return;
    commitGraph({
      ...graph,
      nodes: graph.nodes.filter((n) => n.id !== selectedId),
      connections: graph.connections.filter((c) => c.fromNode !== selectedId && c.toNode !== selectedId),
    }, "Node removed");
    setSelectedId(null);
  }

  function addGraphNode(kind: GraphNodeKind) {
    if (!menu) return;
    const def = getNodeDef(kind);
    const defaultParams: Record<string, ParamValue> = {};
    for (const p of def?.params ?? []) defaultParams[p.name] = p.default;
    const newNode: GraphNode = {
      id: `${kind}_${Date.now()}`,
      kind,
      position: { x: menu.flowX, y: menu.flowY },
      params: defaultParams,
      exposedParams: (def?.params ?? []).filter((p) => p.exposed).map((p) => p.name),
    };
    commitGraph({ ...graph, nodes: [...graph.nodes, newNode] }, `${def?.label ?? kind} node added`);
    setSelectedId(newNode.id);
    setMenu(null);
  }

  return (
    <div className="space-y-2">
      {errors.length > 0 && (
        <div className="space-y-1">
          {errors.map((err, i) => (
            <div key={i} className={cn("flex items-start gap-2 rounded-md border px-3 py-2 text-xs", err.severity === "error" ? "border-red-500/30 bg-red-500/10 text-red-400" : "border-yellow-500/30 bg-yellow-500/10 text-yellow-400")}>
              <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
              {err.message}
            </div>
          ))}
        </div>
      )}
      <div className="grid h-full min-h-[720px] grid-cols-[minmax(0,1fr)_320px] overflow-hidden rounded-md border bg-[#0b0d12]">
        <div className="relative min-h-0">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={graphNodeTypes}
            onNodesChange={(changes) => { onNodesChange(changes); }}
            onNodeDragStop={(_, node) => {
              commitGraph({ ...graph, nodes: graph.nodes.map((n) => n.id === node.id ? { ...n, position: node.position } : n) }, "Node moved");
            }}
            onEdgesChange={(changes) => {
              onEdgesChange(changes);
              // Remove deleted edges from graph state
              const deleted = changes.filter((c) => c.type === "remove").map((c) => c.id);
              if (deleted.length > 0) {
                commitGraph({ ...graph, connections: graph.connections.filter((c) => !deleted.includes(c.id)) }, "Connection removed");
              }
            }}
            onConnect={onConnect}
            onNodeClick={(_, node) => setSelectedId(node.id)}
            onPaneContextMenu={(event) => {
              event.preventDefault();
              const target = event.currentTarget as HTMLElement;
              const bounds = target.getBoundingClientRect();
              setMenu({ x: event.clientX - bounds.left, y: event.clientY - bounds.top, flowX: event.clientX - bounds.left, flowY: event.clientY - bounds.top });
            }}
            onPaneClick={() => setMenu(null)}
            fitView panOnDrag zoomOnScroll
            proOptions={{ hideAttribution: true }}
          >
            <Background color="#252a34" gap={24} size={1} />
            <MiniMap pannable zoomable />
            <Controls />
          </ReactFlow>
          {menu && (
            <div className="absolute z-20 min-w-[200px] rounded-lg border bg-card p-2 shadow-2xl" style={{ left: menu.x, top: menu.y }}>
              <div className="mb-1 px-2 py-1 text-xs font-medium text-muted-foreground">Add node</div>
              {NODE_CATEGORIES.filter((cat) => cat !== "output").map((cat) => (
                <div key={cat}>
                  <div className="px-2 pt-2 pb-0.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60">{cat}</div>
                  {NODES_BY_CATEGORY[cat].map((def) => (
                    <button key={def.kind} type="button" className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-muted" onClick={() => addGraphNode(def.kind)}>
                      <span className="h-2 w-2 rounded-sm shrink-0" style={{ backgroundColor: def.color }} />
                      {def.label}
                    </button>
                  ))}
                </div>
              ))}
            </div>
          )}
        </div>
        <aside className="min-h-0 overflow-auto border-l bg-card/95 p-4">
          <div className="mb-4 flex items-center justify-between gap-3">
            <div>
              <div className="text-sm font-semibold">Node Parameters</div>
              <div className="text-xs text-muted-foreground">Right-click canvas to add nodes.</div>
            </div>
            <Button variant="ghost" size="icon" onClick={deleteSelectedNode} disabled={!selected || selected.kind === "material_output"}>
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
          {selected ? (
            <GraphNodeInspector node={selected} onChange={updateGraphNode} />
          ) : (
            <div className="rounded-md border border-dashed p-4 text-sm text-muted-foreground">Select a node to edit its parameters.</div>
          )}
        </aside>
      </div>
    </div>
  );
}

function GraphNodeComponent({ data, selected }: NodeProps<Node<GraphNodeData>>) {
  const def = getNodeDef(data.kind);
  return (
    <div className={cn("min-w-[180px] overflow-hidden rounded-md border bg-[#11151c] shadow-xl", selected ? "border-amber-400" : "border-slate-700")}>
      <div className="flex items-center gap-2 px-3 py-2 text-xs font-semibold text-white" style={{ backgroundColor: data.color }}>
        <SlidersHorizontal className="h-3.5 w-3.5 shrink-0" />
        <span className="truncate">{data.label}</span>
      </div>
      <div className="relative space-y-1.5 px-3 py-2 text-xs text-slate-300">
        {(def?.inputs ?? []).map((port) => (
          <div key={port.name} className="flex items-center gap-2">
            <Handle type="target" position={Position.Left} id={port.name} style={{ top: "auto", position: "relative", transform: "none" }} className="!relative !h-2.5 !w-2.5 !border-slate-900 !bg-cyan-400" />
            <span className="text-slate-400">{port.label}</span>
          </div>
        ))}
        {(def?.outputs ?? []).map((port) => (
          <div key={port.name} className="flex items-center justify-end gap-2">
            <span className="text-slate-400">{port.label}</span>
            <Handle type="source" position={Position.Right} id={port.name} style={{ top: "auto", position: "relative", transform: "none" }} className="!relative !h-2.5 !w-2.5 !border-slate-900 !bg-violet-400" />
          </div>
        ))}
        {def?.inputs.length === 0 && def?.outputs.length === 0 && (
          <div className="text-slate-500 italic">Output (sink)</div>
        )}
      </div>
    </div>
  );
}

function GraphNodeInspector({ node, onChange }: { node: GraphNode; onChange: (node: GraphNode) => void }) {
  const def = getNodeDef(node.kind);
  if (!def) return <div className="text-xs text-muted-foreground">Unknown node kind: {node.kind}</div>;

  function setParam(name: string, value: ParamValue) {
    onChange({ ...node, params: { ...node.params, [name]: value } });
  }

  function toggleExposed(name: string) {
    const exposed = node.exposedParams.includes(name)
      ? node.exposedParams.filter((p) => p !== name)
      : [...node.exposedParams, name];
    onChange({ ...node, exposedParams: exposed });
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>Label</Label>
        <Input value={node.label ?? def.label} onChange={(e) => onChange({ ...node, label: e.target.value })} />
      </div>
      {def.params.map((param) => (
        <div key={param.name} className="space-y-2">
          <div className="flex items-center justify-between gap-2">
            <Label>{param.label}</Label>
            <button type="button" title="Toggle Simple Mode visibility" onClick={() => toggleExposed(param.name)} className={cn("rounded px-1.5 py-0.5 text-[10px]", node.exposedParams.includes(param.name) ? "bg-primary/20 text-primary" : "text-muted-foreground hover:text-foreground")}>
              {node.exposedParams.includes(param.name) ? "Exposed" : "Hidden"}
            </button>
          </div>
          {param.type === "Color" && (
            <div className="flex gap-2">
              <Input type="color" className="w-14 p-1" value={String(node.params[param.name] ?? param.default)} onChange={(e) => setParam(param.name, e.target.value)} />
              <Input value={String(node.params[param.name] ?? param.default)} onChange={(e) => setParam(param.name, e.target.value)} />
            </div>
          )}
          {param.type === "Float" && (
            <div className="space-y-1">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>{param.min ?? 0}</span>
                <span className="font-mono">{Number(node.params[param.name] ?? param.default).toFixed((param.step ?? 0.01) >= 1 ? 0 : 2)}</span>
                <span>{param.max ?? 1}</span>
              </div>
              <Slider min={param.min ?? 0} max={param.max ?? 1} step={param.step ?? 0.01} value={Number(node.params[param.name] ?? param.default)} onChange={(e) => setParam(param.name, Number(e.currentTarget.value))} />
            </div>
          )}
          {param.type === "Bool" && (
            <Switch checked={Boolean(node.params[param.name] ?? param.default)} onCheckedChange={(checked) => setParam(param.name, checked)} label={param.label} />
          )}
          {param.type === "Select" && param.options && (
            <Select value={String(node.params[param.name] ?? param.default)} onChange={(e) => setParam(param.name, e.target.value)}>
              {param.options.map((opt) => <option key={opt} value={opt}>{opt}</option>)}
            </Select>
          )}
        </div>
      ))}
      {def.kind === "material_output" && (
        <div className="rounded-md border bg-background/60 p-3 text-xs text-muted-foreground">
          The graph evaluates all connected upstream nodes and writes the result as albedo + roughness textures.
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Simple Mode — key parameters without the graph
// ---------------------------------------------------------------------------

/** Simple Mode when material.graph is present — shows exposed params from nodes. */
function GraphSimpleModePanel({
  material,
  onChange,
}: {
  material: MaterialFaceDef;
  onChange: (material: MaterialFaceDef, message?: string) => void;
}) {
  const graph = material.graph!;

  const exposedEntries: Array<{ node: GraphNode; paramName: string; label: string }> = [];
  for (const node of graph.nodes) {
    const def = getNodeDef(node.kind);
    for (const paramName of node.exposedParams) {
      const paramDef = def?.params.find((p) => p.name === paramName);
      if (paramDef) {
        exposedEntries.push({ node, paramName, label: paramDef.exposedLabel ?? paramDef.label });
      }
    }
  }

  function setNodeParam(nodeId: string, paramName: string, value: ParamValue) {
    const nextGraph: ProceduralGraph = {
      ...graph,
      nodes: graph.nodes.map((n) => n.id === nodeId ? { ...n, params: { ...n.params, [paramName]: value } } : n),
    };
    onChange({ ...material, graph: nextGraph, previewVersion: material.previewVersion + 1 }, "Parameter updated");
  }

  if (exposedEntries.length === 0) {
    return (
      <div className="rounded-lg border border-dashed p-5 text-sm text-muted-foreground">
        No parameters exposed yet. Switch to Advanced mode, select a node, and mark params as Exposed.
      </div>
    );
  }

  return (
    <div className="grid gap-4 rounded-lg border bg-card p-5">
      {exposedEntries.map(({ node, paramName, label }) => {
        const def = getNodeDef(node.kind);
        const paramDef = def?.params.find((p) => p.name === paramName);
        if (!paramDef) return null;
        const value = node.params[paramName] ?? paramDef.default;

        return (
          <div key={`${node.id}-${paramName}`} className="space-y-2">
            <Label>{label}</Label>
            {paramDef.type === "Color" && (
              <div className="flex gap-2">
                <Input type="color" className="w-14 p-1" value={String(value)} onChange={(e) => setNodeParam(node.id, paramName, e.target.value)} />
                <Input value={String(value)} onChange={(e) => setNodeParam(node.id, paramName, e.target.value)} />
              </div>
            )}
            {paramDef.type === "Float" && (
              <div className="space-y-1">
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>{paramDef.min ?? 0}</span>
                  <span className="font-mono">{Number(value).toFixed((paramDef.step ?? 0.01) >= 1 ? 0 : 2)}</span>
                  <span>{paramDef.max ?? 1}</span>
                </div>
                <Slider min={paramDef.min ?? 0} max={paramDef.max ?? 1} step={paramDef.step ?? 0.01} value={Number(value)} onChange={(e) => setNodeParam(node.id, paramName, Number(e.currentTarget.value))} />
              </div>
            )}
            {paramDef.type === "Bool" && (
              <Switch checked={Boolean(value)} onCheckedChange={(checked) => setNodeParam(node.id, paramName, checked)} label={label} />
            )}
          </div>
        );
      })}
    </div>
  );
}

function SimpleModePanel({
  material,
  onChange,
}: {
  material: MaterialFaceDef;
  onChange: (material: MaterialFaceDef, message?: string) => void;
}) {
  // Phase 2: if graph is present, show exposed params from graph nodes
  if (material.graph) {
    return <GraphSimpleModePanel material={material} onChange={onChange} />;
  }

  // Legacy: use recipe fields directly
  const recipe = material.recipe;

  function setRecipeField<K extends keyof typeof recipe>(field: K, value: (typeof recipe)[K]) {
    const nextRecipe = { ...recipe, [field]: value };
    onChange(
      { ...material, recipe: nextRecipe, rawRonOverride: undefined, previewVersion: material.previewVersion + 1 },
      "Parameter updated",
    );
  }

  const palette = material.blueprint.nodes.find((n) => n.kind === "palette");
  const surface = material.blueprint.nodes.find((n) => n.kind === "surface");
  const variation = material.blueprint.nodes.find((n) => n.kind === "variation");
  const patterns = material.blueprint.nodes.filter((n) => n.kind === "pattern");

  function setPaletteColor(field: "baseColor" | "shadowColor" | "highlightColor", value: string) {
    const next = {
      ...recipe,
      [field]: value,
    };
    const updated: MaterialFaceDef = {
      ...material,
      recipe: next,
      blueprint: {
        ...material.blueprint,
        nodes: material.blueprint.nodes.map((n) =>
          n.kind === "palette" ? { ...n, params: { ...n.params, [field]: value } } : n,
        ),
      },
      rawRonOverride: undefined,
      previewVersion: material.previewVersion + 1,
    };
    onChange(updated, "Color updated");
  }

  void palette; void surface; void variation; void patterns;

  return (
    <div className="grid gap-5 rounded-lg border bg-card p-5">
      <section className="space-y-3">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Colors</h3>
        <ColorParam label="Base color" value={recipe.baseColor} onChange={(v) => setPaletteColor("baseColor", v)} />
        <ColorParam label="Shadow color" value={recipe.shadowColor} onChange={(v) => setPaletteColor("shadowColor", v)} />
        <ColorParam label="Highlight color" value={recipe.highlightColor} onChange={(v) => setPaletteColor("highlightColor", v)} />
      </section>

      <section className="space-y-3">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Patterns</h3>
        {recipe.patternLayers.length === 0 ? (
          <p className="rounded-md border border-dashed p-3 text-xs text-muted-foreground">
            No pattern layers yet. Switch to Advanced mode to add nodes.
          </p>
        ) : (
          recipe.patternLayers.map((layer, i) => (
            <div key={layer.id} className="rounded-md border bg-background/55 p-3">
              <div className="mb-2 flex items-center justify-between gap-2">
                <span className="text-xs font-medium capitalize">{layer.kind.replace(/_/g, " ")} (layer {i + 1})</span>
                <Switch
                  checked={layer.enabled}
                  onCheckedChange={(checked) => {
                    const next = recipe.patternLayers.map((l, idx) => idx === i ? { ...l, enabled: checked } : l);
                    setRecipeField("patternLayers", next);
                  }}
                />
              </div>
              <div className="space-y-2">
                <div className="flex items-center justify-between gap-2">
                  <Label>Strength</Label>
                  <span className="text-xs text-muted-foreground">{layer.strength.toFixed(2)}</span>
                </div>
                <Slider
                  min={0} max={1} step={0.01} value={layer.strength}
                  onChange={(e) => {
                    const next = recipe.patternLayers.map((l, idx) =>
                      idx === i ? { ...l, strength: Number(e.currentTarget.value) } : l,
                    );
                    setRecipeField("patternLayers", next);
                  }}
                />
              </div>
            </div>
          ))
        )}
      </section>

      <section className="space-y-3">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Surface</h3>
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-1">
            <div className="flex items-center justify-between gap-1 text-xs">
              <Label>Roughness</Label>
              <span className="text-muted-foreground">{recipe.surface.roughness.toFixed(2)}</span>
            </div>
            <Slider min={0} max={1} step={0.01} value={recipe.surface.roughness}
              onChange={(e) => setRecipeField("surface", { ...recipe.surface, roughness: Number(e.currentTarget.value) })}
            />
          </div>
          <div className="space-y-1">
            <div className="flex items-center justify-between gap-1 text-xs">
              <Label>Saturation</Label>
              <span className="text-muted-foreground">{recipe.stylization.saturation.toFixed(2)}</span>
            </div>
            <Slider min={0} max={1.6} step={0.01} value={recipe.stylization.saturation}
              onChange={(e) => setRecipeField("stylization", { ...recipe.stylization, saturation: Number(e.currentTarget.value) })}
            />
          </div>
        </div>
      </section>

      <section className="space-y-3">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Variation</h3>
        <Switch
          checked={recipe.variation.enabled}
          onCheckedChange={(checked) => setRecipeField("variation", { ...recipe.variation, enabled: checked })}
          label="Per-block variation"
        />
        {recipe.variation.enabled && (
          <div className="space-y-1">
            <div className="flex items-center justify-between gap-1 text-xs">
              <Label>Strength</Label>
              <span className="text-muted-foreground">{recipe.variation.perBlockStrength.toFixed(2)}</span>
            </div>
            <Slider min={0} max={0.5} step={0.01} value={recipe.variation.perBlockStrength}
              onChange={(e) => setRecipeField("variation", { ...recipe.variation, perBlockStrength: Number(e.currentTarget.value) })}
            />
          </div>
        )}
      </section>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Advanced mode — full graph editor (extracted from former single component)
// ---------------------------------------------------------------------------

function AdvancedGraphPanel({
  material, onChange,
  nodes, edges, setNodes, setEdges, onNodesChange, onEdgesChange,
  selectedId, setSelectedId, menu, setMenu,
  commitBlueprint, selected, deleteSelectedNode, addNode, updateNode,
}: {
  material: MaterialFaceDef;
  onChange: (material: MaterialFaceDef, message?: string) => void;
  nodes: Node<BlueprintNodeData>[];
  edges: Edge[];
  setNodes: Dispatch<SetStateAction<Node<BlueprintNodeData>[]>>;
  setEdges: ReturnType<typeof useEdgesState>[1];
  onNodesChange: OnNodesChange<Node<BlueprintNodeData>>;
  onEdgesChange: ReturnType<typeof useEdgesState>[2];
  selectedId: string;
  setSelectedId: (id: string) => void;
  menu: { x: number; y: number; flowX: number; flowY: number } | null;
  setMenu: (m: { x: number; y: number; flowX: number; flowY: number } | null) => void;
  commitBlueprint: (nodes: MaterialBlueprintNode[], links?: MaterialFaceDef["blueprint"]["links"], message?: string) => void;
  selected: MaterialBlueprintNode | undefined;
  deleteSelectedNode: () => void;
  addNode: (kind: MaterialBlueprintNodeKind) => void;
  updateNode: (node: MaterialBlueprintNode) => void;
}) {
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

  void onChange;

  return (
    <div className="grid h-full min-h-[720px] grid-cols-[minmax(0,1fr)_320px] overflow-hidden rounded-md border bg-[#0b0d12]">
      <div className="relative min-h-0">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={(changes) => { onNodesChange(changes); }}
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
          fitView panOnDrag zoomOnScroll selectionOnDrag
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

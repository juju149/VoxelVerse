import type { ProceduralGraph } from "../../types/studio";
import { getNodeDef } from "./nodeDefs";

export type GraphError = {
  nodeId?: string;
  message: string;
  severity: "error" | "warning";
};

// ---------------------------------------------------------------------------
// Cycle detection (DFS coloring)
// ---------------------------------------------------------------------------

function hasCycle(graph: ProceduralGraph): boolean {
  const WHITE = 0, GRAY = 1, BLACK = 2;
  const color: Record<string, number> = {};

  function dfs(nodeId: string): boolean {
    color[nodeId] = GRAY;
    for (const conn of graph.connections) {
      if (conn.fromNode !== nodeId) continue;
      const next = conn.toNode;
      if (!color[next]) color[next] = WHITE;
      if (color[next] === GRAY) return true;
      if (color[next] !== BLACK && dfs(next)) return true;
    }
    color[nodeId] = BLACK;
    return false;
  }

  for (const node of graph.nodes) {
    if (!color[node.id]) color[node.id] = WHITE;
    if (color[node.id] === WHITE && dfs(node.id)) return true;
  }
  return false;
}

// ---------------------------------------------------------------------------
// Public validator
// ---------------------------------------------------------------------------

export function validateGraph(graph: ProceduralGraph): GraphError[] {
  const errors: GraphError[] = [];
  const nodeIds = new Set(graph.nodes.map((n) => n.id));

  // 1. Must have a material_output node
  const outputNode = graph.nodes.find((n) => n.kind === "material_output");
  if (!outputNode) {
    errors.push({ message: "Graph has no Material Output node.", severity: "error" });
    return errors; // rest of checks need output
  }

  // 2. Output must receive an albedo connection
  const albedoConn = graph.connections.find(
    (c) => c.toNode === outputNode.id && c.toPort === "albedo",
  );
  if (!albedoConn) {
    errors.push({
      nodeId: outputNode.id,
      message: "Material Output has no Albedo input connected.",
      severity: "error",
    });
  }

  // 3. Dangling connections
  for (const conn of graph.connections) {
    if (!nodeIds.has(conn.fromNode)) {
      errors.push({ message: `Connection references missing node "${conn.fromNode}".`, severity: "error" });
    }
    if (!nodeIds.has(conn.toNode)) {
      errors.push({ message: `Connection references missing node "${conn.toNode}".`, severity: "error" });
    }
  }

  // 4. Unknown node kinds
  for (const node of graph.nodes) {
    if (!getNodeDef(node.kind)) {
      errors.push({ nodeId: node.id, message: `Unknown node kind "${node.kind}".`, severity: "error" });
    }
  }

  // 5. Cycle detection
  if (hasCycle(graph)) {
    errors.push({ message: "Graph contains a cycle — evaluation is impossible.", severity: "error" });
  }

  // 6. Budget warnings
  if (graph.nodes.length > 64) {
    errors.push({
      message: `Graph has ${graph.nodes.length} nodes (recommended max: 64).`,
      severity: "warning",
    });
  }
  if (graph.connections.length > 128) {
    errors.push({
      message: `Graph has ${graph.connections.length} connections (recommended max: 128).`,
      severity: "warning",
    });
  }

  return errors;
}

/** Returns true if the graph has at least one error (not just warnings). */
export function graphHasErrors(graph: ProceduralGraph): boolean {
  return validateGraph(graph).some((e) => e.severity === "error");
}

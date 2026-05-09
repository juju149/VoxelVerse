export type StudioRoute = "materials" | "blocks";

export type ItemStatus = "valid" | "warning" | "error" | "dirty";

/** Determines which view is shown inside the material editor panel. */
export type MaterialEditorMode = "simple" | "advanced";

/**
 * Internal style hint used only by the template system when bootstrapping a
 * new material from a preset. Not stored on MaterialFaceDef.
 */
export type MaterialStylePreset =
  | "soft_natural"
  | "clean_stylized"
  | "rich_organic"
  | "simple_flat";

export type ParamKind = "Color" | "Float" | "Int" | "Bool" | "Text";

export type ParamValue = string | number | boolean;

export type MaterialParam = {
  name: string;
  kind: ParamKind;
  value: ParamValue;
  min?: number;
  max?: number;
};

export type MaterialPatternKind =
  | "soft_noise"
  | "soft_blotches"
  | "organic_cells"
  | "rounded_pebbles"
  | "edge_band"
  | "patch_cells"
  | "rings"
  | "stripes"
  | "dots"
  | "bands"
  | "cracks"
  | "flat";

export type MaterialBlendMode =
  | "mix"
  | "multiply"
  | "screen"
  | "overlay"
  | "add"
  | "subtract"
  | "shadow"
  | "highlight";

export type MaterialMaskKind =
  | "none"
  | "top_band"
  | "bottom_band"
  | "vertical_gradient"
  | "center_soft"
  | "edge_wear";

export type MaterialLayerDomain = "uv" | "warped_uv" | "radial" | "vertical" | "horizontal";

export type MaterialPatternLayer = {
  id: string;
  kind: MaterialPatternKind;
  blend: MaterialBlendMode;
  domain: MaterialLayerDomain;
  mask: MaterialMaskKind;
  strength: number;
  scale: number;
  contrast: number;
  softness: number;
  warp: number;
  offsetX: number;
  offsetY: number;
  threshold?: number;
  color?: string;
  enabled: boolean;
};

export type MaterialSurface = {
  roughness: number;
  heightStrength: number;
  normalStrength: number;
  edgeSoftness: number;
};

export type MaterialVariation = {
  enabled: boolean;
  perBlockStrength: number;
  colorJitter: number;
  patternJitter: number;
};

export type MaterialStylization = {
  colorSteps: number;
  smoothing: number;
  saturation: number;
  valueBoost: number;
  microDetail: number;
};

// ---------------------------------------------------------------------------
// Phase 2 — ProceduralGraph: typed node graph with real evaluation
// ---------------------------------------------------------------------------

/** Every node kind supported by the graph evaluator. */
export type GraphNodeKind =
  // Input
  | "color" | "float"
  // Pattern (→ Mask 0..1)
  | "noise_fbm" | "voronoi" | "stripes" | "rings" | "dots" | "flat"
  // Shape (→ Mask)
  | "gradient" | "band" | "edge_mask"
  // Warp
  | "domain_warp"
  // Blend (→ Color)
  | "blend_mix" | "blend_multiply" | "blend_screen" | "blend_overlay"
  // Adjust
  | "remap" | "contrast_adjust" | "colorize" | "quantize"
  // Output (sink)
  | "material_output";

export type GraphNode = {
  id: string;
  kind: GraphNodeKind;
  /** Optional display label override. */
  label?: string;
  position: { x: number; y: number };
  /** Node-specific param values keyed by param name. */
  params: Record<string, ParamValue>;
  /** Param names that are surfaced in Simple Mode. */
  exposedParams: string[];
};

export type GraphConnection = {
  id: string;
  fromNode: string;
  fromPort: string;
  toNode: string;
  toPort: string;
};

export type ProceduralGraph = {
  version: 1;
  nodes: GraphNode[];
  connections: GraphConnection[];
};

// ---------------------------------------------------------------------------
// Legacy blueprint types (kept for backward compat with v5 recipe materials)
// ---------------------------------------------------------------------------

export type MaterialBlueprintNodeKind =
  | "palette"
  | "pattern"
  | "stylization"
  | "surface"
  | "variation"
  | "output";

export type MaterialBlueprintNode = {
  id: string;
  kind: MaterialBlueprintNodeKind;
  label: string;
  position: { x: number; y: number };
  params: Record<string, ParamValue>;
};

export type MaterialBlueprintLink = {
  id: string;
  from: string;
  to: string;
};

export type MaterialBlueprint = {
  nodes: MaterialBlueprintNode[];
  links: MaterialBlueprintLink[];
};

export type ProceduralMaterialRecipe = {
  style: MaterialStylePreset;
  baseColor: string;
  shadowColor: string;
  highlightColor: string;
  patternLayers: MaterialPatternLayer[];
  surface: MaterialSurface;
  stylization: MaterialStylization;
  variation: MaterialVariation;
  params: MaterialParam[];
};

export type MaterialFaceDef = {
  id: string;
  displayName: string;
  /** Free-form category string, e.g. "terrain", "stone", "wood", "custom". */
  category: string;
  resolutionPreview: number;
  seed: number;
  /**
   * Phase 2: typed node graph — source of truth when present.
   * When set, the graph evaluator drives the canvas preview.
   * When absent, the legacy recipe evaluator is used (backward compat).
   */
  graph?: ProceduralGraph;
  blueprint: MaterialBlueprint;
  recipe: ProceduralMaterialRecipe;
  status: ItemStatus;
  rawRonOverride?: string;
  previewVersion: number;
};

export type BreakSpeedPreset = "soft" | "normal" | "hard" | "very-hard";

export type BlockKind =
  | "cube"
  | "slab"
  | "stairs"
  | "cross_plant"
  | "liquid"
  | "custom";

export type FaceMaterialRefs = {
  all?: string;
  top?: string;
  side?: string;
  bottom?: string;
  north?: string;
  south?: string;
  east?: string;
  west?: string;
};

export type BlockGeometry = {
  kind: BlockKind;
  collisionShape: "solid_cube" | "partial" | "cross" | "fluid" | "none";
  customModel?: string;
};

export type BlockRender = {
  materials: FaceMaterialRefs;
  tint?: string;
  ambientOcclusion: boolean;
  transparent: boolean;
  cullFaces: boolean;
  lightEmission: number;
};

export type BlockGameplay = {
  walkThrough: boolean;
  hardness: number;
  breakSpeedPreset: BreakSpeedPreset;
  drops: string[];
};

export type BlockDef = {
  id: string;
  displayName: string;
  seed: number;
  /** RGB color in 0–1 range used for distant LOD rendering. */
  color: [number, number, number];
  geometry: BlockGeometry;
  render: BlockRender;
  gameplay: BlockGameplay;
  category: string;
  tags: string[];
  status: ItemStatus;
  rawRonOverride?: string;
};

export type ValidationFixKind =
  | "normalize-id"
  | "use-dirt-bottom"
  | "clamp-hardness"
  | "clamp-seed"
  | "assign-all-material"
  | "add-lod-color";

export type ValidationIssue = {
  id: string;
  severity: "warning" | "error";
  message: string;
  path: string;
  details?: string;
  fixable: boolean;
  fixKind?: ValidationFixKind;
  targetId?: string;
};

export type SeedPolicy = {
  packSeed: number;
  previewPositionSeed: number;
};

export type PackProject = {
  schemaVersion: 5;
  id: string;
  namespace: string;
  name: string;
  path: string;
  packSeed: number;
  seedPolicy: SeedPolicy;
  materials: MaterialFaceDef[];
  blocks: BlockDef[];
  validationIssues: ValidationIssue[];
  hasUnsavedChanges: boolean;
  lastSavedAt?: string;
};

/** Metadata for a built-in material template shown in the template gallery. */
export type MaterialTemplate = {
  /** Stable key used to look up the preset recipe (internal to materialPresets). */
  templateKey: string;
  label: string;
  description: string;
  category: string;
};

export type BlockTemplate = {
  kind: BlockKind;
  label: string;
  description: string;
};

export type ExportFile = {
  path: string;
  content: string;
};

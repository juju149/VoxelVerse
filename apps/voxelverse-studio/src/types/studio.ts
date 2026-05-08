export type StudioRoute = "materials" | "blocks";

export type ItemStatus = "valid" | "warning" | "error" | "dirty";

export type MaterialKind =
  | "grass_top"
  | "grass_side"
  | "dirt_base"
  | "stone_base"
  | "sand_base"
  | "wood_rings"
  | "custom";

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
  materialKind: MaterialKind;
  resolutionPreview: number;
  seed: number;
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
  | "assign-all-material";

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
  schemaVersion: 4;
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

export type MaterialPresetChoice = {
  kind: MaterialKind;
  label: string;
  description: string;
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

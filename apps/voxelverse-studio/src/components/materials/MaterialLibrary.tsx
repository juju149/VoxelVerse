import { Plus, Sparkles } from "lucide-react";
import type { MaterialFaceDef, MaterialKind } from "../../types/studio";
import { cn } from "../../lib/cn";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";

type MaterialLibraryProps = {
  materials: MaterialFaceDef[];
  selectedId: string;
  onSelect: (id: string) => void;
  onNew: () => void;
  onQuickCreate: (kind: MaterialKind) => void;
};

export function MaterialLibrary({ materials, selectedId, onSelect, onNew, onQuickCreate }: MaterialLibraryProps) {
  return (
    <aside className="flex min-h-0 flex-col rounded-lg border bg-card">
      <div className="flex items-center justify-between border-b p-4">
        <div>
          <h2 className="text-sm font-semibold">Material Library</h2>
          <p className="text-xs text-muted-foreground">Procedural face materials</p>
        </div>
        <Button size="icon" variant="secondary" onClick={onNew} aria-label="New material">
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      <div className="min-h-0 flex-1 space-y-2 overflow-auto p-3">
        {materials.length === 0 ? (
          <div className="space-y-3 rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
            <p>No materials yet. Create your first procedural material.</p>
            <Button className="w-full" onClick={() => onQuickCreate("grass_top")}>Create Grass Top</Button>
            <Button className="w-full" variant="secondary" onClick={() => onQuickCreate("dirt_base")}>Create Dirt</Button>
            <Button className="w-full" variant="outline" onClick={onNew}>Start from Template</Button>
          </div>
        ) : materials.map((material) => (
          <button
            key={material.id}
            type="button"
            onClick={() => onSelect(material.id)}
            className={cn(
              "flex w-full items-center gap-3 rounded-md border p-3 text-left transition-colors",
              selectedId === material.id ? "border-primary/50 bg-primary/10" : "bg-background/45 hover:bg-muted",
            )}
          >
            <span className="grid h-10 w-10 shrink-0 place-items-center rounded-md bg-primary/12 text-primary">
              <Sparkles className="h-4 w-4" />
            </span>
            <span className="min-w-0 flex-1">
              <span className="block truncate text-sm font-medium">{material.displayName}</span>
              <span className="block truncate text-xs text-muted-foreground">{material.id}</span>
              <span className="block text-xs text-muted-foreground">Seed {material.seed}</span>
            </span>
            <Badge tone={material.status === "error" ? "error" : material.status === "warning" ? "warning" : "ready"}>
              {material.status}
            </Badge>
          </button>
        ))}
      </div>
    </aside>
  );
}

import { Box, Plus } from "lucide-react";
import type { BlockDef } from "../../types/studio";
import { cn } from "../../lib/cn";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";

type BlockLibraryProps = {
  blocks: BlockDef[];
  selectedId: string;
  onSelect: (id: string) => void;
  onNew: () => void;
  onQuickCreate: () => void;
};

export function BlockLibrary({ blocks, selectedId, onSelect, onNew, onQuickCreate }: BlockLibraryProps) {
  return (
    <aside className="flex min-h-0 flex-col rounded-lg border bg-card">
      <div className="flex items-center justify-between border-b p-4">
        <div>
          <h2 className="text-sm font-semibold">Block Library</h2>
          <p className="text-xs text-muted-foreground">Cubes assembled from material faces</p>
        </div>
        <Button size="icon" variant="secondary" onClick={onNew} aria-label="New block">
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      <div className="min-h-0 flex-1 space-y-2 overflow-auto p-3">
        {blocks.length === 0 ? (
          <div className="space-y-3 rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
            <p>No blocks yet. Create your first block.</p>
            <Button className="w-full" onClick={onQuickCreate}>Create Grass Block</Button>
            <Button className="w-full" variant="outline" onClick={onNew}>Start from Template</Button>
          </div>
        ) : blocks.map((block) => (
          <button
            key={block.id}
            type="button"
            onClick={() => onSelect(block.id)}
            className={cn(
              "flex w-full items-center gap-3 rounded-md border p-3 text-left transition-colors",
              selectedId === block.id ? "border-primary/50 bg-primary/10" : "bg-background/45 hover:bg-muted",
            )}
          >
            <span className="grid h-10 w-10 shrink-0 place-items-center rounded-md bg-primary/12 text-primary">
              <Box className="h-4 w-4" />
            </span>
            <span className="min-w-0 flex-1">
              <span className="block truncate text-sm font-medium">{block.displayName}</span>
              <span className="block truncate text-xs text-muted-foreground">{block.id}</span>
              <span className="block text-xs text-muted-foreground">{block.geometry.kind} · seed {block.seed}</span>
            </span>
            <Badge tone={block.status === "error" ? "error" : block.status === "warning" ? "warning" : "ready"}>
              {block.status}
            </Badge>
          </button>
        ))}
      </div>
    </aside>
  );
}

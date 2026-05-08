import { useState } from "react";
import { materialTemplates } from "../../lib/presets/materialPresets";
import { Button } from "../ui/button";
import { Dialog } from "../ui/dialog";

type MaterialTemplateGalleryProps = {
  open: boolean;
  onClose: () => void;
  /** Called with the template key chosen by the user. */
  onSelect: (templateKey: string) => void;
};

const CATEGORY_ORDER = ["terrain", "stone", "wood", "natural", "building", "ore", "liquid", "special"];
const CATEGORY_LABELS: Record<string, string> = {
  terrain: "Terrain", stone: "Stone", wood: "Wood", natural: "Natural",
  building: "Building", ore: "Ore", liquid: "Liquid", special: "Special",
};

export function MaterialTemplateGallery({ open, onClose, onSelect }: MaterialTemplateGalleryProps) {
  const [selected, setSelected] = useState<string | null>(null);
  const [filter, setFilter] = useState("");

  const normalized = filter.trim().toLowerCase();
  const visible = normalized
    ? materialTemplates.filter(
        (t) => t.label.toLowerCase().includes(normalized) || t.category.includes(normalized),
      )
    : materialTemplates;

  // Group by category in defined order
  const grouped = CATEGORY_ORDER.map((cat) => ({
    cat,
    items: visible.filter((t) => t.category === cat),
  })).filter((g) => g.items.length > 0);

  function confirm() {
    if (selected) {
      onSelect(selected);
      setSelected(null);
      setFilter("");
    }
  }

  function handleClose() {
    setSelected(null);
    setFilter("");
    onClose();
  }

  return (
    <Dialog open={open} title="Start from Template" onClose={handleClose}>
      <div className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Choose a starting template. You can freely edit every parameter after creation.
        </p>

        <input
          type="search"
          placeholder="Filter templates…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="h-9 w-full rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
        />

        <div className="max-h-[420px] space-y-4 overflow-auto">
          {grouped.map(({ cat, items }) => (
            <section key={cat} className="space-y-2">
              <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                {CATEGORY_LABELS[cat] ?? cat}
              </h3>
              <div className="grid grid-cols-2 gap-2">
                {items.map((t) => (
                  <button
                    key={t.templateKey}
                    type="button"
                    onClick={() => setSelected(t.templateKey)}
                    className={`rounded-md border p-3 text-left text-sm transition-colors ${
                      selected === t.templateKey
                        ? "border-primary bg-primary/10"
                        : "bg-background/55 hover:bg-muted"
                    }`}
                  >
                    <span className="block font-medium">{t.label}</span>
                    <span className="mt-0.5 block text-xs text-muted-foreground">{t.description}</span>
                  </button>
                ))}
              </div>
            </section>
          ))}
          {grouped.length === 0 && (
            <p className="py-6 text-center text-sm text-muted-foreground">No templates match your filter.</p>
          )}
        </div>

        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={handleClose}>Cancel</Button>
          <Button onClick={confirm} disabled={!selected}>
            Use Template
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

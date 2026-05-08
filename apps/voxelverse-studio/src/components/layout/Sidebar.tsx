import { Blocks, Layers, Palette } from "lucide-react";
import { cn } from "../../lib/cn";
import type { StudioRoute } from "../../types/studio";

type SidebarProps = {
  route: StudioRoute;
  onRouteChange: (route: StudioRoute) => void;
};

const navItems = [
  { route: "materials" as const, label: "Materials", icon: Palette },
  { route: "blocks" as const, label: "Blocks", icon: Blocks },
];

export function Sidebar({ route, onRouteChange }: SidebarProps) {
  return (
    <aside className="flex w-64 shrink-0 flex-col border-r bg-black/20">
      <div className="flex h-16 items-center gap-3 border-b px-5">
        <div className="grid h-9 w-9 place-items-center rounded-lg bg-primary/20 text-indigo-200">
          <Layers className="h-5 w-5" />
        </div>
        <div>
          <div className="text-sm font-semibold">VoxelVerse Studio</div>
          <div className="text-xs text-muted-foreground">Pack authoring 0.1</div>
        </div>
      </div>
      <nav className="space-y-1 p-3">
        {navItems.map((item) => {
          const Icon = item.icon;
          const active = route === item.route;
          return (
            <button
              key={item.route}
              type="button"
              onClick={() => onRouteChange(item.route)}
              className={cn(
                "flex h-10 w-full items-center gap-3 rounded-md px-3 text-sm transition-colors",
                active
                  ? "bg-primary/18 text-foreground ring-1 ring-primary/25"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground",
              )}
            >
              <Icon className="h-4 w-4" />
              <span>{item.label}</span>
            </button>
          );
        })}
      </nav>
      <div className="mt-auto border-t p-4 text-xs text-muted-foreground">
        Local-first editor for procedural `.ron` materials and blocks.
      </div>
    </aside>
  );
}

import { ChevronRight, Save, ShieldCheck } from "lucide-react";
import { Button } from "../ui/button";
import type { PackProject, StudioRoute } from "../../types/studio";
import { ExportButton } from "../validation/ExportButton";

type TopBarProps = {
  project: PackProject;
  route: StudioRoute;
  onSave: () => void;
  onValidate: () => void;
  onExport: () => void;
};

export function TopBar({ project, route, onSave, onValidate, onExport }: TopBarProps) {
  const pageLabel = route === "materials" ? "Materials" : "Blocks";

  return (
    <header className="flex h-16 items-center justify-between border-b bg-background/80 px-5 backdrop-blur">
      <div className="flex min-w-0 items-center gap-3">
        <button className="flex h-9 items-center gap-2 rounded-md border bg-card px-3 text-sm shadow-sm" type="button">
          <span className="h-2 w-2 rounded-full bg-emerald-400" />
          {project.name}
        </button>
        <div className="flex min-w-0 items-center gap-2 text-sm text-muted-foreground">
          <span>{project.name}</span>
          <ChevronRight className="h-4 w-4" />
          <span className="font-medium text-foreground">{pageLabel}</span>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="secondary" onClick={onSave}>
          <Save className="h-4 w-4" />
          Save
        </Button>
        <Button variant="outline" onClick={onValidate}>
          <ShieldCheck className="h-4 w-4" />
          Validate
        </Button>
        <ExportButton
          disabled={project.validationIssues.some((issue) => issue.severity === "error")}
          onExport={onExport}
        />
      </div>
    </header>
  );
}

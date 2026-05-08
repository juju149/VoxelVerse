import { CheckCircle, Folder } from "lucide-react";
import { Badge } from "../ui/badge";
import type { PackProject } from "../../types/studio";

type StatusBarProps = {
  project: PackProject;
  statusMessage: string;
};

export function StatusBar({ project, statusMessage }: StatusBarProps) {
  const errors = project.validationIssues.filter((issue) => issue.severity === "error").length;
  const warnings = project.validationIssues.filter((issue) => issue.severity === "warning").length;

  return (
    <footer className="flex h-10 items-center justify-between border-t bg-background/80 px-5 text-xs text-muted-foreground">
      <div className="flex items-center gap-3">
        <span className="flex items-center gap-1.5">
          <Folder className="h-3.5 w-3.5" />
          Local mode
        </span>
        <span>{project.hasUnsavedChanges ? "Unsaved changes" : "All changes saved locally"}</span>
        <span>{statusMessage}</span>
      </div>
      <div className="flex items-center gap-2">
        <Badge tone={errors > 0 ? "error" : warnings > 0 ? "warning" : "ready"}>
          <CheckCircle className="mr-1 h-3 w-3" />
          {errors > 0 ? `${errors} error(s)` : warnings > 0 ? "Pack valid with warnings" : "Pack valid"}
        </Badge>
        <span>{project.path}</span>
      </div>
    </footer>
  );
}

import { AlertTriangle, CheckCircle } from "lucide-react";
import type { ValidationIssue } from "../../types/studio";
import { Button } from "../ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";

type ValidationPanelProps = {
  issues: ValidationIssue[];
  onFix: (issue: ValidationIssue) => void;
};

export function ValidationPanel({ issues, onFix }: ValidationPanelProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Validation</CardTitle>
        <CardDescription>Current pack data checks.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-2">
        {issues.length === 0 ? (
          <div className="flex items-center gap-2 rounded-md border bg-emerald-500/10 p-3 text-sm text-emerald-200">
            <CheckCircle className="h-4 w-4" />
            Pack valid.
          </div>
        ) : issues.map((issue) => (
          <div key={issue.id} className="rounded-md border bg-background/55 p-3">
            <div className="flex items-start gap-2">
              <AlertTriangle className={`mt-0.5 h-4 w-4 ${issue.severity === "error" ? "text-red-300" : "text-amber-300"}`} />
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium">{issue.message}</div>
                <div className="truncate text-xs text-muted-foreground">{issue.path}</div>
                {issue.details ? <div className="mt-1 text-xs text-muted-foreground">{issue.details}</div> : null}
              </div>
              {issue.fixable ? (
                <Button size="sm" variant="secondary" onClick={() => onFix(issue)}>Fix</Button>
              ) : (
                <Button size="sm" variant="ghost" onClick={() => window.alert(issue.details ?? issue.message)}>Details</Button>
              )}
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

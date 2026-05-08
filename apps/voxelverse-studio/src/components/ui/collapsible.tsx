import * as React from "react";
import { ChevronRight } from "lucide-react";
import { cn } from "../../lib/cn";

type CollapsibleProps = React.PropsWithChildren<{
  title: string;
  icon?: React.ReactNode;
  defaultOpen?: boolean;
}>;

export function Collapsible({ title, icon, defaultOpen = false, children }: CollapsibleProps) {
  return (
    <details className="group rounded-lg border bg-background/45" open={defaultOpen}>
      <summary className="flex cursor-pointer list-none items-center gap-2 px-3 py-2 text-sm font-medium">
        <ChevronRight className={cn("h-4 w-4 text-muted-foreground transition-transform group-open:rotate-90")} />
        {icon}
        <span>{title}</span>
      </summary>
      <div className="border-t p-3">{children}</div>
    </details>
  );
}

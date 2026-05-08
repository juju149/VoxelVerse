import * as React from "react";
import { cn } from "../../lib/cn";

type BadgeTone = "default" | "ready" | "warning" | "error" | "muted";

const tones: Record<BadgeTone, string> = {
  default: "border-primary/40 bg-primary/15 text-indigo-100",
  ready: "border-emerald-500/30 bg-emerald-500/10 text-emerald-200",
  warning: "border-amber-500/30 bg-amber-500/10 text-amber-200",
  error: "border-red-500/30 bg-red-500/10 text-red-200",
  muted: "border-border bg-muted text-muted-foreground",
};

export function Badge({
  className,
  tone = "default",
  ...props
}: React.HTMLAttributes<HTMLSpanElement> & { tone?: BadgeTone }) {
  return (
    <span
      className={cn(
        "inline-flex h-6 items-center rounded-md border px-2 text-[11px] font-medium",
        tones[tone],
        className,
      )}
      {...props}
    />
  );
}

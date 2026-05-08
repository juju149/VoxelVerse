import * as React from "react";
import { cn } from "../../lib/cn";

export function Alert({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("rounded-lg border bg-muted/40 p-3 text-sm", className)} {...props} />;
}

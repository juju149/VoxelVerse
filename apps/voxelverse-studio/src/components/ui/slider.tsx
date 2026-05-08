import * as React from "react";
import { cn } from "../../lib/cn";

export function Slider({ className, ...props }: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      type="range"
      className={cn("h-2 w-full accent-indigo-400", className)}
      {...props}
    />
  );
}

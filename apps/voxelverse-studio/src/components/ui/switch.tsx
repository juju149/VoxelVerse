import { cn } from "../../lib/cn";

type SwitchProps = {
  checked: boolean;
  onCheckedChange?: (checked: boolean) => void;
  label?: string;
};

export function Switch({ checked, onCheckedChange, label }: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onCheckedChange?.(!checked)}
      className="flex items-center gap-3 text-sm"
    >
      <span
        className={cn(
          "relative h-5 w-9 rounded-full border transition-colors",
          checked ? "border-primary bg-primary" : "border-border bg-muted",
        )}
      >
        <span
          className={cn(
            "absolute top-0.5 h-4 w-4 rounded-full bg-white transition-transform",
            checked ? "translate-x-4" : "translate-x-0.5",
          )}
        />
      </span>
      {label ? <span className="text-muted-foreground">{label}</span> : null}
    </button>
  );
}

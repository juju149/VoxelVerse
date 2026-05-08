import { Upload } from "lucide-react";
import { Button } from "../ui/button";

type ExportButtonProps = {
  disabled?: boolean;
  onExport: () => void;
};

export function ExportButton({ disabled = false, onExport }: ExportButtonProps) {
  return (
    <Button disabled={disabled} onClick={onExport} title={disabled ? "Fix validation errors before export." : "Export .ron files"}>
      <Upload className="h-4 w-4" />
      Export
    </Button>
  );
}

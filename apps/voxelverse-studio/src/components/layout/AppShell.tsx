import type { ReactNode } from "react";
import type { PackProject, StudioRoute } from "../../types/studio";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import { TopBar } from "./TopBar";

type AppShellProps = {
  project: PackProject;
  route: StudioRoute;
  statusMessage: string;
  onRouteChange: (route: StudioRoute) => void;
  onSave: () => void;
  onValidate: () => void;
  onExport: () => void;
  children: ReactNode;
};

export function AppShell({
  project,
  route,
  statusMessage,
  onRouteChange,
  onSave,
  onValidate,
  onExport,
  children,
}: AppShellProps) {
  return (
    <div className="flex h-screen min-h-[720px] overflow-hidden bg-background text-foreground">
      <Sidebar route={route} onRouteChange={onRouteChange} />
      <div className="flex min-w-0 flex-1 flex-col">
        <TopBar project={project} route={route} onSave={onSave} onValidate={onValidate} onExport={onExport} />
        <main className="min-h-0 flex-1 overflow-hidden">{children}</main>
        <StatusBar project={project} statusMessage={statusMessage} />
      </div>
    </div>
  );
}

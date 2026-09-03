import type { ReactNode } from "react";
import { Plus, Search } from "lucide-react";
import { Sidebar } from "./Sidebar";

export type AppSection =
  | "dashboard"
  | "customers"
  | "motorcycles"
  | "service"
  | "inventory"
  | "invoices"
  | "settings";

type AppShellProps = {
  activeSection: AppSection;
  onSectionChange: (section: AppSection) => void;
  onNewServiceVisit: () => void;
  children: ReactNode;
};

export function AppShell({
  activeSection,
  onSectionChange,
  onNewServiceVisit,
  children,
}: AppShellProps) {
  return (
    <div className="app-shell">
      <Sidebar
        activeSection={activeSection}
        onSectionChange={onSectionChange}
      />

      <div className="app-main">
        <header className="topbar">
          <div className="global-search">
            <Search size={18} strokeWidth={2} />

            <input
              type="search"
              aria-label="Global search"
              placeholder="Search customer, phone, plate, VIN..."
            />
          </div>

          <button
            className="primary-button"
            type="button"
            onClick={onNewServiceVisit}
          >
          <Plus size={18} />
            New Service Visit
          </button>
        </header>

        <main className="page-content">{children}</main>
      </div>
    </div>
  );
}

import {
  Bike,
  Boxes,
  FileText,
  LayoutDashboard,
  Settings,
  Users,
  Wrench,
} from "lucide-react";
import type { AppSection } from "./AppShell";

type SidebarProps = {
  activeSection: AppSection;
  onSectionChange: (section: AppSection) => void;
};

const primaryItems = [
  {
    id: "dashboard",
    label: "Dashboard",
    icon: LayoutDashboard,
  },
  {
    id: "customers",
    label: "Customers",
    icon: Users,
  },
  {
    id: "motorcycles",
    label: "Motorcycles",
    icon: Bike,
  },
  {
    id: "service",
    label: "Service",
    icon: Wrench,
  },
  {
    id: "inventory",
    label: "Inventory",
    icon: Boxes,
  },
  {
    id: "invoices",
    label: "Invoices",
    icon: FileText,
  },
] satisfies Array<{
  id: AppSection;
  label: string;
  icon: typeof Users;
}>;

export function Sidebar({
  activeSection,
  onSectionChange,
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">
          <Wrench size={19} />
        </div>

        <div>
          <strong>Moto Workshop</strong>
          <span>Management</span>
        </div>
      </div>

      <nav className="sidebar-nav" aria-label="Main navigation">
        {primaryItems.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            type="button"
            className={`nav-item ${
              activeSection === id ? "nav-item-active" : ""
            }`}
            onClick={() => onSectionChange(id)}
          >
            <Icon size={18} strokeWidth={2} />
            <span>{label}</span>
          </button>
        ))}
      </nav>

      <div className="sidebar-footer">
        <button
          type="button"
          className={`nav-item ${
            activeSection === "settings" ? "nav-item-active" : ""
          }`}
          onClick={() => onSectionChange("settings")}
        >
          <Settings size={18} strokeWidth={2} />
          <span>Settings</span>
        </button>
      </div>
    </aside>
  );
}
// @vitest-environment jsdom

import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  loadServiceVisitWorkspace,
  type ServiceVisitWorkspace,
} from "./features/service/api/serviceVisitApi";
import App from "./App";

const workspace = {
  visit: { id: 31 },
  owner: { id: 7 },
} as ServiceVisitWorkspace;
const dashboardMountMock = vi.hoisted(() => vi.fn());

vi.mock("./features/service/api/serviceVisitApi", async () => {
  const actual = await vi.importActual<typeof import("./features/service/api/serviceVisitApi")>(
    "./features/service/api/serviceVisitApi",
  );
  return { ...actual, loadServiceVisitWorkspace: vi.fn() };
});

vi.mock("./components/AppShell", () => ({
  AppShell: ({
    children,
    onSectionChange,
    onNewServiceVisit,
  }: {
    children: React.ReactNode;
    onSectionChange: (section: string) => void;
    onNewServiceVisit: () => void;
  }) => (
    <main>
      <button type="button" onClick={() => onSectionChange("service")}>Service section</button>
      <button type="button" onClick={() => onSectionChange("dashboard")}>Dashboard section</button>
      <button type="button" onClick={() => onSectionChange("motorcycles")}>Motorcycles section</button>
      <button type="button" onClick={() => onSectionChange("inventory")}>Inventory section</button>
      <button type="button" onClick={() => onSectionChange("invoices")}>Invoices section</button>
      <button type="button" onClick={onNewServiceVisit}>Topbar New Service Visit</button>
      {children}
    </main>
  ),
}));

vi.mock("./features/customers/CustomersPage", () => ({
  CustomersPage: ({ onSelectCustomer }: { onSelectCustomer: (id: number) => void }) => (
    <button type="button" onClick={() => onSelectCustomer(7)}>Open customer 7</button>
  ),
}));

vi.mock("./features/customers/CustomerDetailsPage", () => ({
  CustomerDetailsPage: ({
    customerId,
    onBack,
    onOpenServiceVisit,
  }: {
    customerId: number;
    onBack: () => void;
    onOpenServiceVisit: (workspace: ServiceVisitWorkspace) => void;
  }) => (
    <section>
      <span>Customer details {customerId}</span>
      <button type="button" onClick={onBack}>Back to customers</button>
      <button type="button" onClick={() => onOpenServiceVisit(workspace)}>
        Open workspace
      </button>
    </section>
  ),
}));

vi.mock("./features/service/ServiceVisitPage", () => ({
  ServiceVisitPage: ({
    workspace: current,
    onBack,
    onOpenInvoice,
  }: {
    workspace: ServiceVisitWorkspace;
    onBack: () => void;
    onOpenInvoice: (invoiceId: number) => void;
  }) => (
    <section>
      <span>Real workspace {current.visit.id}</span>
      <button type="button" onClick={onBack}>Back from workspace</button>
      <button type="button" onClick={() => onOpenInvoice(4)}>Open workspace invoice</button>
    </section>
  ),
}));

vi.mock("./features/invoices/InvoicesPage", () => ({
  InvoicesPage: ({ onSelectInvoice, initialStatusFilter }: { onSelectInvoice: (id: number) => void; initialStatusFilter?: string }) => (
    <section><span>Invoices list</span><span>{initialStatusFilter}</span><button type="button" onClick={() => onSelectInvoice(4)}>Open invoice 4</button></section>
  ),
}));

vi.mock("./features/invoices/InvoiceDetailsPage", () => ({
  InvoiceDetailsPage: ({ invoiceId, onBack, onOpenServiceVisit }: {
    invoiceId: number; onBack: () => void; onOpenServiceVisit: (id: number) => void;
  }) => (
    <section><span>Invoice details {invoiceId}</span><button type="button" onClick={onBack}>Back from invoice</button><button type="button" onClick={() => onOpenServiceVisit(31)}>Open invoice visit</button></section>
  ),
}));

vi.mock("./features/motorcycles/MotorcyclesPage", () => ({
  MotorcyclesPage: ({ onSelectMotorcycle }: { onSelectMotorcycle: (id: number) => void }) => (
    <section><span>Motorcycles list</span><button type="button" onClick={() => onSelectMotorcycle(11)}>Open motorcycle 11</button></section>
  ),
}));

vi.mock("./features/motorcycles/MotorcycleDetailsPage", () => ({
  MotorcycleDetailsPage: ({ motorcycleId, onBack, onOpenCustomer, onOpenServiceVisit }: { motorcycleId: number; onBack: () => void; onOpenCustomer: (id: number) => void; onOpenServiceVisit: (workspace: ServiceVisitWorkspace) => void }) => (
    <section><span>Motorcycle details {motorcycleId}</span><button type="button" onClick={onBack}>Back to motorcycles</button><button type="button" onClick={() => onOpenCustomer(7)}>Open motorcycle owner</button><button type="button" onClick={() => onOpenServiceVisit(workspace)}>Open motorcycle workspace</button></section>
  ),
}));

vi.mock("./features/service/directory/ServiceVisitsPage", () => ({
  ServiceVisitsPage: ({
    onOpenServiceVisit,
    initialStatusFilter,
  }: {
    onOpenServiceVisit: (workspace: ServiceVisitWorkspace) => void;
    initialStatusFilter?: string;
  }) => (
    <section>
      <span>Service Visits list</span>
      <span>{initialStatusFilter}</span>
      <button type="button" onClick={() => onOpenServiceVisit(workspace)}>
        Open listed workspace
      </button>
    </section>
  ),
}));

vi.mock("./features/dashboard/DashboardPage", async () => {
  const React = await vi.importActual<typeof import("react")>("react");
  return {
  DashboardPage: ({
    onOpenServiceVisit,
    onOpenInvoice,
    onOpenInventoryItem,
    onShowService,
    onShowInventory,
    onShowInvoices,
  }: {
    onOpenServiceVisit: (workspace: ServiceVisitWorkspace) => void;
    onOpenInvoice: (id: number) => void;
    onOpenInventoryItem: (id: number) => void;
    onShowService: (filter: string) => void;
    onShowInventory: () => void;
    onShowInvoices: (filter: string) => void;
  }) => {
    React.useEffect(() => {
      dashboardMountMock();
    }, []);
    return <section>
      <span>Real Dashboard</span>
      <button type="button" onClick={() => onOpenServiceVisit(workspace)}>Open dashboard visit</button>
      <button type="button" onClick={() => onOpenInvoice(4)}>Open dashboard invoice</button>
      <button type="button" onClick={() => onOpenInventoryItem(19)}>Open dashboard inventory</button>
      <button type="button" onClick={() => onShowService("READY_FOR_PICKUP")}>Dashboard ready card</button>
      <button type="button" onClick={onShowInventory}>Dashboard stock card</button>
      <button type="button" onClick={() => onShowInvoices("ISSUED")}>Dashboard issued card</button>
    </section>;
  },
  };
});

vi.mock("./features/inventory/InventoryPage", () => ({
  InventoryPage: ({ onSelectItem }: { onSelectItem: (id: number) => void }) => (
    <section>
      <span>Inventory list</span>
      <button type="button" onClick={() => onSelectItem(19)}>Open inventory 19</button>
    </section>
  ),
}));

vi.mock("./features/inventory/InventoryItemDetailsPage", () => ({
  InventoryItemDetailsPage: ({
    inventoryItemId,
    onBack,
  }: {
    inventoryItemId: number;
    onBack: () => void;
  }) => (
    <section>
      <span>Inventory details {inventoryItemId}</span>
      <button type="button" onClick={onBack}>Back to inventory</button>
    </section>
  ),
}));

vi.mock("./features/service/new-visit/NewServiceVisitDialog", () => ({
  NewServiceVisitDialog: ({
    open,
    onCreated,
  }: {
    open: boolean;
    onCreated: (workspace: ServiceVisitWorkspace) => void;
  }) => open ? (
    <button type="button" onClick={() => onCreated(workspace)}>Create topbar visit</button>
  ) : null,
}));

describe("App customer workspace navigation", () => {
  beforeEach(() => {
    dashboardMountMock.mockClear();
    vi.mocked(loadServiceVisitWorkspace).mockResolvedValue(workspace);
  });
  afterEach(() => cleanup());

  it("opens a real workspace from customer details and returns to those details", async () => {
    // Arrange
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Open customer 7" }));

    // Act
    await user.click(screen.getByRole("button", { name: "Open workspace" }));
    expect(screen.getByText("Real workspace 31")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back from workspace" }));

    // Assert
    expect(screen.getByText("Customer details 7")).toBeTruthy();
  });

  it("opens a listed Service workspace and returns to the Service list", async () => {
    // Arrange
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Service section" }));

    // Act
    await user.click(screen.getByRole("button", { name: "Open listed workspace" }));
    expect(screen.getByText("Real workspace 31")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back from workspace" }));

    // Assert
    expect(screen.getByText("Service Visits list")).toBeTruthy();
  });

  it("routes a topbar-created visit to its real workspace", async () => {
    // Arrange
    const user = userEvent.setup();
    render(<App />);

    // Act
    await user.click(screen.getByRole("button", { name: "Topbar New Service Visit" }));
    await user.click(screen.getByRole("button", { name: "Create topbar visit" }));

    // Assert
    expect(screen.getByText("Real workspace 31")).toBeTruthy();
  });

  it("preserves motorcycle detail origins through workspace and Customer Details", async () => {
    // Arrange
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Motorcycles section" }));
    await user.click(screen.getByRole("button", { name: "Open motorcycle 11" }));

    // Act / Assert
    await user.click(screen.getByRole("button", { name: "Open motorcycle workspace" }));
    await user.click(screen.getByRole("button", { name: "Back from workspace" }));
    expect(screen.getByText("Motorcycle details 11")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Open motorcycle owner" }));
    expect(screen.getByText("Customer details 7")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back to customers" }));
    expect(screen.getByText("Motorcycle details 11")).toBeTruthy();
  });

  it("opens Inventory details by persisted ID and returns to the Inventory list", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole("button", { name: "Inventory section" }));
    await user.click(screen.getByRole("button", { name: "Open inventory 19" }));
    expect(screen.getByText("Inventory details 19")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Back to inventory" }));
    expect(screen.getByText("Inventory list")).toBeTruthy();
  });

  it("preserves the Service Visit origin when opening and leaving an Invoice", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Service section" }));
    await user.click(screen.getByRole("button", { name: "Open listed workspace" }));
    await user.click(screen.getByRole("button", { name: "Open workspace invoice" }));
    expect(screen.getByText("Invoice details 4")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back from invoice" }));
    expect(screen.getByText("Real workspace 31")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back from workspace" }));
    expect(screen.getByText("Service Visits list")).toBeTruthy();
  });

  it("opens an Invoice-linked Service Visit and Back returns to that Invoice", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Invoices section" }));
    await user.click(screen.getByRole("button", { name: "Open invoice 4" }));
    await user.click(screen.getByRole("button", { name: "Open invoice visit" }));
    expect(await screen.findByText("Real workspace 31")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back from workspace" }));
    expect(screen.getByText("Invoice details 4")).toBeTruthy();
  });

  it("returns dashboard detail links to the Dashboard", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Dashboard section" }));
    expect(dashboardMountMock).toHaveBeenCalledTimes(1);

    await user.click(screen.getByRole("button", { name: "Open dashboard visit" }));
    await user.click(screen.getByRole("button", { name: "Back from workspace" }));
    expect(screen.getByText("Real Dashboard")).toBeTruthy();
    expect(dashboardMountMock).toHaveBeenCalledTimes(2);

    await user.click(screen.getByRole("button", { name: "Open dashboard invoice" }));
    await user.click(screen.getByRole("button", { name: "Open invoice visit" }));
    await user.click(screen.getByRole("button", { name: "Back from workspace" }));
    expect(screen.getByText("Invoice details 4")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Back from invoice" }));
    expect(screen.getByText("Real Dashboard")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Open dashboard inventory" }));
    await user.click(screen.getByRole("button", { name: "Back to inventory" }));
    expect(screen.getByText("Real Dashboard")).toBeTruthy();
    expect(dashboardMountMock).toHaveBeenCalledTimes(4);
  });

  it("routes dashboard cards to their useful filtered lists", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Dashboard section" }));

    await user.click(screen.getByRole("button", { name: "Dashboard ready card" }));
    expect(screen.getByText("READY_FOR_PICKUP")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Dashboard section" }));
    await user.click(screen.getByRole("button", { name: "Dashboard issued card" }));
    expect(screen.getByText("ISSUED")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Dashboard section" }));
    await user.click(screen.getByRole("button", { name: "Dashboard stock card" }));
    expect(screen.getByText("Inventory list")).toBeTruthy();
  });
});

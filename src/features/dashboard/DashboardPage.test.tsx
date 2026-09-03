// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { loadServiceVisitWorkspace, type ServiceVisitWorkspace } from "../service/api/serviceVisitApi";
import { loadDashboard, type DashboardData } from "./api/dashboardApi";
import { DashboardPage } from "./DashboardPage";

vi.mock("./api/dashboardApi", async () => {
  const actual = await vi.importActual<typeof import("./api/dashboardApi")>("./api/dashboardApi");
  return { ...actual, loadDashboard: vi.fn() };
});
vi.mock("../service/api/serviceVisitApi", async () => {
  const actual = await vi.importActual<typeof import("../service/api/serviceVisitApi")>("../service/api/serviceVisitApi");
  return { ...actual, loadServiceVisitWorkspace: vi.fn() };
});

const data: DashboardData = {
  summary: { activeServiceVisits: 4, readyForPickupVisits: 2, customerCount: 18,
    motorcycleCount: 22, lowStockItemCount: 3, negativeStockItemCount: 1,
    issuedInvoiceCountToday: 2, issuedInvoiceValueTodayFils: 21_500 },
  recentServiceVisits: [{ id: 31, customerName: "Ahmad Ali", motorcycle: "Honda CB150R",
    plateNumber: "29-12345", openedAt: 1_725_000_000_000, status: "OPEN", complaint: "Oil leak" }],
  recentInvoices: [{ id: 4, invoiceNumber: "INV-000004", issuedAt: 1_725_000_100_000,
    customerName: "Ahmad Ali", motorcycle: "Honda CB150R", totalFils: 21_500 }],
  inventoryAlerts: [{ id: 9, itemName: "Oil Filter", sku: "FILTER", unitName: "Piece",
    quantityScale: 1, currentQuantity: -2, minimumStockQuantity: 3, negativeStock: true }],
};
const workspace = { visit: { id: 31 } } as ServiceVisitWorkspace;
const loadMock = vi.mocked(loadDashboard);
const workspaceMock = vi.mocked(loadServiceVisitWorkspace);

describe("DashboardPage", () => {
  beforeEach(() => { loadMock.mockReset(); workspaceMock.mockReset(); workspaceMock.mockResolvedValue(workspace); });
  afterEach(() => cleanup());

  it("shows loading then real metrics and actionable bounded rows/cards", async () => {
    // # Arrange
    let resolveDashboard!: (value: DashboardData) => void;
    loadMock.mockReturnValue(new Promise((resolve) => { resolveDashboard = resolve; }));
    const user = userEvent.setup();
    const props = { onOpenServiceVisit: vi.fn(), onOpenInvoice: vi.fn(), onOpenInventoryItem: vi.fn(),
      onShowService: vi.fn(), onShowInventory: vi.fn(), onShowInvoices: vi.fn() };
    render(<DashboardPage {...props} />);
    expect(screen.getByText("Loading Dashboard...")).toBeTruthy();

    // # Act
    resolveDashboard(data);
    expect(await screen.findAllByText("Ahmad Ali")).toHaveLength(2);
    await user.click(screen.getByRole("button", { name: "Show active Service Visits" }));
    await user.click(screen.getByRole("button", { name: "Show ready Service Visits" }));
    await user.click(screen.getByRole("button", { name: "Show low-stock Inventory" }));
    await user.click(screen.getByRole("button", { name: "Show issued Invoices" }));
    await user.click(screen.getByRole("button", { name: "Open recent Service Visit 31" }));
    await user.click(screen.getByRole("button", { name: "Open recent Invoice INV-000004" }));
    await user.click(screen.getByRole("button", { name: "Open Inventory Item 9" }));

    // # Assert
    expect(screen.getAllByText("21.500 JD")).toHaveLength(2);
    expect(screen.getByText("1 negative")).toBeTruthy();
    expect(props.onShowService).toHaveBeenNthCalledWith(1, "ACTIVE");
    expect(props.onShowService).toHaveBeenNthCalledWith(2, "READY_FOR_PICKUP");
    expect(props.onShowInventory).toHaveBeenCalled();
    expect(props.onShowInvoices).toHaveBeenCalledWith("ISSUED");
    expect(workspaceMock).toHaveBeenCalledWith(31);
    await waitFor(() => expect(props.onOpenServiceVisit).toHaveBeenCalledWith(workspace));
    expect(props.onOpenInvoice).toHaveBeenCalledWith(4);
    expect(props.onOpenInventoryItem).toHaveBeenCalledWith(9);
  });

  it("shows safe failure and a useful zero/empty state", async () => {
    // # Arrange / Act
    loadMock.mockRejectedValueOnce(new Error("sqlite details"));
    const props = { onOpenServiceVisit: vi.fn(), onOpenInvoice: vi.fn(), onOpenInventoryItem: vi.fn(),
      onShowService: vi.fn(), onShowInventory: vi.fn(), onShowInvoices: vi.fn() };
    const { rerender } = render(<DashboardPage {...props} />);
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.queryByText("sqlite details")).toBeNull();
    loadMock.mockResolvedValueOnce({ ...data, summary: { activeServiceVisits: 0, readyForPickupVisits: 0,
      customerCount: 0, motorcycleCount: 0, lowStockItemCount: 0, negativeStockItemCount: 0,
      issuedInvoiceCountToday: 0, issuedInvoiceValueTodayFils: 0 }, recentServiceVisits: [],
      recentInvoices: [], inventoryAlerts: [] });
    rerender(<DashboardPage key="empty" {...props} />);

    // # Assert
    expect(await screen.findByText("No recent Service Visits")).toBeTruthy();
    expect(screen.getByText("No issued invoices yet")).toBeTruthy();
    expect(screen.getByText("No Inventory alerts")).toBeTruthy();
  });
});

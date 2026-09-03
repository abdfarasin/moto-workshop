// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  adjustInventoryStock,
  loadInventoryItemDetails,
  updateInventoryItem,
} from "./api/inventoryApi";
import type { InventoryItemDetails } from "./api/inventoryApi";
import { InventoryItemDetailsPage } from "./InventoryItemDetailsPage";

vi.mock("./api/inventoryApi", async () => ({
  ...(await vi.importActual("./api/inventoryApi")),
  loadInventoryItemDetails: vi.fn(),
  adjustInventoryStock: vi.fn(),
  updateInventoryItem: vi.fn(),
}));

const loadMock = vi.mocked(loadInventoryItemDetails);
const adjustMock = vi.mocked(adjustInventoryStock);
const updateMock = vi.mocked(updateInventoryItem);
const details: InventoryItemDetails = {
  id: 7,
  name: "Oil Filter",
  sku: "FILTER-1",
  unitId: 1,
  unitName: "Piece",
  quantityScale: 1,
  defaultPurchasePriceFils: 3000,
  defaultSellingPriceFils: 4500,
  minimumStockQuantity: 3,
  notes: null,
  currentQuantity: -2,
  lowStock: true,
  movements: [
    {
      id: 9,
      movementType: "ADJUSTMENT_OUT",
      quantityDelta: -7,
      notes: "Count",
      serviceVisitPartId: null,
      createdAt: 2000,
    },
  ],
};

describe("InventoryItemDetailsPage", () => {
  beforeEach(() => {
    loadMock.mockReset();
    adjustMock.mockReset();
    updateMock.mockReset();
    loadMock.mockResolvedValue(details);
    adjustMock.mockResolvedValue({ ...details, currentQuantity: 3 });
    updateMock.mockResolvedValue(details);
  });

  afterEach(() => cleanup());

  it("loads details/history and applies an auditable negative adjustment then refreshes", async () => {
    const user = userEvent.setup();
    render(<InventoryItemDetailsPage inventoryItemId={7} onBack={vi.fn()} />);

    expect(await screen.findByText(/FILTER-1/)).toBeTruthy();
    expect(screen.getByText("ADJUSTMENT OUT")).toBeTruthy();
    expect(screen.getByText(/Stock is negative/)).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Adjust Stock" }));
    await user.type(screen.getByLabelText("Quantity delta"), "-5");
    await user.type(screen.getByLabelText("Reason"), "Correction");
    await user.click(screen.getByRole("button", { name: "Save Adjustment" }));

    await waitFor(() =>
      expect(adjustMock).toHaveBeenCalledWith(
        expect.objectContaining({
          inventoryItemId: 7,
          quantityDelta: -5,
          notes: "Correction",
        }),
      ),
    );
    expect(loadMock).toHaveBeenCalledTimes(2);
  });

  it("edits metadata with exact fils and refreshes without changing quantity directly", async () => {
    const user = userEvent.setup();
    render(<InventoryItemDetailsPage inventoryItemId={7} onBack={vi.fn()} />);
    await screen.findByText(/FILTER-1/);

    await user.click(screen.getByRole("button", { name: "Edit Item" }));
    const dialog = screen.getByRole("dialog", { name: "Edit Inventory Item" });
    expect(dialog.textContent).toContain("Unit remains Piece");

    const name = screen.getByLabelText("Name");
    await user.clear(name);
    await user.type(name, "Premium Oil Filter");
    const selling = screen.getByLabelText("Selling price (JD)");
    await user.clear(selling);
    await user.type(selling, "5.125");
    await user.click(screen.getByRole("button", { name: "Save Item" }));

    await waitFor(() =>
      expect(updateMock).toHaveBeenCalledWith(
        expect.objectContaining({
          inventoryItemId: 7,
          name: "Premium Oil Filter",
          defaultSellingPriceFils: 5125,
        }),
      ),
    );
    expect(updateMock.mock.calls[0][0]).not.toHaveProperty("currentQuantity");
    expect(loadMock).toHaveBeenCalledTimes(2);
  });
});

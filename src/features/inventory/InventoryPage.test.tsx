// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  createInventoryItem,
  listInventoryUnits,
  searchInventoryItems,
} from "./api/inventoryApi";
import type { InventoryItemSummary } from "./api/inventoryApi";
import { InventoryPage } from "./InventoryPage";

vi.mock("./api/inventoryApi", async () => ({
  ...(await vi.importActual("./api/inventoryApi")),
  searchInventoryItems: vi.fn(),
  listInventoryUnits: vi.fn(),
  createInventoryItem: vi.fn(),
}));

const searchMock = vi.mocked(searchInventoryItems);
const listUnitsMock = vi.mocked(listInventoryUnits);
const createMock = vi.mocked(createInventoryItem);
const item: InventoryItemSummary = {
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
  currentQuantity: 2,
  lowStock: true,
};

describe("InventoryPage", () => {
  beforeEach(() => {
    searchMock.mockReset();
    listUnitsMock.mockReset();
    createMock.mockReset();
    searchMock.mockResolvedValue([item]);
    listUnitsMock.mockResolvedValue([{ id: 1, name: "Piece", quantityScale: 1 }]);
    createMock.mockResolvedValue({ ...item, movements: [] });
  });

  afterEach(() => cleanup());

  it("loads bounded real rows, searches in SQLite, and opens by ID with keyboard", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<InventoryPage onSelectItem={onSelect} />);

    expect(await screen.findByText("Oil Filter")).toBeTruthy();
    expect(searchMock).toHaveBeenCalledWith({ query: "", limit: 50 });

    await user.type(screen.getByLabelText("Search Inventory"), "FILTER-1");
    await user.click(screen.getByRole("button", { name: "Search" }));
    expect(searchMock).toHaveBeenLastCalledWith({ query: "FILTER-1", limit: 50 });

    screen.getByRole("button", { name: "Open Inventory Item 7" }).focus();
    await user.keyboard("{Enter}");
    expect(onSelect).toHaveBeenCalledWith(7);
  });

  it("creates an item with exact fils/opening stock and reloads from SQLite", async () => {
    const user = userEvent.setup();
    render(<InventoryPage onSelectItem={vi.fn()} />);
    await screen.findByText("Oil Filter");

    await user.click(screen.getByRole("button", { name: "New Inventory Item" }));
    await screen.findByRole("dialog", { name: "New Inventory Item" });
    await user.type(screen.getByLabelText("Item name"), "Brake Pad");
    await user.type(screen.getByLabelText("SKU"), "PAD-1");
    await user.type(screen.getByLabelText("Selling price"), "12.500");
    const opening = screen.getByLabelText("Opening stock");
    await user.clear(opening);
    await user.type(opening, "2");
    await user.click(screen.getByRole("button", { name: "Create Item" }));

    await waitFor(() =>
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          name: "Brake Pad",
          sku: "PAD-1",
          unitId: 1,
          defaultSellingPriceFils: 12500,
          openingQuantity: 2,
        }),
      ),
    );
    await waitFor(() => expect(searchMock).toHaveBeenCalledTimes(2));
  });

  it("shows empty and safe error states", async () => {
    searchMock.mockResolvedValueOnce([]);
    const { unmount } = render(<InventoryPage onSelectItem={vi.fn()} />);
    expect(await screen.findByText("No Inventory Items found")).toBeTruthy();
    unmount();

    searchMock.mockRejectedValueOnce(new Error("transport details must not leak"));
    render(<InventoryPage onSelectItem={vi.fn()} />);
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Inventory could not be loaded",
    );
  });
});

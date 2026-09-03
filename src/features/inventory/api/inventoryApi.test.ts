import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  adjustInventoryStock,
  createInventoryItem,
  InventoryCommandError,
  listInventoryUnits,
  loadInventoryItemDetails,
  searchInventoryItems,
  UnexpectedInventoryApiError,
  updateInventoryItem,
} from "./inventoryApi";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const invokeMock = vi.mocked(invoke);

describe("inventory API", () => {
  beforeEach(() => invokeMock.mockReset());

  it("uses exact feature-local Tauri commands and input wrappers", async () => {
    invokeMock.mockResolvedValue({ id: 7 });
    const create = {
      name: "Oil",
      sku: null,
      unitId: 1,
      defaultPurchasePriceFils: null,
      defaultSellingPriceFils: 4500,
      minimumStockQuantity: 0,
      notes: null,
      openingQuantity: 5,
      createdAt: 1,
    };
    const update = {
      inventoryItemId: 7,
      name: "Oil",
      sku: null,
      defaultPurchasePriceFils: null,
      defaultSellingPriceFils: 5000,
      minimumStockQuantity: 0,
      notes: null,
      updatedAt: 2,
    };
    const adjustment = {
      inventoryItemId: 7,
      quantityDelta: -2,
      notes: "Count",
      createdAt: 3,
    };

    await searchInventoryItems({ query: "Oil", limit: 50 });
    await loadInventoryItemDetails(7);
    await listInventoryUnits();
    await createInventoryItem(create);
    await updateInventoryItem(update);
    await adjustInventoryStock(adjustment);

    expect(invokeMock).toHaveBeenNthCalledWith(1, "search_inventory_items", {
      input: { query: "Oil", limit: 50 },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "load_inventory_item_details", {
      input: { inventoryItemId: 7 },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "list_inventory_units");
    expect(invokeMock).toHaveBeenNthCalledWith(4, "create_inventory_item", { input: create });
    expect(invokeMock).toHaveBeenNthCalledWith(5, "update_inventory_item", { input: update });
    expect(invokeMock).toHaveBeenNthCalledWith(6, "adjust_inventory_stock", {
      input: adjustment,
    });
  });

  it("preserves backend categories and distinguishes unexpected failures", async () => {
    invokeMock.mockRejectedValueOnce({
      category: "inventorySkuAlreadyExists",
      message: "An Inventory Item with this SKU already exists.",
    });

    const known = await searchInventoryItems({ query: "", limit: 50 }).catch(
      (error: unknown) => error,
    );
    expect(known).toBeInstanceOf(InventoryCommandError);
    expect(known).toMatchObject({
      category: "inventorySkuAlreadyExists",
      message: "An Inventory Item with this SKU already exists.",
    });

    invokeMock.mockRejectedValueOnce("transport offline");
    const unexpected = await listInventoryUnits().catch((error: unknown) => error);
    expect(unexpected).toBeInstanceOf(UnexpectedInventoryApiError);
    expect(unexpected).toMatchObject({ cause: "transport offline" });
  });
});

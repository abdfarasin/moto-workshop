import { invoke } from "@tauri-apps/api/core";

import type {
  AdjustInventoryStockInput,
  CreateInventoryItemInput,
  InventoryCommandErrorCategory,
  InventoryItemDetails,
  InventoryItemSummary,
  InventoryUnit,
  SearchInventoryItemsInput,
  UpdateInventoryItemInput,
} from "./inventoryApi.types";

export class InventoryCommandError extends Error {
  readonly category: InventoryCommandErrorCategory;

  constructor(payload: { category: InventoryCommandErrorCategory; message: string }) {
    super(payload.message);
    this.name = "InventoryCommandError";
    this.category = payload.category;
  }
}

export class UnexpectedInventoryApiError extends Error {
  readonly cause: unknown;

  constructor(cause: unknown) {
    super("The Inventory command failed unexpectedly.");
    this.name = "UnexpectedInventoryApiError";
    this.cause = cause;
  }
}

const commandErrorCategories: InventoryCommandErrorCategory[] = [
  "inventoryItemNotFound",
  "inventoryUnitNotFound",
  "inventorySkuAlreadyExists",
  "validationError",
  "databaseError",
];

async function invokeInventory<T>(
  command: string,
  argumentsValue?: Record<string, unknown>,
): Promise<T> {
  try {
    return argumentsValue === undefined
      ? await invoke<T>(command)
      : await invoke<T>(command, argumentsValue);
  } catch (error: unknown) {
    if (
      typeof error === "object" &&
      error !== null &&
      typeof (error as { message?: unknown }).message === "string" &&
      commandErrorCategories.includes(
        (error as { category: InventoryCommandErrorCategory }).category,
      )
    ) {
      throw new InventoryCommandError(
        error as { category: InventoryCommandErrorCategory; message: string },
      );
    }
    throw new UnexpectedInventoryApiError(error);
  }
}

export function searchInventoryItems(input: SearchInventoryItemsInput) {
  return invokeInventory<InventoryItemSummary[]>("search_inventory_items", { input });
}

export function loadInventoryItemDetails(inventoryItemId: number) {
  return invokeInventory<InventoryItemDetails>("load_inventory_item_details", {
    input: { inventoryItemId },
  });
}

export function listInventoryUnits() {
  return invokeInventory<InventoryUnit[]>("list_inventory_units");
}

export function createInventoryItem(input: CreateInventoryItemInput) {
  return invokeInventory<InventoryItemDetails>("create_inventory_item", { input });
}

export function updateInventoryItem(input: UpdateInventoryItemInput) {
  return invokeInventory<InventoryItemDetails>("update_inventory_item", { input });
}

export function adjustInventoryStock(input: AdjustInventoryStockInput) {
  return invokeInventory<InventoryItemDetails>("adjust_inventory_stock", { input });
}

export type * from "./inventoryApi.types";

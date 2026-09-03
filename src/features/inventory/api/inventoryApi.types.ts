export type StockMovementType =
  | "OPENING_STOCK"
  | "PURCHASE"
  | "ADJUSTMENT_IN"
  | "ADJUSTMENT_OUT"
  | "SERVICE_USAGE"
  | "SERVICE_USAGE_REVERSAL";

export interface InventoryItemSummary {
  id: number;
  name: string;
  sku: string | null;
  unitId: number;
  unitName: string;
  quantityScale: number;
  defaultPurchasePriceFils: number | null;
  defaultSellingPriceFils: number;
  minimumStockQuantity: number;
  notes: string | null;
  currentQuantity: number;
  lowStock: boolean;
}

export interface StockMovement {
  id: number;
  movementType: StockMovementType;
  quantityDelta: number;
  notes: string | null;
  serviceVisitPartId: number | null;
  createdAt: number;
}

export interface InventoryItemDetails extends InventoryItemSummary {
  movements: StockMovement[];
}

export interface InventoryUnit {
  id: number;
  name: string;
  quantityScale: number;
}

export interface SearchInventoryItemsInput {
  query: string;
  limit?: number;
}

export interface CreateInventoryItemInput {
  name: string;
  sku: string | null;
  unitId: number;
  defaultPurchasePriceFils: number | null;
  defaultSellingPriceFils: number;
  minimumStockQuantity: number;
  notes: string | null;
  openingQuantity: number;
  createdAt: number;
}

export interface UpdateInventoryItemInput {
  inventoryItemId: number;
  name: string;
  sku: string | null;
  defaultPurchasePriceFils: number | null;
  defaultSellingPriceFils: number;
  minimumStockQuantity: number;
  notes: string | null;
  updatedAt: number;
}

export interface AdjustInventoryStockInput {
  inventoryItemId: number;
  quantityDelta: number;
  notes: string | null;
  createdAt: number;
}

export type InventoryCommandErrorCategory =
  | "inventoryItemNotFound"
  | "inventoryUnitNotFound"
  | "inventorySkuAlreadyExists"
  | "validationError"
  | "databaseError";

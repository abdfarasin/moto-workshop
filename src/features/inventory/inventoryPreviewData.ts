export type InventoryItemPreview = {
  id: number;
  name: string;
  unitName: string;
  quantityScale: 1 | 10 | 100 | 1000;
  defaultSellingPriceFils: number;
  currentQuantity: number;
};

export const previewInventoryItems: InventoryItemPreview[] = [
  {
    id: 1,
    name: "Oil Filter",
    unitName: "Piece",
    quantityScale: 1,
    defaultSellingPriceFils: 4_500,
    currentQuantity: 12,
  },
  {
    id: 2,
    name: "10W40 Engine Oil",
    unitName: "Liter",
    quantityScale: 1000,
    defaultSellingPriceFils: 6_000,
    currentQuantity: 8_500,
  },
  {
    id: 3,
    name: "Front Brake Pads",
    unitName: "Piece",
    quantityScale: 1,
    defaultSellingPriceFils: 9_000,
    currentQuantity: 4,
  },
  {
    id: 4,
    name: "Spark Plug",
    unitName: "Piece",
    quantityScale: 1,
    defaultSellingPriceFils: 3_250,
    currentQuantity: 18,
  },
];
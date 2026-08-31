export function isPartQuantityAboveStock(
  requestedQuantity: number,
  currentQuantity: number,
): boolean {
  return requestedQuantity > currentQuantity;
}
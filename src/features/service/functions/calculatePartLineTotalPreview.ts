export function calculatePartLineTotalPreview(
  quantity: number,
  quantityScale: number,
  unitPriceFils: number,
): number {
  return Math.floor(
    (quantity * unitPriceFils + quantityScale / 2) / quantityScale,
  );
}
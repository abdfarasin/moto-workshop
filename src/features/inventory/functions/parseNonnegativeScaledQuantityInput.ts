import { parseScaledQuantityInput } from "../../service/functions/parseScaledQuantityInput";

export function parseNonnegativeScaledQuantityInput(
  input: string,
  quantityScale: number,
): number | null {
  const trimmed = input.trim();
  const precision = precisionForScale(quantityScale);
  if (precision === null) {
    return null;
  }

  const zeroPattern = precision === 0
    ? /^0$/
    : new RegExp(`^0(?:\\.0{1,${precision}})?$`);
  if (zeroPattern.test(trimmed)) {
    return 0;
  }

  return parseScaledQuantityInput(trimmed, quantityScale);
}

function precisionForScale(quantityScale: number): number | null {
  switch (quantityScale) {
    case 1:
      return 0;
    case 10:
      return 1;
    case 100:
      return 2;
    case 1_000:
      return 3;
    default:
      return null;
  }
}

const MAX_SCALED_QUANTITY = 1_000_000_000n;
export function parseScaledQuantityInput(
  input: string,
  quantityScale: number,
): number | null {
  const precision = precisionForScale(quantityScale);

  if (precision === null) {
    return null;
  }

  const trimmed = input.trim();

  if (trimmed === "") {
    return null;
  }

  const pattern =
    precision === 0
      ? /^\d+$/
      : new RegExp(`^\\d+(?:\\.\\d{1,${precision}})?$`);

  if (!pattern.test(trimmed)) {
    return null;
  }

  const [wholePart, fractionalPart = ""] = trimmed.split(".");

  const whole = BigInt(wholePart);
  const fraction =
    precision === 0
      ? 0n
      : BigInt(fractionalPart.padEnd(precision, "0"));

  const scaledQuantity =
    whole * BigInt(quantityScale) + fraction;

  if (scaledQuantity <= 0n) {
    return null;
  }

  if (scaledQuantity > MAX_SCALED_QUANTITY) {
    return null;
  }

  return Number(scaledQuantity);
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
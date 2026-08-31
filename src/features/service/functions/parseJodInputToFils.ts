const MAX_UNIT_PRICE_FILS = 1_000_000_000;

export function parseJodInputToFils(input: string): number | null {
  const trimmed = input.trim();

  if (trimmed === "") {
    return null;
  }

  if (!/^\d+(?:\.\d{1,3})?$/.test(trimmed)) {
    return null;
  }

  const [wholePart, fractionalPart = ""] = trimmed.split(".");

  const wholeFils = BigInt(wholePart) * 1_000n;
  const fractionalFils = BigInt(fractionalPart.padEnd(3, "0"));

  const totalFils = wholeFils + fractionalFils;

  if (totalFils > BigInt(MAX_UNIT_PRICE_FILS)) {
    return null;
  }

  return Number(totalFils);
}
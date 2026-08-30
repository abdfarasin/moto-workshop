import type { ServicePartPreview } from "../../customers/customerPreviewData";

export function calculatePartsTotal(
  parts: ServicePartPreview[],
): number {
  return parts.reduce(
    (total, part) => total + part.lineTotalFils,
    0,
  );
}
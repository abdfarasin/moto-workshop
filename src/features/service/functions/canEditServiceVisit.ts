import type { ServiceHistoryPreview } from "../../customers/customerPreviewData";

export function canEditServiceVisit(
  visit: ServiceHistoryPreview,
): boolean {
  return visit.status === "OPEN";
}
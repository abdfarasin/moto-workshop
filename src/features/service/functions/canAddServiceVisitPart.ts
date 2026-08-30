import type { ServiceHistoryPreview } from "../../customers/customerPreviewData";

export function canAddServiceVisitPart(
  visit: ServiceHistoryPreview,
): boolean {
  return (
    visit.status === "OPEN" ||
    visit.status === "READY_FOR_PICKUP"
  );
}
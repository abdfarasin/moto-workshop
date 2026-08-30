import type { ServiceHistoryPreview } from "../../customers/customerPreviewData";

export function formatServiceVisitStatus(
  status: ServiceHistoryPreview["status"],
): string {
  return status.replace(/_/g, " ");
}
import type { ServiceVisitStatus } from "../api/serviceVisitApi";

export function formatServiceVisitStatus(
  status: ServiceVisitStatus,
): string {
  return status.replace(/_/g, " ");
}

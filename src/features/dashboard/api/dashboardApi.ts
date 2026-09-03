import { invoke } from "@tauri-apps/api/core";

import { localDayRange } from "../functions/localDayRange";
import type {
  DashboardCommandErrorCategory,
  DashboardCommandErrorPayload,
  DashboardData,
} from "./dashboardApi.types";

const categories: readonly DashboardCommandErrorCategory[] = ["validationError", "databaseError"];

export class DashboardCommandError extends Error {
  readonly category: DashboardCommandErrorCategory;
  constructor(payload: DashboardCommandErrorPayload) {
    super(payload.message);
    this.name = "DashboardCommandError";
    this.category = payload.category;
  }
}

export class UnexpectedDashboardApiError extends Error {
  readonly cause: unknown;
  constructor(cause: unknown) {
    super("The Dashboard command failed unexpectedly.");
    this.name = "UnexpectedDashboardApiError";
    this.cause = cause;
  }
}

function isPayload(error: unknown): error is DashboardCommandErrorPayload {
  if (typeof error !== "object" || error === null) return false;
  const candidate = error as Partial<DashboardCommandErrorPayload>;
  return typeof candidate.message === "string" && typeof candidate.category === "string" &&
    categories.includes(candidate.category as DashboardCommandErrorCategory);
}

export async function loadDashboard(now: Date = new Date()): Promise<DashboardData> {
  const input = localDayRange(now);
  try {
    return await invoke<DashboardData>("load_dashboard", { input });
  } catch (error: unknown) {
    if (isPayload(error)) throw new DashboardCommandError(error);
    throw new UnexpectedDashboardApiError(error);
  }
}

export type {
  DashboardData,
  DashboardInventoryAlert,
  DashboardInvoice,
  DashboardServiceVisit,
  DashboardSummary,
} from "./dashboardApi.types";

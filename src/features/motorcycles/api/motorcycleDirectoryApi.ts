import { invoke } from "@tauri-apps/api/core";

import type {
  MotorcycleDirectoryCommandErrorPayload,
  MotorcycleDetails,
  MotorcycleDirectoryEntry,
  SearchMotorcycleDirectoryInput,
} from "./motorcycleDirectoryApi.types";

export class MotorcycleDirectoryCommandError extends Error {
  readonly category: MotorcycleDirectoryCommandErrorPayload["category"];
  constructor(payload: MotorcycleDirectoryCommandErrorPayload) {
    super(payload.message);
    this.name = "MotorcycleDirectoryCommandError";
    this.category = payload.category;
  }
}

export class UnexpectedMotorcycleDirectoryApiError extends Error {
  readonly cause: unknown;
  constructor(cause: unknown) {
    super("The Motorcycle command failed unexpectedly.");
    this.name = "UnexpectedMotorcycleDirectoryApiError";
    this.cause = cause;
  }
}

function isKnownError(error: unknown): error is { category: "motorcycleNotFound" | "databaseError"; message: string } {
  if (typeof error !== "object" || error === null) return false;
  const candidate = error as { category?: unknown; message?: unknown };
  return (
    (candidate.category === "motorcycleNotFound" || candidate.category === "databaseError") &&
    typeof candidate.message === "string"
  );
}

async function invokeMotorcycleCommand<T>(command: string, input: object): Promise<T> {
  try {
    return await invoke<T>(command, { input });
  } catch (error: unknown) {
    if (isKnownError(error)) throw new MotorcycleDirectoryCommandError(error);
    throw new UnexpectedMotorcycleDirectoryApiError(error);
  }
}

export function searchMotorcycleDirectory(
  input: SearchMotorcycleDirectoryInput,
): Promise<MotorcycleDirectoryEntry[]> {
  return invokeMotorcycleCommand("search_motorcycle_directory", input);
}

export function loadMotorcycleDetails(motorcycleId: number): Promise<MotorcycleDetails> {
  return invokeMotorcycleCommand("load_motorcycle_details", { motorcycleId });
}

export type {
  MotorcycleDetails,
  MotorcycleDirectoryCommandErrorCategory,
  MotorcycleDirectoryCommandErrorPayload,
  MotorcycleDirectoryEntry,
  MotorcycleServiceHistoryEntry,
  SearchMotorcycleDirectoryInput,
} from "./motorcycleDirectoryApi.types";

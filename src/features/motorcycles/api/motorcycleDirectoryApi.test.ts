import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, expectTypeOf, it, vi } from "vitest";

import {
  loadMotorcycleDetails,
  MotorcycleDirectoryCommandError,
  searchMotorcycleDirectory,
  UnexpectedMotorcycleDirectoryApiError,
} from "./motorcycleDirectoryApi";
import type {
  MotorcycleDetails,
  MotorcycleDirectoryEntry,
} from "./motorcycleDirectoryApi.types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

describe("Motorcycle directory API", () => {
  beforeEach(() => invokeMock.mockReset());

  it("uses exact command input wrappers and keeps results strongly typed", async () => {
    // Arrange
    const entries = [{ id: 11 }] as MotorcycleDirectoryEntry[];
    const details = { id: 11, serviceHistory: [] } as unknown as MotorcycleDetails;
    invokeMock.mockResolvedValueOnce(entries).mockResolvedValueOnce(details);

    // Act
    const listed = await searchMotorcycleDirectory({ query: "Honda", limit: 50 });
    const loaded = await loadMotorcycleDetails(11);

    // Assert
    expect(invokeMock).toHaveBeenNthCalledWith(1, "search_motorcycle_directory", {
      input: { query: "Honda", limit: 50 },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "load_motorcycle_details", {
      input: { motorcycleId: 11 },
    });
    expectTypeOf(listed).toEqualTypeOf<MotorcycleDirectoryEntry[]>();
    expectTypeOf(loaded).toEqualTypeOf<MotorcycleDetails>();
  });

  it("preserves known command errors and distinguishes unexpected transport failures", async () => {
    // Arrange / Act
    invokeMock.mockRejectedValueOnce({ category: "motorcycleNotFound", message: "Missing." });
    const known = await loadMotorcycleDetails(999).catch((error: unknown) => error);
    const transport = new Error("IPC closed");
    invokeMock.mockRejectedValueOnce(transport);
    const unexpected = await searchMotorcycleDirectory({ query: "", limit: 50 }).catch((error: unknown) => error);

    // Assert
    expect(known).toBeInstanceOf(MotorcycleDirectoryCommandError);
    expect(known).toMatchObject({ category: "motorcycleNotFound", message: "Missing." });
    expect(unexpected).toBeInstanceOf(UnexpectedMotorcycleDirectoryApiError);
    expect(unexpected).toMatchObject({ cause: transport });
  });
});

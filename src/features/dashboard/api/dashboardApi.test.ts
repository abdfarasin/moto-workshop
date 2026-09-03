import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { DashboardCommandError, loadDashboard, UnexpectedDashboardApiError } from "./dashboardApi";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const invokeMock = vi.mocked(invoke);

describe("dashboard API", () => {
  beforeEach(() => invokeMock.mockReset());

  it("uses the exact command and explicit local day range", async () => {
    // # Arrange
    invokeMock.mockResolvedValue({ summary: {} });
    const now = new Date(2026, 7, 31, 15, 45, 12);

    // # Act
    await loadDashboard(now);

    // # Assert
    expect(invokeMock).toHaveBeenCalledWith("load_dashboard", {
      input: {
        dayStartMs: new Date(2026, 7, 31).getTime(),
        dayEndMs: new Date(2026, 8, 1).getTime(),
      },
    });
  });

  it("preserves typed command errors and distinguishes unexpected failures", async () => {
    // # Arrange / Act
    invokeMock.mockRejectedValueOnce({ category: "validationError", message: "Bad day." });
    const known = await loadDashboard(new Date()).catch((error) => error);
    invokeMock.mockRejectedValueOnce("offline");
    const unexpected = await loadDashboard(new Date()).catch((error) => error);

    // # Assert
    expect(known).toBeInstanceOf(DashboardCommandError);
    expect(known).toMatchObject({ category: "validationError", message: "Bad day." });
    expect(unexpected).toBeInstanceOf(UnexpectedDashboardApiError);
    expect(unexpected).toMatchObject({ cause: "offline" });
  });
});

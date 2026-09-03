import { describe, expect, it } from "vitest";

import { localDayRange } from "./localDayRange";

describe("localDayRange", () => {
  it("uses local calendar midnight and rolls month/year boundaries safely", () => {
    // # Arrange
    const now = new Date(2026, 11, 31, 23, 59, 59, 999);

    // # Act
    const range = localDayRange(now);

    // # Assert
    expect(range.dayStartMs).toBe(new Date(2026, 11, 31).getTime());
    expect(range.dayEndMs).toBe(new Date(2027, 0, 1).getTime());
    expect(range.dayStartMs).toBeLessThan(now.getTime());
    expect(range.dayEndMs).toBeGreaterThan(now.getTime());
  });
});

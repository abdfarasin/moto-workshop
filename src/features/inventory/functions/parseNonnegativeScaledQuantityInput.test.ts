import { describe, expect, it } from "vitest";

import { parseNonnegativeScaledQuantityInput } from "./parseNonnegativeScaledQuantityInput";

describe("parseNonnegativeScaledQuantityInput", () => {
  it("accepts exact zero representations supported by the Unit scale", () => {
    expect(parseNonnegativeScaledQuantityInput("0", 1)).toBe(0);
    expect(parseNonnegativeScaledQuantityInput("0.000", 1_000)).toBe(0);
    expect(parseNonnegativeScaledQuantityInput("0.00", 100)).toBe(0);
  });

  it("reuses positive scaled parsing and rejects excess precision", () => {
    expect(parseNonnegativeScaledQuantityInput("3.750", 1_000)).toBe(3_750);
    expect(parseNonnegativeScaledQuantityInput("0.0000", 1_000)).toBeNull();
    expect(parseNonnegativeScaledQuantityInput("-1", 1)).toBeNull();
  });
});

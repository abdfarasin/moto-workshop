import { describe, expect, it } from "vitest";

import { calculatePartLineTotalPreview } from "./calculatePartLineTotalPreview";

describe("calculatePartLineTotalPreview", () => {
  it("calculates a whole-piece line total", () => {
    // # Arrange
    const quantity = 2;
    const quantityScale = 1;
    const unitPriceFils = 4_500;

    // # Act
    const result = calculatePartLineTotalPreview(
      quantity,
      quantityScale,
      unitPriceFils,
    );

    // # Assert
    expect(result).toBe(9_000);
  });

  it("calculates a scaled liter line total", () => {
    // # Arrange
    const quantity = 2_500;
    const quantityScale = 1_000;
    const unitPriceFils = 7_000;

    // # Act
    const result = calculatePartLineTotalPreview(
      quantity,
      quantityScale,
      unitPriceFils,
    );

    // # Assert
    expect(result).toBe(17_500);
  });

  it("uses the same half-up rounding rule as Rust", () => {
    // # Arrange
    const quantity = 1;
    const quantityScale = 2;
    const unitPriceFils = 1;

    // # Act
    const result = calculatePartLineTotalPreview(
      quantity,
      quantityScale,
      unitPriceFils,
    );

    // # Assert
    expect(result).toBe(1);
  });
});
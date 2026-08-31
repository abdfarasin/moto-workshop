import { describe, expect, it } from "vitest";

import { parseScaledQuantityInput } from "./parseScaledQuantityInput.ts";

describe("parseScaledQuantityInput", () => {
  it("parses whole-piece quantities", () => {
    // # Arrange
    const input = "3";

    // # Act
    const result = parseScaledQuantityInput(input, 1);

    // # Assert
    expect(result).toBe(3);
  });

  it("parses liter quantities into scaled integers", () => {
    // # Arrange
    const input = "2.500";

    // # Act
    const result = parseScaledQuantityInput(input, 1_000);

    // # Assert
    expect(result).toBe(2_500);
  });

  it("pads fractional digits to the unit scale", () => {
    // # Arrange
    const input = "2.5";

    // # Act
    const result = parseScaledQuantityInput(input, 1_000);

    // # Assert
    expect(result).toBe(2_500);
  });

  it("rejects fractional pieces", () => {
    // # Arrange
    const input = "1.5";

    // # Act
    const result = parseScaledQuantityInput(input, 1);

    // # Assert
    expect(result).toBeNull();
  });

  it("rejects more precision than the unit supports", () => {
    // # Arrange
    const input = "1.0001";

    // # Act
    const result = parseScaledQuantityInput(input, 1_000);

    // # Assert
    expect(result).toBeNull();
  });

  it("rejects zero and negative quantities", () => {
    // # Arrange
    const zero = "0";
    const negative = "-1";

    // # Act
    const zeroResult = parseScaledQuantityInput(zero, 1);
    const negativeResult = parseScaledQuantityInput(negative, 1);

    // # Assert
    expect(zeroResult).toBeNull();
    expect(negativeResult).toBeNull();
  });

  it("rejects non-numeric input", () => {
    // # Arrange
    const input = "abc";

    // # Act
    const result = parseScaledQuantityInput(input, 1_000);

    // # Assert
    expect(result).toBeNull();
  });
});

it("accepts the backend maximum scaled quantity", () => {
  // # Arrange
  const input = "1000000000";

  // # Act
  const result = parseScaledQuantityInput(input, 1);

  // # Assert
  expect(result).toBe(1_000_000_000);
});

it("rejects a scaled quantity above the backend maximum", () => {
  // # Arrange
  const input = "1000000001";

  // # Act
  const result = parseScaledQuantityInput(input, 1);

  // # Assert
  expect(result).toBeNull();
});
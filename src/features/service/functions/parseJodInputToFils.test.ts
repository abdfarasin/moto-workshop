import { describe, expect, it } from "vitest";

import { parseJodInputToFils } from "./parseJodInputToFils";

describe("parseJodInputToFils", () => {
  it("parses whole JOD values", () => {
    // # Arrange
    const input = "4";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBe(4_000);
  });

  it("parses three-decimal JOD values", () => {
    // # Arrange
    const input = "4.500";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBe(4_500);
  });

  it("pads shorter fractional values to fils", () => {
    // # Arrange
    const input = "4.5";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBe(4_500);
  });

  it("allows zero because the backend allows a free charged part", () => {
    // # Arrange
    const input = "0";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBe(0);
  });

  it("rejects more than three decimal places", () => {
    // # Arrange
    const input = "4.5001";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBeNull();
  });

  it("rejects negative values", () => {
    // # Arrange
    const input = "-1";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBeNull();
  });

  it("rejects non-numeric input", () => {
    // # Arrange
    const input = "abc";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBeNull();
  });

  it("rejects prices above the backend maximum", () => {
    // # Arrange
    const input = "1000000.001";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBeNull();
  });

  it("accepts the exact backend maximum", () => {
    // # Arrange
    const input = "1000000.000";

    // # Act
    const result = parseJodInputToFils(input);

    // # Assert
    expect(result).toBe(1_000_000_000);
  });
});
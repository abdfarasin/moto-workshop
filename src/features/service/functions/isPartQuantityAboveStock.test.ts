import { describe, expect, it } from "vitest";

import { isPartQuantityAboveStock } from "./isPartQuantityAboveStock";

describe("isPartQuantityAboveStock", () => {
  it("returns false when requested quantity is within available stock", () => {
    // # Arrange
    const requestedQuantity = 4;
    const currentQuantity = 10;

    // # Act
    const result = isPartQuantityAboveStock(
      requestedQuantity,
      currentQuantity,
    );

    // # Assert
    expect(result).toBe(false);
  });

  it("returns true when requested quantity exceeds available stock", () => {
    // # Arrange
    const requestedQuantity = 12;
    const currentQuantity = 10;

    // # Act
    const result = isPartQuantityAboveStock(
      requestedQuantity,
      currentQuantity,
    );

    // # Assert
    expect(result).toBe(true);
  });

  it("returns true when stock is already negative", () => {
    // # Arrange
    const requestedQuantity = 1;
    const currentQuantity = -2;

    // # Act
    const result = isPartQuantityAboveStock(
      requestedQuantity,
      currentQuantity,
    );

    // # Assert
    expect(result).toBe(true);
  });
});
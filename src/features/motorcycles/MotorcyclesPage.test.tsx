// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { searchMotorcycleDirectory } from "./api/motorcycleDirectoryApi";
import { MotorcyclesPage } from "./MotorcyclesPage";

vi.mock("./api/motorcycleDirectoryApi", () => ({ searchMotorcycleDirectory: vi.fn() }));
const searchMock = vi.mocked(searchMotorcycleDirectory);

describe("MotorcyclesPage", () => {
  beforeEach(() => searchMock.mockReset());
  afterEach(() => cleanup());

  it("loads a bounded directory, delegates search to SQLite, and opens a real ID", async () => {
    // Arrange
    const user = userEvent.setup();
    const onSelect = vi.fn();
    searchMock.mockResolvedValue([{ id: 11, makeName: "Honda", model: "CB150R", ownerCustomerId: 7, ownerName: "Ahmad Ali", ownerPhone: "+962791234567", plateNumber: "29-12345", vin: null, chassisNumber: null, year: 2022, colorName: "Black", latestServiceVisitAt: 2000, activeServiceVisitId: 31, activeServiceVisitStatus: "OPEN" }]);
    render(<MotorcyclesPage onSelectMotorcycle={onSelect} />);
    expect(await screen.findByText("Honda CB150R")).toBeTruthy();
    expect(searchMock).toHaveBeenCalledWith({ query: "", limit: 50 });

    // Act
    await user.type(screen.getByLabelText("Search Motorcycles"), "29-12345");
    await user.click(screen.getByRole("button", { name: "Search" }));
    await user.click(screen.getByRole("button", { name: "Open Motorcycle 11" }));

    // Assert
    expect(searchMock).toHaveBeenLastCalledWith({ query: "29-12345", limit: 50 });
    expect(onSelect).toHaveBeenCalledWith(11);
  });

  it("shows loading, empty, and safe failure states", async () => {
    // Arrange
    let resolve!: (value: []) => void;
    searchMock.mockReturnValueOnce(new Promise((done) => { resolve = done; }));
    const { rerender } = render(<MotorcyclesPage onSelectMotorcycle={vi.fn()} />);

    // Act / Assert
    expect(screen.getByText("Loading Motorcycles...")).toBeTruthy();
    resolve([]);
    expect(await screen.findByText("No Motorcycles found")).toBeTruthy();
    searchMock.mockRejectedValueOnce(new Error("raw database detail"));
    rerender(<MotorcyclesPage key="failure" onSelectMotorcycle={vi.fn()} />);
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.queryByText("raw database detail")).toBeNull();
  });
});

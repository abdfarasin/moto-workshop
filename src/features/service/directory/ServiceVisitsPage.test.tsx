// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  listServiceVisits,
  loadServiceVisitWorkspace,
} from "../api/serviceVisitApi";
import type {
  ServiceVisitDirectoryEntry,
  ServiceVisitWorkspace,
} from "../api/serviceVisitApi";
import { ServiceVisitsPage } from "./ServiceVisitsPage";

vi.mock("../api/serviceVisitApi", async () => {
  const actual = await vi.importActual<typeof import("../api/serviceVisitApi")>(
    "../api/serviceVisitApi",
  );
  return {
    ...actual,
    listServiceVisits: vi.fn(),
    loadServiceVisitWorkspace: vi.fn(),
  };
});

const visits: ServiceVisitDirectoryEntry[] = [
  {
    id: 31,
    customerName: "Ahmad Ali",
    customerPhone: "+962791234567",
    motorcycleId: 11,
    makeName: "Honda",
    model: "CB150R",
    plateNumber: "29-12345",
    openedAt: 1_725_000_000_000,
    customerComplaint: "Oil leak",
    status: "OPEN",
    totalFils: 9_500,
  },
];

const workspace = {
  visit: { id: 31 },
  owner: { id: 7 },
} as ServiceVisitWorkspace;

const listServiceVisitsMock = vi.mocked(listServiceVisits);
const loadServiceVisitWorkspaceMock = vi.mocked(loadServiceVisitWorkspace);

describe("ServiceVisitsPage", () => {
  beforeEach(() => {
    listServiceVisitsMock.mockReset();
    loadServiceVisitWorkspaceMock.mockReset();
    listServiceVisitsMock.mockResolvedValue(visits);
    loadServiceVisitWorkspaceMock.mockResolvedValue(workspace);
  });

  afterEach(() => cleanup());

  it("loads the bounded active-work default and renders identifying persisted fields", async () => {
    // Arrange / Act
    render(<ServiceVisitsPage onOpenServiceVisit={vi.fn()} />);

    // Assert
    expect(await screen.findByText("Ahmad Ali")).toBeTruthy();
    expect(listServiceVisitsMock).toHaveBeenCalledWith({
      query: "",
      statusFilter: "ACTIVE",
      limit: 50,
    });
    expect(screen.getByText("+962791234567")).toBeTruthy();
    expect(screen.getByText("Honda CB150R")).toBeTruthy();
    expect(screen.getByText("29-12345")).toBeTruthy();
    expect(screen.getByText("Oil leak")).toBeTruthy();
    expect(screen.getByText("9.500 JD")).toBeTruthy();
  });

  it("sends status and submitted search filters back to SQLite", async () => {
    // Arrange
    const user = userEvent.setup();
    render(<ServiceVisitsPage onOpenServiceVisit={vi.fn()} />);
    await screen.findByText("Ahmad Ali");

    // Act
    await user.selectOptions(screen.getByLabelText("Status"), "CLOSED");
    await user.type(screen.getByLabelText("Search Service Visits"), "29-12345");
    await user.click(screen.getByRole("button", { name: "Search" }));

    // Assert
    await waitFor(() => expect(listServiceVisitsMock).toHaveBeenLastCalledWith({
      query: "29-12345",
      statusFilter: "CLOSED",
      limit: 50,
    }));
  });

  it("loads the clicked visit workspace by real ID and opens it", async () => {
    // Arrange
    const user = userEvent.setup();
    const onOpenServiceVisit = vi.fn();
    render(<ServiceVisitsPage onOpenServiceVisit={onOpenServiceVisit} />);
    await screen.findByText("Ahmad Ali");

    // Act
    await user.click(screen.getByRole("button", { name: "Open Service Visit 31" }));

    // Assert
    expect(loadServiceVisitWorkspaceMock).toHaveBeenCalledWith(31);
    await waitFor(() => expect(onOpenServiceVisit).toHaveBeenCalledWith(workspace));
  });

  it("shows safe list and workspace-loading failures", async () => {
    // Arrange
    listServiceVisitsMock.mockRejectedValueOnce(new Error("sqlite detail"));
    const user = userEvent.setup();
    const { rerender } = render(<ServiceVisitsPage onOpenServiceVisit={vi.fn()} />);
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.queryByText("sqlite detail")).toBeNull();

    listServiceVisitsMock.mockResolvedValueOnce(visits);
    rerender(<ServiceVisitsPage key="retry" onOpenServiceVisit={vi.fn()} />);
    await screen.findByText("Ahmad Ali");
    loadServiceVisitWorkspaceMock.mockRejectedValueOnce(new Error("raw database detail"));

    // Act
    await user.click(screen.getByRole("button", { name: "Open Service Visit 31" }));

    // Assert
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.queryByText("raw database detail")).toBeNull();
  });
});

// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { loadServiceVisitWorkspace } from "../service/api/serviceVisitApi";
import type {
  CustomerMotorcycleLookup,
  CustomerSummary,
  ServiceVisitWorkspace,
} from "../service/api/serviceVisitApi";
import { loadCustomerDetails } from "./api/customerDirectoryApi";
import { CustomerDetailsPage } from "./CustomerDetailsPage";

vi.mock("./api/customerDirectoryApi", async () => {
  const actual = await vi.importActual<typeof import("./api/customerDirectoryApi")>(
    "./api/customerDirectoryApi",
  );
  return { ...actual, loadCustomerDetails: vi.fn() };
});

vi.mock("../service/api/serviceVisitApi", async () => {
  const actual = await vi.importActual<typeof import("../service/api/serviceVisitApi")>(
    "../service/api/serviceVisitApi",
  );
  return { ...actual, loadServiceVisitWorkspace: vi.fn() };
});

vi.mock("../motorcycles/new-motorcycle/NewMotorcycleDialog", () => ({
  NewMotorcycleDialog: ({
    open,
    customer,
    onCreated,
  }: {
    open: boolean;
    customer: CustomerSummary;
    onCreated: (motorcycle: CustomerMotorcycleLookup) => void;
  }) => open ? (
    <div role="dialog" aria-label="Motorcycle registration">
      <span>Register for {customer.name}</span>
      <button
        type="button"
        onClick={() => onCreated(createdMotorcycle)}
      >
        Complete motorcycle
      </button>
    </div>
  ) : null,
}));

vi.mock("../service/new-visit/NewServiceVisitDialog", () => ({
  NewServiceVisitDialog: ({
    open,
    initialCustomer,
    onCreated,
  }: {
    open: boolean;
    initialCustomer?: CustomerSummary;
    onCreated: (workspace: ServiceVisitWorkspace) => void;
  }) => open ? (
    <div role="dialog" aria-label="Service Visit creation">
      <span>Visit for {initialCustomer?.name}</span>
      <button type="button" onClick={() => onCreated(workspace)}>
        Complete Service Visit
      </button>
    </div>
  ) : null,
}));

const details = {
  id: 7,
  name: "Ahmad Ali",
  phone: "+962791234567",
  motorcycles: [
    {
      id: 11,
      makeName: "Honda",
      model: "CB150R",
      year: 2022,
      plateNumber: "29-12345",
      vin: null,
      chassisNumber: null,
      colorName: "Black",
    },
  ],
  serviceHistory: [
    {
      id: 31,
      motorcycleId: 11,
      openedAt: 1_725_000_000_000,
      odometerKm: 15_200,
      customerComplaint: "Oil leak",
      status: "OPEN",
      totalFils: 5_000,
    },
  ],
};

const createdMotorcycle: CustomerMotorcycleLookup = {
  id: 12,
  makeName: "Yamaha",
  model: "YBR125",
  year: 2020,
  colorName: "Red",
  plateNumber: "30-99",
  vin: null,
  chassisNumber: null,
  activeServiceVisitId: null,
  activeServiceVisitStatus: null,
};

const workspace: ServiceVisitWorkspace = {
  visit: {
    id: 31,
    motorcycleId: 11,
    ownerCustomerId: 7,
    status: "OPEN",
    openedAt: 1_725_000_000_000,
    completedAt: null,
    closedAt: null,
    cancelledAt: null,
    odometerKm: 15_200,
    customerComplaint: "Oil leak",
    diagnosis: null,
    workPerformed: null,
    laborChargeFils: 5_000,
    cancellationReason: null,
    notes: null,
    createdAt: 1_725_000_000_000,
    updatedAt: 1_725_000_000_000,
  },
  owner: { id: 7, name: "Ahmad Ali", phone: "+962791234567" },
  motorcycle: {
    id: 11,
    makeName: "Honda",
    model: "CB150R",
    year: 2022,
    plateNumber: "29-12345",
    vin: null,
    chassisNumber: null,
    colorName: "Black",
  },
  parts: [],
};

const loadCustomerDetailsMock = vi.mocked(loadCustomerDetails);
const loadServiceVisitWorkspaceMock = vi.mocked(loadServiceVisitWorkspace);

describe("CustomerDetailsPage actions", () => {
  beforeEach(() => {
    loadCustomerDetailsMock.mockReset();
    loadServiceVisitWorkspaceMock.mockReset();
    loadCustomerDetailsMock.mockResolvedValue(details);
    loadServiceVisitWorkspaceMock.mockResolvedValue(workspace);
  });

  afterEach(() => cleanup());

  it("opens motorcycle registration for the current customer and reloads after success", async () => {
    // Arrange
    const user = userEvent.setup();
    render(
      <CustomerDetailsPage customerId={7} onBack={vi.fn()} onOpenServiceVisit={vi.fn()} />,
    );
    await screen.findByRole("heading", { name: "Ahmad Ali" });

    // Act
    await user.click(screen.getByRole("button", { name: "Add Motorcycle" }));
    expect(screen.getByText("Register for Ahmad Ali")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Complete motorcycle" }));

    // Assert
    await waitFor(() => expect(loadCustomerDetailsMock).toHaveBeenCalledTimes(2));
  });

  it("opens Service Visit creation with the current customer and forwards the created workspace", async () => {
    // Arrange
    const user = userEvent.setup();
    const onOpenServiceVisit = vi.fn();
    render(
      <CustomerDetailsPage
        customerId={7}
        onBack={vi.fn()}
        onOpenServiceVisit={onOpenServiceVisit}
      />,
    );
    await screen.findByRole("heading", { name: "Ahmad Ali" });

    // Act
    await user.click(screen.getByRole("button", { name: "New Service Visit" }));
    expect(screen.getByText("Visit for Ahmad Ali")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Complete Service Visit" }));

    // Assert
    expect(onOpenServiceVisit).toHaveBeenCalledWith(workspace);
    await waitFor(() => expect(loadCustomerDetailsMock).toHaveBeenCalledTimes(2));
  });

  it("loads a real workspace by the clicked service-history visit ID", async () => {
    // Arrange
    const user = userEvent.setup();
    const onOpenServiceVisit = vi.fn();
    render(
      <CustomerDetailsPage
        customerId={7}
        onBack={vi.fn()}
        onOpenServiceVisit={onOpenServiceVisit}
      />,
    );
    await screen.findByText("Oil leak");

    // Act
    await user.click(screen.getByText("Oil leak"));

    // Assert
    await waitFor(() => expect(loadServiceVisitWorkspaceMock).toHaveBeenCalledWith(31));
    expect(onOpenServiceVisit).toHaveBeenCalledWith(workspace);
  });

  it("keeps the existing safe error state when customer loading fails", async () => {
    // Arrange
    loadCustomerDetailsMock.mockRejectedValueOnce(new Error("sqlite internals"));

    // Act
    render(
      <CustomerDetailsPage customerId={7} onBack={vi.fn()} onOpenServiceVisit={vi.fn()} />,
    );

    // Assert
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Customer could not be loaded",
    );
    expect(screen.queryByText("sqlite internals")).toBeNull();
  });
});

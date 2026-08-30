// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  createServiceVisit,
  listCustomerMotorcycles,
  searchCustomers,
  ServiceVisitCommandError,
} from "../api/serviceVisitApi";
import type {
  CustomerMotorcycleLookup,
  CustomerSummary,
  ServiceVisitWorkspace,
} from "../api/serviceVisitApi";
import { NewServiceVisitDialog } from "./NewServiceVisitDialog";

vi.mock("../api/serviceVisitApi", async () => {
  const actual = await vi.importActual<typeof import("../api/serviceVisitApi")>(
    "../api/serviceVisitApi",
  );

  return {
    ...actual,
    searchCustomers: vi.fn(),
    listCustomerMotorcycles: vi.fn(),
    createServiceVisit: vi.fn(),
  };
});

const searchCustomersMock = vi.mocked(searchCustomers);
const listCustomerMotorcyclesMock = vi.mocked(listCustomerMotorcycles);
const createServiceVisitMock = vi.mocked(createServiceVisit);

const customers: CustomerSummary[] = [
  { id: 7, name: "Ahmad Ali", phone: "+962791234567" },
  { id: 8, name: "Lina Saleh", phone: "+962799999999" },
];

const motorcycles: CustomerMotorcycleLookup[] = [
  {
    id: 11,
    makeName: "Honda",
    model: "CB150R",
    year: 2022,
    colorName: "Black",
    plateCode: "29",
    plateNumber: 12345,
    vin: "JH2RC4468MK123456",
    chassisNumber: null,
    activeServiceVisitId: null,
    activeServiceVisitStatus: null,
  },
  {
    id: 12,
    makeName: "Yamaha",
    model: "YBR125",
    year: null,
    colorName: "Red",
    plateCode: null,
    plateNumber: null,
    vin: null,
    chassisNumber: "FRAME-12",
    activeServiceVisitId: 91,
    activeServiceVisitStatus: "OPEN",
  },
  {
    id: 13,
    makeName: "Suzuki",
    model: "GSX",
    year: 2020,
    colorName: "Blue",
    plateCode: "30",
    plateNumber: 88,
    vin: null,
    chassisNumber: null,
    activeServiceVisitId: 92,
    activeServiceVisitStatus: "READY_FOR_PICKUP",
  },
];

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
    odometerKm: null,
    customerComplaint: "Oil leak",
    diagnosis: null,
    workPerformed: null,
    laborChargeFils: 0,
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
    plateCode: "29",
    plateNumber: 12345,
    vin: "JH2RC4468MK123456",
    chassisNumber: null,
    colorName: "Black",
  },
  parts: [],
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve;
    reject = promiseReject;
  });
  return { promise, resolve, reject };
}

function renderDialog(overrides?: {
  onClose?: () => void;
  onCreated?: (created: ServiceVisitWorkspace) => void;
}) {
  const onClose = overrides?.onClose ?? vi.fn();
  const onCreated = overrides?.onCreated ?? vi.fn();
  const result = render(
    <NewServiceVisitDialog open onClose={onClose} onCreated={onCreated} />,
  );
  return { ...result, onClose, onCreated };
}

async function selectUsableMotorcycle(user: ReturnType<typeof userEvent.setup>) {
  await screen.findByText("Ahmad Ali");
  await user.click(screen.getByRole("button", { name: /Ahmad Ali/i }));
  await screen.findByText("Honda CB150R");
  await user.click(screen.getByRole("button", { name: /Honda CB150R/i }));
}

describe("NewServiceVisitDialog", () => {
  beforeEach(() => {
    searchCustomersMock.mockReset();
    listCustomerMotorcyclesMock.mockReset();
    createServiceVisitMock.mockReset();
    searchCustomersMock.mockResolvedValue(customers);
    listCustomerMotorcyclesMock.mockResolvedValue(motorcycles);
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it("renders nothing while closed", () => {
    // Arrange / Act
    const { container } = render(
      <NewServiceVisitDialog open={false} onClose={vi.fn()} onCreated={vi.fn()} />,
    );

    // Assert
    expect(container.firstChild).toBeNull();
    expect(searchCustomersMock).not.toHaveBeenCalled();
  });

  it("loads recent Customers on open and displays loading, name, phone, and empty states", async () => {
    // Arrange
    const recent = deferred<CustomerSummary[]>();
    searchCustomersMock.mockReturnValueOnce(recent.promise);

    // Act
    const { rerender } = renderDialog();

    // Assert
    expect(screen.getByText("Loading customers…")).toBeTruthy();
    expect(searchCustomersMock).toHaveBeenCalledWith({ query: "", limit: 25 });
    recent.resolve(customers);
    expect(await screen.findByText("Ahmad Ali")).toBeTruthy();
    expect(screen.getByText("+962791234567")).toBeTruthy();

    searchCustomersMock.mockResolvedValueOnce([]);
    rerender(<NewServiceVisitDialog open={false} onClose={vi.fn()} onCreated={vi.fn()} />);
    rerender(<NewServiceVisitDialog open onClose={vi.fn()} onCreated={vi.fn()} />);
    expect(await screen.findByText("No customers found.")).toBeTruthy();
  });

  it("searches by the entered name or phone and shows a safe failed-search state", async () => {
    // Arrange
    const user = userEvent.setup();
    renderDialog();
    await screen.findByText("Ahmad Ali");
    searchCustomersMock.mockResolvedValueOnce([customers[1]]);

    // Act
    await user.type(screen.getByLabelText("Search customers"), "  Lina  ");
    await user.click(screen.getByRole("button", { name: "Search" }));

    // Assert
    expect(searchCustomersMock).toHaveBeenLastCalledWith({
      query: "Lina",
      limit: 25,
    });
    expect(await screen.findByText("Lina Saleh")).toBeTruthy();

    searchCustomersMock.mockRejectedValueOnce(new Error("secret transport"));
    await user.click(screen.getByRole("button", { name: "Search" }));
    expect(await screen.findByText("Could not load customers. Please try again.")).toBeTruthy();
    expect(screen.queryByText("secret transport")).toBeNull();
  });

  it("loads the exact Customer's Motorcycles and shows a clear zero-Motorcycle state", async () => {
    // Arrange
    const user = userEvent.setup();
    listCustomerMotorcyclesMock.mockResolvedValueOnce([]);
    renderDialog();

    // Act
    await user.click(await screen.findByRole("button", { name: /Ahmad Ali/i }));

    // Assert
    expect(listCustomerMotorcyclesMock).toHaveBeenCalledWith(7);
    expect(await screen.findByText("This customer has no motorcycles.")).toBeTruthy();
  });

  it("shows real Motorcycle fields and blocks OPEN and READY_FOR_PICKUP visits with text", async () => {
    // Arrange
    const user = userEvent.setup();
    renderDialog();

    // Act
    await user.click(await screen.findByRole("button", { name: /Ahmad Ali/i }));

    // Assert
    expect(await screen.findByText("Honda CB150R")).toBeTruthy();
    expect(screen.getByText(/2022.*Black/)).toBeTruthy();
    expect(screen.getByText(/Plate 29-12345/)).toBeTruthy();
    expect(screen.getByText(/VIN JH2RC4468MK123456/)).toBeTruthy();
    expect(screen.getByText(/Chassis FRAME-12/)).toBeTruthy();
    expect(screen.getByText("Already has an active Visit: OPEN")).toBeTruthy();
    expect(screen.getByText("Already has an active Visit: READY_FOR_PICKUP")).toBeTruthy();
    expect(
      (screen.getByRole("button", { name: /Yamaha YBR125/i }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(
      (screen.getByRole("button", { name: /Suzuki GSX/i }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  });

  it("allows a usable Motorcycle and clears it when the Customer changes", async () => {
    // Arrange
    const user = userEvent.setup();
    renderDialog();
    await selectUsableMotorcycle(user);
    expect(screen.getByLabelText("Customer complaint")).toBeTruthy();
    listCustomerMotorcyclesMock.mockResolvedValueOnce([]);

    // Act
    await user.click(screen.getByRole("button", { name: /Lina Saleh/i }));

    // Assert
    expect(listCustomerMotorcyclesMock).toHaveBeenLastCalledWith(8);
    await waitFor(() => {
      expect(screen.queryByLabelText("Customer complaint")).toBeNull();
    });
  });

  it("requires a complaint and rejects invalid odometer shapes without submitting", async () => {
    // Arrange
    const user = userEvent.setup();
    renderDialog();
    await selectUsableMotorcycle(user);

    // Act
    await user.click(screen.getByRole("button", { name: "Create Service Visit" }));

    // Assert
    expect(await screen.findByText("Customer Complaint is required.")).toBeTruthy();
    expect(createServiceVisitMock).not.toHaveBeenCalled();

    await user.type(screen.getByLabelText("Customer complaint"), "Oil leak");
    await user.type(screen.getByLabelText("Odometer (km)"), "1.5");
    await user.click(screen.getByRole("button", { name: "Create Service Visit" }));
    expect(screen.getByText("Odometer must be a nonnegative whole number.")).toBeTruthy();
    expect(createServiceVisitMock).not.toHaveBeenCalled();
  });

  it("sends one exact safe payload with trimmed text and null blank optional fields", async () => {
    // Arrange
    const user = userEvent.setup();
    const now = 1_725_000_000_000;
    vi.spyOn(Date, "now").mockReturnValue(now);
    createServiceVisitMock.mockResolvedValue(workspace);
    renderDialog();
    await selectUsableMotorcycle(user);

    // Act
    await user.type(screen.getByLabelText("Customer complaint"), "  Oil leak  ");
    await user.type(screen.getByLabelText("Notes"), "   ");
    await user.click(screen.getByRole("button", { name: "Create Service Visit" }));

    // Assert
    await waitFor(() => expect(createServiceVisitMock).toHaveBeenCalledTimes(1));
    expect(createServiceVisitMock).toHaveBeenCalledWith({
      motorcycleId: 11,
      openedAt: now,
      odometerKm: null,
      customerComplaint: "Oil leak",
      notes: null,
      createdAt: now,
    });
    expect(Object.keys(createServiceVisitMock.mock.calls[0][0]).sort()).toEqual(
      [
        "motorcycleId",
        "openedAt",
        "odometerKm",
        "customerComplaint",
        "notes",
        "createdAt",
      ].sort(),
    );
  });

  it("sends integer odometer and trimmed nonblank notes", async () => {
    // Arrange
    const user = userEvent.setup();
    createServiceVisitMock.mockResolvedValue(workspace);
    renderDialog();
    await selectUsableMotorcycle(user);

    // Act
    await user.type(screen.getByLabelText("Customer complaint"), "Brake noise");
    await user.type(screen.getByLabelText("Odometer (km)"), "48231");
    await user.type(screen.getByLabelText("Notes"), "  Customer waiting  ");
    await user.click(screen.getByRole("button", { name: "Create Service Visit" }));

    // Assert
    await waitFor(() => expect(createServiceVisitMock).toHaveBeenCalledTimes(1));
    expect(createServiceVisitMock.mock.calls[0][0]).toMatchObject({
      odometerKm: 48_231,
      notes: "Customer waiting",
    });
    expect(Number.isNaN(createServiceVisitMock.mock.calls[0][0].odometerKm)).toBe(false);
  });

  it("disables Create while pending and ignores duplicate submission", async () => {
    // Arrange
    const user = userEvent.setup();
    const creation = deferred<ServiceVisitWorkspace>();
    createServiceVisitMock.mockReturnValue(creation.promise);
    renderDialog();
    await selectUsableMotorcycle(user);
    await user.type(screen.getByLabelText("Customer complaint"), "Oil leak");
    const createButton = screen.getByRole("button", { name: "Create Service Visit" });

    // Act
    await user.click(createButton);
    fireEvent.click(createButton);

    // Assert
    expect((createButton as HTMLButtonElement).disabled).toBe(true);
    expect(createServiceVisitMock).toHaveBeenCalledTimes(1);
    creation.resolve(workspace);
  });

  it("returns the workspace, resets transient state, and closes after success", async () => {
    // Arrange
    const user = userEvent.setup();
    const onCreated = vi.fn();
    const onClose = vi.fn();
    createServiceVisitMock.mockResolvedValue(workspace);
    const { rerender } = renderDialog({ onCreated, onClose });
    await selectUsableMotorcycle(user);
    await user.type(screen.getByLabelText("Customer complaint"), "Oil leak");

    // Act
    await user.click(screen.getByRole("button", { name: "Create Service Visit" }));

    // Assert
    await waitFor(() => expect(onCreated).toHaveBeenCalledWith(workspace));
    expect(onClose).toHaveBeenCalledTimes(1);
    rerender(<NewServiceVisitDialog open={false} onClose={onClose} onCreated={onCreated} />);
    rerender(<NewServiceVisitDialog open onClose={onClose} onCreated={onCreated} />);
    await screen.findByLabelText("Search customers");
    expect((screen.getByLabelText("Search customers") as HTMLInputElement).value).toBe("");
    expect(screen.queryByLabelText("Customer complaint")).toBeNull();
  });

  it.each([
    ["customerNotFound", "The selected customer is no longer available."],
    ["motorcycleNotFound", "The selected motorcycle is no longer available."],
    ["activeServiceVisitExists", "This motorcycle now has an active Service Visit."],
    ["validationError", "Please review the Visit details and try again."],
    ["databaseError", "The Service Visit could not be saved. Please try again."],
  ] as const)("shows safe %s creation feedback", async (category, expectedMessage) => {
    // Arrange
    const user = userEvent.setup();
    createServiceVisitMock.mockRejectedValue(
      new ServiceVisitCommandError({ category, message: "backend detail" }),
    );
    renderDialog();
    await selectUsableMotorcycle(user);
    await user.type(screen.getByLabelText("Customer complaint"), "Oil leak");

    // Act
    await user.click(screen.getByRole("button", { name: "Create Service Visit" }));

    // Assert
    expect(await screen.findByText(expectedMessage)).toBeTruthy();
    expect(screen.queryByText("backend detail")).toBeNull();
  });

  it("shows a generic message for an unexpected API failure", async () => {
    // Arrange
    const user = userEvent.setup();
    createServiceVisitMock.mockRejectedValue(new Error("stack and secret"));
    renderDialog();
    await selectUsableMotorcycle(user);
    await user.type(screen.getByLabelText("Customer complaint"), "Oil leak");

    // Act
    await user.click(screen.getByRole("button", { name: "Create Service Visit" }));

    // Assert
    expect(await screen.findByText("Something went wrong. Please try again.")).toBeTruthy();
    expect(screen.queryByText("stack and secret")).toBeNull();
  });

  it("ignores stale search and Motorcycle responses", async () => {
    // Arrange
    const user = userEvent.setup();
    const oldSearch = deferred<CustomerSummary[]>();
    searchCustomersMock.mockReturnValueOnce(oldSearch.promise);
    renderDialog();
    searchCustomersMock.mockResolvedValueOnce([customers[1]]);

    // Act
    await user.type(screen.getByLabelText("Search customers"), "Lina");
    await user.click(screen.getByRole("button", { name: "Search" }));
    expect(await screen.findByText("Lina Saleh")).toBeTruthy();
    oldSearch.resolve([customers[0]]);

    // Assert
    await waitFor(() => expect(screen.queryByText("Ahmad Ali")).toBeNull());

    searchCustomersMock.mockResolvedValueOnce(customers);
    await user.clear(screen.getByLabelText("Search customers"));
    await user.click(screen.getByRole("button", { name: "Search" }));
    await screen.findByText("Ahmad Ali");
    const oldMotorcycles = deferred<CustomerMotorcycleLookup[]>();
    listCustomerMotorcyclesMock.mockReturnValueOnce(oldMotorcycles.promise);
    listCustomerMotorcyclesMock.mockResolvedValueOnce([]);
    await user.click(screen.getByRole("button", { name: /Ahmad Ali/i }));
    await user.click(screen.getByRole("button", { name: /Lina Saleh/i }));
    expect(await screen.findByText("This customer has no motorcycles.")).toBeTruthy();
    oldMotorcycles.resolve(motorcycles);
    await waitFor(() => expect(screen.queryByText("Honda CB150R")).toBeNull());
  });

  it("only closes from the actual backdrop and resets state when closed", async () => {
    // Arrange
    const user = userEvent.setup();
    const onClose = vi.fn();
    const { rerender } = renderDialog({ onClose });
    await user.type(screen.getByLabelText("Search customers"), "Ahmad");
    const dialog = screen.getByRole("dialog", { name: "New Service Visit" });
    const backdrop = dialog.parentElement as HTMLElement;

    // Act / Assert
    fireEvent.click(dialog);
    expect(onClose).not.toHaveBeenCalled();
    fireEvent.click(backdrop);
    expect(onClose).toHaveBeenCalledTimes(1);

    rerender(<NewServiceVisitDialog open={false} onClose={onClose} onCreated={vi.fn()} />);
    rerender(<NewServiceVisitDialog open onClose={onClose} onCreated={vi.fn()} />);
    expect((await screen.findByLabelText("Search customers") as HTMLInputElement).value).toBe("");
  });

  it("closes on Escape and resets transient state", async () => {
    // Arrange
    const user = userEvent.setup();
    const onClose = vi.fn();
    const { rerender } = renderDialog({ onClose });
    await user.type(screen.getByLabelText("Search customers"), "Ahmad");

    // Act
    fireEvent.keyDown(document, { key: "Escape" });

    // Assert
    expect(onClose).toHaveBeenCalledTimes(1);
    rerender(<NewServiceVisitDialog open={false} onClose={onClose} onCreated={vi.fn()} />);
    rerender(<NewServiceVisitDialog open onClose={onClose} onCreated={vi.fn()} />);
    expect((await screen.findByLabelText("Search customers") as HTMLInputElement).value).toBe("");
  });
});

// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  createMotorcycle,
  loadMotorcycleRegistrationReferenceData,
  ServiceVisitCommandError,
} from "../../service/api/serviceVisitApi";
import type {
  CustomerMotorcycleLookup,
  CustomerSummary,
  MotorcycleRegistrationReferenceData,
} from "../../service/api/serviceVisitApi";
import { NewMotorcycleDialog } from "./NewMotorcycleDialog";

vi.mock("../../service/api/serviceVisitApi", async () => {
  const actual = await vi.importActual<
    typeof import("../../service/api/serviceVisitApi")
  >("../../service/api/serviceVisitApi");
  return {
    ...actual,
    loadMotorcycleRegistrationReferenceData: vi.fn(),
    createMotorcycle: vi.fn(),
  };
});

const loadReferencesMock = vi.mocked(loadMotorcycleRegistrationReferenceData);
const createMotorcycleMock = vi.mocked(createMotorcycle);

const customer: CustomerSummary = {
  id: 17,
  name: "Ahmad Ali",
  phone: "+962791234567",
};

const references: MotorcycleRegistrationReferenceData = {
  makes: [
    { id: 2, name: "Honda" },
    { id: 5, name: "Yamaha" },
  ],
  colors: [
    { id: 3, name: "Black" },
    { id: 8, name: "Red" },
  ],
  plateCodes: [
    { id: 4, code: "29" },
    { id: 9, code: "30" },
  ],
};

const createdMotorcycle: CustomerMotorcycleLookup = {
  id: 31,
  makeName: "Honda",
  model: "CB150R",
  year: 2022,
  colorName: "Black",
  plateCode: "29",
  plateNumber: 12345,
  vin: "1HGCM82633A004352",
  chassisNumber: null,
  activeServiceVisitId: null,
  activeServiceVisitStatus: null,
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
  onCreated?: (motorcycle: CustomerMotorcycleLookup) => void;
}) {
  const onClose = overrides?.onClose ?? vi.fn();
  const onCreated = overrides?.onCreated ?? vi.fn();
  const result = render(
    <NewMotorcycleDialog
      open
      customer={customer}
      onClose={onClose}
      onCreated={onCreated}
    />,
  );
  return { ...result, onClose, onCreated };
}

async function completeRequiredFields(user: ReturnType<typeof userEvent.setup>) {
  await screen.findByRole("option", { name: "Honda" });
  await user.selectOptions(screen.getByLabelText("Make"), "2");
  await user.type(screen.getByLabelText("Model"), "CB150R");
  await user.selectOptions(screen.getByLabelText("Color"), "3");
}

describe("NewMotorcycleDialog", () => {
  beforeEach(() => {
    loadReferencesMock.mockReset();
    createMotorcycleMock.mockReset();
    loadReferencesMock.mockResolvedValue(references);
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it("renders nothing while closed", () => {
    // Arrange / Act
    const { container } = render(
      <NewMotorcycleDialog
        open={false}
        customer={customer}
        onClose={vi.fn()}
        onCreated={vi.fn()}
      />,
    );

    // Assert
    expect(container.firstChild).toBeNull();
    expect(loadReferencesMock).not.toHaveBeenCalled();
  });

  it("shows authoritative Customer context and loads references exactly once on open", async () => {
    // Arrange / Act
    renderDialog();

    // Assert
    expect(screen.getByRole("dialog", { name: "New Motorcycle" })).toBeTruthy();
    expect(screen.getByText("Ahmad Ali")).toBeTruthy();
    expect(screen.getByText("+962791234567")).toBeTruthy();
    await screen.findByRole("option", { name: "Honda" });
    expect(loadReferencesMock).toHaveBeenCalledTimes(1);
  });

  it("shows loading and API-provided make, color, and plate-code options", async () => {
    // Arrange
    const loading = deferred<MotorcycleRegistrationReferenceData>();
    loadReferencesMock.mockReturnValueOnce(loading.promise);

    // Act
    renderDialog();

    // Assert
    expect(screen.getByText("Loading registration options…")).toBeTruthy();
    loading.resolve(references);
    expect(await screen.findByRole("option", { name: "Honda" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Yamaha" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Black" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Red" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "29" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "30" })).toBeTruthy();
  });

  it("shows a safe reference-data failure without raw details", async () => {
    // Arrange
    loadReferencesMock.mockRejectedValue(new Error("SQLite secret"));

    // Act
    renderDialog();

    // Assert
    expect(
      await screen.findByText("Could not load motorcycle registration options. Please try again."),
    ).toBeTruthy();
    expect(screen.queryByText("SQLite secret")).toBeNull();
  });

  it("shows useful empty states for missing expected catalogs", async () => {
    // Arrange
    loadReferencesMock.mockResolvedValue({ makes: [], colors: [], plateCodes: [] });

    // Act
    renderDialog();

    // Assert
    expect(await screen.findByText("No motorcycle makes are available.")).toBeTruthy();
    expect(screen.getByText("No motorcycle colors are available.")).toBeTruthy();
    expect(
      screen.getByText("No plate codes are available; use a VIN or chassis number."),
    ).toBeTruthy();
  });

  it("requires Make, Model, and Color without copying backend identity rules", async () => {
    // Arrange
    const user = userEvent.setup();
    renderDialog();
    await screen.findByRole("option", { name: "Honda" });

    // Act
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));

    // Assert
    expect(screen.getByText("Make is required.")).toBeTruthy();
    expect(screen.getByText("Model is required.")).toBeTruthy();
    expect(screen.getByText("Color is required.")).toBeTruthy();
    expect(createMotorcycleMock).not.toHaveBeenCalled();
  });

  it("rejects malformed Year but sends a valid whole Year as a number", async () => {
    // Arrange
    const user = userEvent.setup();
    createMotorcycleMock.mockResolvedValue(createdMotorcycle);
    renderDialog();
    await completeRequiredFields(user);

    // Act / Assert
    await user.type(screen.getByLabelText("Year"), "2022.5");
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));
    expect(screen.getByText("Year must be a whole number.")).toBeTruthy();
    expect(createMotorcycleMock).not.toHaveBeenCalled();

    await user.clear(screen.getByLabelText("Year"));
    await user.type(screen.getByLabelText("Year"), "2022");
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));
    await waitFor(() => expect(createMotorcycleMock).toHaveBeenCalledTimes(1));
    expect(createMotorcycleMock.mock.calls[0][0].year).toBe(2022);
  });

  it("sends blank optionals as null in the exact safe payload", async () => {
    // Arrange
    const user = userEvent.setup();
    const now = 1_725_000_000_000;
    createMotorcycleMock.mockResolvedValue(createdMotorcycle);
    renderDialog();
    await completeRequiredFields(user);
    await user.type(screen.getByLabelText("Model"), "   ");
    const createButton = screen.getByRole("button", { name: "Create Motorcycle" });
    vi.spyOn(Date, "now").mockReturnValue(now);

    // Act
    createButton.click();

    // Assert
    await waitFor(() => expect(createMotorcycleMock).toHaveBeenCalledTimes(1));
    expect(createMotorcycleMock).toHaveBeenCalledWith({
      customerId: 17,
      makeId: 2,
      model: "CB150R",
      year: null,
      plateCodeId: null,
      plateNumber: null,
      vin: null,
      chassisNumber: null,
      colorId: 3,
      notes: null,
      createdAt: now,
    });
    expect(Object.keys(createMotorcycleMock.mock.calls[0][0]).sort()).toEqual(
      [
        "customerId",
        "makeId",
        "model",
        "year",
        "plateCodeId",
        "plateNumber",
        "vin",
        "chassisNumber",
        "colorId",
        "notes",
        "createdAt",
      ].sort(),
    );
  });

  it("preserves plate, VIN, and chassis strings while trimming model and Notes", async () => {
    // Arrange
    const user = userEvent.setup();
    createMotorcycleMock.mockResolvedValue(createdMotorcycle);
    renderDialog();
    await completeRequiredFields(user);

    // Act
    await user.selectOptions(screen.getByLabelText("Plate Code"), "9");
    await user.type(screen.getByLabelText("Plate Number"), "  00042  ");
    await user.type(screen.getByLabelText("VIN"), "  vinLower123456789  ");
    await user.type(screen.getByLabelText("Chassis Number"), "  frame-Abc/1  ");
    await user.type(screen.getByLabelText("Notes"), "  Customer notes  ");
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));

    // Assert
    await waitFor(() => expect(createMotorcycleMock).toHaveBeenCalledTimes(1));
    expect(createMotorcycleMock.mock.calls[0][0]).toMatchObject({
      customerId: 17,
      makeId: 2,
      colorId: 3,
      plateCodeId: 9,
      plateNumber: "00042",
      vin: "vinLower123456789",
      chassisNumber: "frame-Abc/1",
      notes: "Customer notes",
    });
    expect(typeof createMotorcycleMock.mock.calls[0][0].plateNumber).toBe("string");
  });

  it("disables Create while pending and prevents rapid duplicate requests", async () => {
    // Arrange
    const user = userEvent.setup();
    const creation = deferred<CustomerMotorcycleLookup>();
    createMotorcycleMock.mockReturnValue(creation.promise);
    renderDialog();
    await completeRequiredFields(user);
    const createButton = screen.getByRole("button", { name: "Create Motorcycle" });

    // Act
    await user.click(createButton);
    fireEvent.click(createButton);

    // Assert
    expect((createButton as HTMLButtonElement).disabled).toBe(true);
    expect(createMotorcycleMock).toHaveBeenCalledTimes(1);
    creation.resolve(createdMotorcycle);
  });

  it("returns the Motorcycle, resets, and closes after success", async () => {
    // Arrange
    const user = userEvent.setup();
    const onCreated = vi.fn();
    const onClose = vi.fn();
    createMotorcycleMock.mockResolvedValue(createdMotorcycle);
    const { rerender } = renderDialog({ onCreated, onClose });
    await completeRequiredFields(user);

    // Act
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));

    // Assert
    await waitFor(() => expect(onCreated).toHaveBeenCalledWith(createdMotorcycle));
    expect(onClose).toHaveBeenCalledTimes(1);
    rerender(
      <NewMotorcycleDialog open={false} customer={customer} onClose={onClose} onCreated={onCreated} />,
    );
    rerender(
      <NewMotorcycleDialog open customer={customer} onClose={onClose} onCreated={onCreated} />,
    );
    await screen.findByRole("option", { name: "Honda" });
    expect((screen.getByLabelText("Make") as HTMLSelectElement).value).toBe("");
    expect((screen.getByLabelText("Model") as HTMLInputElement).value).toBe("");
  });

  it("ignores a stale creation result after the dialog closes", async () => {
    // Arrange
    const user = userEvent.setup();
    const creation = deferred<CustomerMotorcycleLookup>();
    const onCreated = vi.fn();
    const onClose = vi.fn();
    createMotorcycleMock.mockReturnValue(creation.promise);
    renderDialog({ onCreated, onClose });
    await completeRequiredFields(user);
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));

    // Act
    await user.click(screen.getByRole("button", { name: "Close dialog" }));
    creation.resolve(createdMotorcycle);

    // Assert
    await Promise.resolve();
    expect(onCreated).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it.each([
    ["customerNotFound", "The selected customer is no longer available."],
    [
      "motorcycleIdentityAlreadyExists",
      "A motorcycle with this plate, VIN, or chassis number already exists.",
    ],
    ["validationError", "Please review the motorcycle details and try again."],
    ["databaseError", "The motorcycle could not be saved. Please try again."],
  ] as const)("shows safe %s feedback", async (category, expectedMessage) => {
    // Arrange
    const user = userEvent.setup();
    createMotorcycleMock.mockRejectedValue(
      new ServiceVisitCommandError({ category, message: "raw SQLite detail" }),
    );
    renderDialog();
    await completeRequiredFields(user);

    // Act
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));

    // Assert
    expect(await screen.findByText(expectedMessage)).toBeTruthy();
    expect(screen.queryByText("raw SQLite detail")).toBeNull();
  });

  it("shows a generic safe message for unexpected rejection", async () => {
    // Arrange
    const user = userEvent.setup();
    createMotorcycleMock.mockRejectedValue(new Error("stack and SQL secret"));
    renderDialog();
    await completeRequiredFields(user);

    // Act
    await user.click(screen.getByRole("button", { name: "Create Motorcycle" }));

    // Assert
    expect(await screen.findByText("Something went wrong. Please try again.")).toBeTruthy();
    expect(screen.queryByText("stack and SQL secret")).toBeNull();
  });

  it("only closes from the actual backdrop and resets state", async () => {
    // Arrange
    const user = userEvent.setup();
    const onClose = vi.fn();
    const { rerender } = renderDialog({ onClose });
    await screen.findByRole("option", { name: "Honda" });
    await user.type(screen.getByLabelText("Model"), "Unsaved Model");
    const dialog = screen.getByRole("dialog", { name: "New Motorcycle" });
    const backdrop = dialog.parentElement as HTMLElement;

    // Act / Assert
    fireEvent.click(dialog);
    expect(onClose).not.toHaveBeenCalled();
    fireEvent.click(backdrop);
    expect(onClose).toHaveBeenCalledTimes(1);
    rerender(
      <NewMotorcycleDialog open={false} customer={customer} onClose={onClose} onCreated={vi.fn()} />,
    );
    rerender(
      <NewMotorcycleDialog open customer={customer} onClose={onClose} onCreated={vi.fn()} />,
    );
    expect((await screen.findByLabelText("Model") as HTMLInputElement).value).toBe("");
  });

  it("closes on Escape and resets state", async () => {
    // Arrange
    const user = userEvent.setup();
    const onClose = vi.fn();
    const { rerender } = renderDialog({ onClose });
    await screen.findByRole("option", { name: "Honda" });
    await user.type(screen.getByLabelText("VIN"), "unsaved-vin");

    // Act
    fireEvent.keyDown(document, { key: "Escape" });

    // Assert
    expect(onClose).toHaveBeenCalledTimes(1);
    rerender(
      <NewMotorcycleDialog open={false} customer={customer} onClose={onClose} onCreated={vi.fn()} />,
    );
    rerender(
      <NewMotorcycleDialog open customer={customer} onClose={onClose} onCreated={vi.fn()} />,
    );
    expect((await screen.findByLabelText("VIN") as HTMLInputElement).value).toBe("");
  });

  it("does not let an older reference response update a reopened dialog", async () => {
    // Arrange
    const oldLoad = deferred<MotorcycleRegistrationReferenceData>();
    loadReferencesMock.mockReturnValueOnce(oldLoad.promise);
    const { rerender } = renderDialog();
    rerender(
      <NewMotorcycleDialog open={false} customer={customer} onClose={vi.fn()} onCreated={vi.fn()} />,
    );
    loadReferencesMock.mockResolvedValueOnce(references);

    // Act
    rerender(
      <NewMotorcycleDialog open customer={customer} onClose={vi.fn()} onCreated={vi.fn()} />,
    );
    expect(await screen.findByRole("option", { name: "Honda" })).toBeTruthy();
    oldLoad.resolve({
      makes: [{ id: 99, name: "Stale Make" }],
      colors: [{ id: 99, name: "Stale Color" }],
      plateCodes: [],
    });

    // Assert
    await waitFor(() => expect(screen.queryByRole("option", { name: "Stale Make" })).toBeNull());
  });
});

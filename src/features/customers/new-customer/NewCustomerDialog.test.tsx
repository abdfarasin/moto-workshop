// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  createCustomer,
  ServiceVisitCommandError,
} from "../../service/api/serviceVisitApi";
import type { CustomerSummary } from "../../service/api/serviceVisitApi";
import { NewCustomerDialog } from "./NewCustomerDialog";

vi.mock("../../service/api/serviceVisitApi", async () => {
  const actual = await vi.importActual<
    typeof import("../../service/api/serviceVisitApi")
  >("../../service/api/serviceVisitApi");

  return {
    ...actual,
    createCustomer: vi.fn(),
  };
});

const createCustomerMock = vi.mocked(createCustomer);
const createdCustomer: CustomerSummary = {
  id: 41,
  name: "Ahmad Ali",
  phone: "+962791234567",
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((promiseResolve) => {
    resolve = promiseResolve;
  });
  return { promise, resolve };
}

function renderDialog(overrides?: {
  onClose?: () => void;
  onCreated?: (customer: CustomerSummary) => void;
}) {
  const onClose = overrides?.onClose ?? vi.fn();
  const onCreated = overrides?.onCreated ?? vi.fn();
  const result = render(
    <NewCustomerDialog open onClose={onClose} onCreated={onCreated} />,
  );
  return { ...result, onClose, onCreated };
}

describe("NewCustomerDialog", () => {
  beforeEach(() => {
    createCustomerMock.mockReset();
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it("renders nothing while closed", () => {
    // Arrange / Act
    const { container } = render(
      <NewCustomerDialog open={false} onClose={vi.fn()} onCreated={vi.fn()} />,
    );

    // Assert
    expect(container.firstChild).toBeNull();
  });

  it("shows the three associated fields and initially focuses Name", () => {
    // Arrange / Act
    renderDialog();

    // Assert
    expect(screen.getByRole("dialog", { name: "New Customer" })).toBeTruthy();
    expect(screen.getByLabelText("Name")).toBeTruthy();
    expect(screen.getByLabelText("Phone")).toBeTruthy();
    expect(screen.getByLabelText("Notes")).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByLabelText("Name"));
  });

  it("requires both Name and Phone before submission", async () => {
    // Arrange
    const user = userEvent.setup();
    renderDialog();

    // Act
    await user.click(screen.getByRole("button", { name: "Create Customer" }));

    // Assert
    expect(screen.getByText("Name is required.")).toBeTruthy();
    expect(screen.getByText("Phone is required.")).toBeTruthy();
    expect(createCustomerMock).not.toHaveBeenCalled();

    await user.type(screen.getByLabelText("Name"), "Ahmad Ali");
    await user.click(screen.getByRole("button", { name: "Create Customer" }));
    expect(screen.queryByText("Name is required.")).toBeNull();
    expect(screen.getByText("Phone is required.")).toBeTruthy();
    expect(createCustomerMock).not.toHaveBeenCalled();
  });

  it("sends the exact safe payload once, trims Name, keeps the Jordan phone format, and nulls blank Notes", async () => {
    // Arrange
    const user = userEvent.setup();
    const now = 1_725_000_000_000;
    createCustomerMock.mockResolvedValue(createdCustomer);
    renderDialog();

    // Act
    await user.type(screen.getByLabelText("Name"), "  Ahmad Ali  ");
    await user.type(screen.getByLabelText("Phone"), "  00962791234567  ");
    await user.type(screen.getByLabelText("Notes"), "   ");
    const createButton = screen.getByRole("button", { name: "Create Customer" });
    vi.spyOn(Date, "now").mockReturnValue(now);
    createButton.click();

    // Assert
    await waitFor(() => expect(createCustomerMock).toHaveBeenCalledTimes(1));
    expect(createCustomerMock).toHaveBeenCalledWith({
      name: "Ahmad Ali",
      phone: "00962791234567",
      notes: null,
      createdAt: now,
    });
    expect(Object.keys(createCustomerMock.mock.calls[0][0]).sort()).toEqual(
      ["name", "phone", "notes", "createdAt"].sort(),
    );
  });

  it("trims nonblank Notes without canonicalizing a local phone", async () => {
    // Arrange
    const user = userEvent.setup();
    createCustomerMock.mockResolvedValue(createdCustomer);
    renderDialog();

    // Act
    await user.type(screen.getByLabelText("Name"), "Lina Saleh");
    await user.type(screen.getByLabelText("Phone"), "079 123 4567");
    await user.type(screen.getByLabelText("Notes"), "  Prefers afternoon calls  ");
    await user.click(screen.getByRole("button", { name: "Create Customer" }));

    // Assert
    await waitFor(() => expect(createCustomerMock).toHaveBeenCalledTimes(1));
    expect(createCustomerMock.mock.calls[0][0]).toMatchObject({
      phone: "079 123 4567",
      notes: "Prefers afternoon calls",
    });
  });

  it("disables Create while pending and ignores rapid duplicate submission", async () => {
    // Arrange
    const user = userEvent.setup();
    const creation = deferred<CustomerSummary>();
    createCustomerMock.mockReturnValue(creation.promise);
    renderDialog();
    await user.type(screen.getByLabelText("Name"), "Ahmad Ali");
    await user.type(screen.getByLabelText("Phone"), "0791234567");
    const createButton = screen.getByRole("button", { name: "Create Customer" });

    // Act
    await user.click(createButton);
    fireEvent.click(createButton);

    // Assert
    expect((createButton as HTMLButtonElement).disabled).toBe(true);
    expect(createCustomerMock).toHaveBeenCalledTimes(1);
    creation.resolve(createdCustomer);
  });

  it("returns the Customer, resets, and closes after successful creation", async () => {
    // Arrange
    const user = userEvent.setup();
    const onCreated = vi.fn();
    const onClose = vi.fn();
    createCustomerMock.mockResolvedValue(createdCustomer);
    const { rerender } = renderDialog({ onCreated, onClose });
    await user.type(screen.getByLabelText("Name"), "Ahmad Ali");
    await user.type(screen.getByLabelText("Phone"), "0791234567");

    // Act
    await user.click(screen.getByRole("button", { name: "Create Customer" }));

    // Assert
    await waitFor(() => expect(onCreated).toHaveBeenCalledWith(createdCustomer));
    expect(onClose).toHaveBeenCalledTimes(1);
    rerender(<NewCustomerDialog open={false} onClose={onClose} onCreated={onCreated} />);
    rerender(<NewCustomerDialog open onClose={onClose} onCreated={onCreated} />);
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe("");
    expect((screen.getByLabelText("Phone") as HTMLInputElement).value).toBe("");
    expect((screen.getByLabelText("Notes") as HTMLTextAreaElement).value).toBe("");
  });

  it.each([
    [
      "customerPhoneAlreadyExists",
      "A customer with this phone number already exists.",
    ],
    ["validationError", "Please review the customer details and try again."],
    ["databaseError", "The customer could not be saved. Please try again."],
  ] as const)("shows safe %s feedback", async (category, expectedMessage) => {
    // Arrange
    const user = userEvent.setup();
    createCustomerMock.mockRejectedValue(
      new ServiceVisitCommandError({ category, message: "raw backend detail" }),
    );
    renderDialog();
    await user.type(screen.getByLabelText("Name"), "Ahmad Ali");
    await user.type(screen.getByLabelText("Phone"), "0791234567");

    // Act
    await user.click(screen.getByRole("button", { name: "Create Customer" }));

    // Assert
    expect(await screen.findByText(expectedMessage)).toBeTruthy();
    expect(screen.queryByText("raw backend detail")).toBeNull();
  });

  it("shows a generic safe error for an unexpected rejection", async () => {
    // Arrange
    const user = userEvent.setup();
    createCustomerMock.mockRejectedValue(new Error("stack, SQL, and secret"));
    renderDialog();
    await user.type(screen.getByLabelText("Name"), "Ahmad Ali");
    await user.type(screen.getByLabelText("Phone"), "0791234567");

    // Act
    await user.click(screen.getByRole("button", { name: "Create Customer" }));

    // Assert
    expect(await screen.findByText("Something went wrong. Please try again.")).toBeTruthy();
    expect(screen.queryByText("stack, SQL, and secret")).toBeNull();
  });

  it("only closes from the actual backdrop and resets when closed", async () => {
    // Arrange
    const user = userEvent.setup();
    const onClose = vi.fn();
    const { rerender } = renderDialog({ onClose });
    await user.type(screen.getByLabelText("Name"), "Unsaved Name");
    const dialog = screen.getByRole("dialog", { name: "New Customer" });
    const backdrop = dialog.parentElement as HTMLElement;

    // Act / Assert
    fireEvent.click(dialog);
    expect(onClose).not.toHaveBeenCalled();
    fireEvent.click(backdrop);
    expect(onClose).toHaveBeenCalledTimes(1);

    rerender(<NewCustomerDialog open={false} onClose={onClose} onCreated={vi.fn()} />);
    rerender(<NewCustomerDialog open onClose={onClose} onCreated={vi.fn()} />);
    expect((screen.getByLabelText("Name") as HTMLInputElement).value).toBe("");
  });

  it("closes on Escape and resets transient form state", async () => {
    // Arrange
    const user = userEvent.setup();
    const onClose = vi.fn();
    const { rerender } = renderDialog({ onClose });
    await user.type(screen.getByLabelText("Phone"), "0791234567");

    // Act
    fireEvent.keyDown(document, { key: "Escape" });

    // Assert
    expect(onClose).toHaveBeenCalledTimes(1);
    rerender(<NewCustomerDialog open={false} onClose={onClose} onCreated={vi.fn()} />);
    rerender(<NewCustomerDialog open onClose={onClose} onCreated={vi.fn()} />);
    expect((screen.getByLabelText("Phone") as HTMLInputElement).value).toBe("");
  });
});

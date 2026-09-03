// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { issueInvoice, loadServiceVisitInvoice } from "../invoices/api/invoiceApi";
import type { ServiceVisitWorkspace } from "./api/serviceVisitApi";
import { ServiceVisitPage } from "./ServiceVisitPage";

vi.mock("../invoices/api/invoiceApi", async () => {
  const actual = await vi.importActual<typeof import("../invoices/api/invoiceApi")>("../invoices/api/invoiceApi");
  return { ...actual, issueInvoice: vi.fn(), loadServiceVisitInvoice: vi.fn() };
});

const workspace: ServiceVisitWorkspace = {
  visit: {
    id: 31,
    motorcycleId: 11,
    ownerCustomerId: 7,
    status: "CLOSED",
    openedAt: 1_725_000_000_000,
    completedAt: 1_725_000_100_000,
    closedAt: 1_725_000_200_000,
    cancelledAt: null,
    odometerKm: 15_200,
    customerComplaint: "Oil leak",
    diagnosis: "Filter seal leak",
    workPerformed: "Replaced the filter",
    laborChargeFils: 5_000,
    cancellationReason: null,
    notes: null,
    createdAt: 1_725_000_000_000,
    updatedAt: 1_725_000_200_000,
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
  parts: [
    {
      id: 1,
      serviceVisitId: 31,
      inventoryItemId: 4,
      itemName: "Engine oil",
      unitName: "L",
      quantity: 2_000,
      quantityScale: 1_000,
      unitPriceFils: 6_000,
      lineTotalFils: 12_000,
      status: "ACTIVE",
      voidedAt: null,
      voidReason: null,
      createdAt: 1_725_000_010_000,
    },
    {
      id: 2,
      serviceVisitId: 31,
      inventoryItemId: 5,
      itemName: "Old filter",
      unitName: "Piece",
      quantity: 1,
      quantityScale: 1,
      unitPriceFils: 4_500,
      lineTotalFils: 4_500,
      status: "VOIDED",
      voidedAt: 1_725_000_020_000,
      voidReason: "Selected by mistake",
      createdAt: 1_725_000_015_000,
    },
  ],
};

describe("ServiceVisitPage", () => {
  beforeEach(() => {
    vi.mocked(loadServiceVisitInvoice).mockResolvedValue({ id: 4, serviceVisitId: 31,
      status: "DRAFT", invoiceNumber: null, issuedAt: null, customerName: "Ahmad Ali",
      customerPhone: "+962791234567", motorcycleMakeName: "Honda", motorcycleModel: "CB150R",
      motorcyclePlateNumber: "29-12345", motorcycleVin: null, motorcycleChassisNumber: null,
      laborChargeFils: 5_000, partsTotalFils: 12_000, totalFils: 17_000, notes: null,
      lines: [] });
    vi.mocked(issueInvoice).mockResolvedValue({ id: 4, serviceVisitId: 31,
      status: "ISSUED", invoiceNumber: "INV-000004", issuedAt: 2_000, customerName: "Ahmad Ali",
      customerPhone: "+962791234567", motorcycleMakeName: "Honda", motorcycleModel: "CB150R",
      motorcyclePlateNumber: "29-12345", motorcycleVin: null, motorcycleChassisNumber: null,
      laborChargeFils: 5_000, partsTotalFils: 12_000, totalFils: 17_000, notes: null,
      lines: [] });
  });
  afterEach(() => cleanup());

  it("renders the real workspace and excludes voided lines from the service total", () => {
    // Arrange
    render(<ServiceVisitPage workspace={workspace} onBack={vi.fn()} />);

    // Act
    const visitHeading = screen.getByRole("heading", { name: "Service Visit #31" });

    // Assert
    expect(visitHeading).toBeTruthy();
    expect(screen.getByText("Filter seal leak")).toBeTruthy();
    expect(screen.getByText("Engine oil")).toBeTruthy();
    expect(screen.getByText("Old filter")).toBeTruthy();
    expect(screen.getByText("17.000 JD")).toBeTruthy();
  });

  it("issues the completed visit through the typed Invoice API and opens the result", async () => {
    const user = userEvent.setup();
    const onOpenInvoice = vi.fn();
    render(<ServiceVisitPage workspace={workspace} onBack={vi.fn()} onOpenInvoice={onOpenInvoice} />);
    await user.click(await screen.findByRole("button", { name: "Create Invoice" }));
    expect(issueInvoice).toHaveBeenCalledWith({ serviceVisitId: 31, issuedAt: expect.any(Number) });
    await waitFor(() => expect(onOpenInvoice).toHaveBeenCalledWith(4));
  });
});

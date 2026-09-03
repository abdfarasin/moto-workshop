// @vitest-environment jsdom

import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { loadInvoiceDetails } from "./api/invoiceApi";
import { InvoiceDetailsPage } from "./InvoiceDetailsPage";

vi.mock("./api/invoiceApi", async () => {
  const actual = await vi.importActual<typeof import("./api/invoiceApi")>("./api/invoiceApi");
  return { ...actual, loadInvoiceDetails: vi.fn() };
});
const loadMock = vi.mocked(loadInvoiceDetails);

describe("InvoiceDetailsPage", () => {
  afterEach(() => cleanup());
  it("renders authoritative snapshot totals and opens the linked Service Visit", async () => {
    loadMock.mockResolvedValue({ id: 4, serviceVisitId: 31, status: "ISSUED",
      invoiceNumber: "INV-000004", issuedAt: 2_000, customerName: "Ahmad Ali",
      customerPhone: "+962791234567", motorcycleMakeName: "Honda", motorcycleModel: "CB150R",
      motorcyclePlateNumber: "29-12345", motorcycleVin: null, motorcycleChassisNumber: null,
      laborChargeFils: 12_500, partsTotalFils: 9_000, totalFils: 21_500, notes: null,
      lines: [{ serviceVisitPartId: 8, itemName: "Oil Filter", unitName: "Piece",
        quantity: 2, quantityScale: 1, unitPriceFils: 4_500, lineTotalFils: 9_000 }] });
    const user = userEvent.setup(); const openVisit = vi.fn();
    render(<InvoiceDetailsPage invoiceId={4} onBack={vi.fn()} onOpenServiceVisit={openVisit} />);
    expect(await screen.findByText("INV-000004")).toBeTruthy();
    expect(screen.getByText("21.500 JD")).toBeTruthy();
    expect(screen.getByText("Oil Filter")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Open Service Visit" }));
    expect(openVisit).toHaveBeenCalledWith(31);
  });
});

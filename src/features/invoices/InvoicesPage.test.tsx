// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { listInvoices } from "./api/invoiceApi";
import { InvoicesPage } from "./InvoicesPage";

vi.mock("./api/invoiceApi", async () => {
  const actual = await vi.importActual<typeof import("./api/invoiceApi")>("./api/invoiceApi");
  return { ...actual, listInvoices: vi.fn() };
});
const listMock = vi.mocked(listInvoices);

describe("InvoicesPage", () => {
  beforeEach(() => listMock.mockReset());
  afterEach(() => cleanup());

  it("loads a bounded database directory and opens the persisted invoice ID", async () => {
    listMock.mockResolvedValue([{ id: 4, serviceVisitId: 31, status: "ISSUED",
      invoiceNumber: "INV-000004", issuedAt: 2_000, customerName: "Ahmad Ali",
      customerPhone: "+962791234567", motorcycle: "Honda CB150R",
      plateNumber: "29-12345", totalFils: 21_500 }]);
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<InvoicesPage onSelectInvoice={onSelect} />);
    expect(await screen.findByText("INV-000004")).toBeTruthy();
    expect(listMock).toHaveBeenCalledWith({ query: "", statusFilter: "ALL", limit: 50 });
    expect(screen.getByText("21.500 JD")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Open Invoice 4" }));
    expect(onSelect).toHaveBeenCalledWith(4);
  });

  it("sends search and status filters to SQLite and keeps failures safe", async () => {
    listMock.mockResolvedValue([]);
    const user = userEvent.setup();
    render(<InvoicesPage onSelectInvoice={vi.fn()} />);
    await screen.findByText("No invoices found");
    await user.selectOptions(screen.getByLabelText("Invoice status"), "DRAFT");
    await user.type(screen.getByLabelText("Search Invoices"), "29-12345");
    await user.click(screen.getByRole("button", { name: "Search" }));
    await waitFor(() => expect(listMock).toHaveBeenLastCalledWith({ query: "29-12345", statusFilter: "DRAFT", limit: 50 }));
    listMock.mockRejectedValueOnce(new Error("sqlite internals"));
    await user.selectOptions(screen.getByLabelText("Invoice status"), "CANCELLED");
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.queryByText("sqlite internals")).toBeNull();
  });
});

import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  InvoiceCommandError,
  issueInvoice,
  listInvoices,
  loadInvoiceDetails,
  loadServiceVisitInvoice,
  UnexpectedInvoiceApiError,
} from "./invoiceApi";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const invokeMock = vi.mocked(invoke);

describe("invoice API", () => {
  beforeEach(() => invokeMock.mockReset());

  it("uses exact invoice commands and argument shapes", async () => {
    invokeMock.mockResolvedValue({ id: 3 });
    const listInput = { query: "Ahmad", statusFilter: "ISSUED" as const, limit: 50 };
    const issueInput = { serviceVisitId: 9, issuedAt: 2000 };
    await listInvoices(listInput);
    await loadInvoiceDetails(3);
    await loadServiceVisitInvoice(9);
    await issueInvoice(issueInput);
    expect(invokeMock).toHaveBeenNthCalledWith(1, "list_invoices", { input: listInput });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "load_invoice_details", { invoiceId: 3 });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "load_service_visit_invoice", { serviceVisitId: 9 });
    expect(invokeMock).toHaveBeenNthCalledWith(4, "issue_invoice", { input: issueInput });
  });

  it("preserves known command errors and distinguishes transport failures", async () => {
    invokeMock.mockRejectedValueOnce({ category: "invoiceAlreadyIssued", message: "Already issued." });
    const known = await issueInvoice({ serviceVisitId: 9, issuedAt: 2000 }).catch((error) => error);
    expect(known).toBeInstanceOf(InvoiceCommandError);
    expect(known).toMatchObject({ category: "invoiceAlreadyIssued", message: "Already issued." });
    invokeMock.mockRejectedValueOnce("offline");
    const unexpected = await loadInvoiceDetails(3).catch((error) => error);
    expect(unexpected).toBeInstanceOf(UnexpectedInvoiceApiError);
    expect(unexpected).toMatchObject({ cause: "offline" });
  });
});

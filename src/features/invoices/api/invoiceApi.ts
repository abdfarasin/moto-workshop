import { invoke } from "@tauri-apps/api/core";

import type {
  InvoiceCommandErrorCategory,
  InvoiceCommandErrorPayload,
  InvoiceDetails,
  InvoiceDirectoryEntry,
  IssueInvoiceInput,
  ListInvoicesInput,
} from "./invoiceApi.types";

const categories: readonly InvoiceCommandErrorCategory[] = [
  "invoiceNotFound",
  "invoiceAlreadyIssued",
  "serviceVisitNotInvoiceable",
  "validationError",
  "databaseError",
];

export class InvoiceCommandError extends Error {
  readonly category: InvoiceCommandErrorCategory;
  constructor(payload: InvoiceCommandErrorPayload) {
    super(payload.message);
    this.name = "InvoiceCommandError";
    this.category = payload.category;
  }
}

export class UnexpectedInvoiceApiError extends Error {
  readonly cause: unknown;
  constructor(cause: unknown) {
    super("The Invoice command failed unexpectedly.");
    this.name = "UnexpectedInvoiceApiError";
    this.cause = cause;
  }
}

function isPayload(error: unknown): error is InvoiceCommandErrorPayload {
  if (typeof error !== "object" || error === null) return false;
  const candidate = error as Partial<InvoiceCommandErrorPayload>;
  return typeof candidate.message === "string" && typeof candidate.category === "string" &&
    categories.includes(candidate.category as InvoiceCommandErrorCategory);
}

async function invokeInvoice<T>(command: string, args: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    if (isPayload(error)) throw new InvoiceCommandError(error);
    throw new UnexpectedInvoiceApiError(error);
  }
}

export function listInvoices(input: ListInvoicesInput): Promise<InvoiceDirectoryEntry[]> {
  return invokeInvoice("list_invoices", { input });
}
export function loadInvoiceDetails(invoiceId: number): Promise<InvoiceDetails> {
  return invokeInvoice("load_invoice_details", { invoiceId });
}
export function loadServiceVisitInvoice(serviceVisitId: number): Promise<InvoiceDetails> {
  return invokeInvoice("load_service_visit_invoice", { serviceVisitId });
}
export function issueInvoice(input: IssueInvoiceInput): Promise<InvoiceDetails> {
  return invokeInvoice("issue_invoice", { input });
}

export type {
  InvoiceCommandErrorCategory,
  InvoiceDetails,
  InvoiceDirectoryEntry,
  InvoiceDirectoryStatusFilter,
  InvoiceLine,
  InvoiceStatus,
  IssueInvoiceInput,
  ListInvoicesInput,
} from "./invoiceApi.types";

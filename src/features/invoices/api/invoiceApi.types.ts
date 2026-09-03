export type InvoiceStatus = "DRAFT" | "ISSUED" | "CANCELLED";
export type InvoiceDirectoryStatusFilter = "ALL" | InvoiceStatus;

export interface ListInvoicesInput {
  query: string;
  statusFilter: InvoiceDirectoryStatusFilter;
  limit?: number;
}

export interface InvoiceDirectoryEntry {
  id: number;
  serviceVisitId: number;
  status: InvoiceStatus;
  invoiceNumber: string | null;
  issuedAt: number | null;
  customerName: string;
  customerPhone: string;
  motorcycle: string;
  plateNumber: string | null;
  totalFils: number;
}

export interface InvoiceLine {
  serviceVisitPartId: number;
  itemName: string;
  unitName: string;
  quantity: number;
  quantityScale: number;
  unitPriceFils: number;
  lineTotalFils: number;
}

export interface InvoiceDetails {
  id: number;
  serviceVisitId: number;
  status: InvoiceStatus;
  invoiceNumber: string | null;
  issuedAt: number | null;
  customerName: string;
  customerPhone: string;
  motorcycleMakeName: string;
  motorcycleModel: string;
  motorcyclePlateNumber: string | null;
  motorcycleVin: string | null;
  motorcycleChassisNumber: string | null;
  laborChargeFils: number;
  partsTotalFils: number;
  totalFils: number;
  notes: string | null;
  lines: InvoiceLine[];
}

export interface IssueInvoiceInput {
  serviceVisitId: number;
  issuedAt: number;
}

export type InvoiceCommandErrorCategory =
  | "invoiceNotFound"
  | "invoiceAlreadyIssued"
  | "serviceVisitNotInvoiceable"
  | "validationError"
  | "databaseError";

export interface InvoiceCommandErrorPayload {
  category: InvoiceCommandErrorCategory;
  message: string;
}

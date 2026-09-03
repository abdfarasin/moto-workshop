import type { ServiceVisitStatus } from "../../service/api/serviceVisitApi";

export interface DashboardSummary {
  activeServiceVisits: number;
  readyForPickupVisits: number;
  customerCount: number;
  motorcycleCount: number;
  lowStockItemCount: number;
  negativeStockItemCount: number;
  issuedInvoiceCountToday: number;
  issuedInvoiceValueTodayFils: number;
}

export interface DashboardServiceVisit {
  id: number;
  customerName: string;
  motorcycle: string;
  plateNumber: string | null;
  openedAt: number;
  status: ServiceVisitStatus;
  complaint: string;
}

export interface DashboardInvoice {
  id: number;
  invoiceNumber: string;
  issuedAt: number;
  customerName: string;
  motorcycle: string;
  totalFils: number;
}

export interface DashboardInventoryAlert {
  id: number;
  itemName: string;
  sku: string | null;
  unitName: string;
  quantityScale: number;
  currentQuantity: number;
  minimumStockQuantity: number;
  negativeStock: boolean;
}

export interface DashboardData {
  summary: DashboardSummary;
  recentServiceVisits: DashboardServiceVisit[];
  recentInvoices: DashboardInvoice[];
  inventoryAlerts: DashboardInventoryAlert[];
}

export type DashboardCommandErrorCategory = "validationError" | "databaseError";
export interface DashboardCommandErrorPayload {
  category: DashboardCommandErrorCategory;
  message: string;
}

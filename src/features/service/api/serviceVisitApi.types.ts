export type ServiceVisitStatus =
  | "OPEN"
  | "READY_FOR_PICKUP"
  | "CLOSED"
  | "CANCELLED";

export type ServiceVisitPartStatus = "ACTIVE" | "VOIDED";

export type ServiceVisitCommandErrorCategory =
  | "motorcycleNotFound"
  | "activeServiceVisitExists"
  | "serviceVisitNotFound"
  | "inventoryItemNotFound"
  | "serviceVisitPartNotFound"
  | "lifecycleRejected"
  | "validationError"
  | "databaseError";

export interface ServiceVisitCommandErrorPayload {
  category: ServiceVisitCommandErrorCategory;
  message: string;
}

export interface ServiceVisitDetails {
  id: number;
  motorcycleId: number;
  ownerCustomerId: number;
  status: ServiceVisitStatus;
  openedAt: number;
  completedAt: number | null;
  closedAt: number | null;
  cancelledAt: number | null;
  odometerKm: number | null;
  customerComplaint: string;
  diagnosis: string | null;
  workPerformed: string | null;
  laborChargeFils: number;
  cancellationReason: string | null;
  notes: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface ServiceVisitOwner {
  id: number;
  name: string;
  phone: string;
}

export interface ServiceVisitMotorcycle {
  id: number;
  makeName: string;
  model: string;
  year: number | null;
  plateCode: string | null;
  plateNumber: number | null;
  vin: string | null;
  chassisNumber: string | null;
  colorName: string;
}

export interface ServiceVisitPart {
  id: number;
  serviceVisitId: number;
  inventoryItemId: number;
  itemName: string;
  unitName: string;
  quantity: number;
  quantityScale: number;
  unitPriceFils: number;
  lineTotalFils: number;
  status: ServiceVisitPartStatus;
  voidedAt: number | null;
  voidReason: string | null;
  createdAt: number;
}

export interface ServiceVisitWorkspace {
  visit: ServiceVisitDetails;
  owner: ServiceVisitOwner;
  motorcycle: ServiceVisitMotorcycle;
  parts: ServiceVisitPart[];
}

export interface InventoryItemSelection {
  id: number;
  itemName: string;
  sku: string | null;
  unitId: number;
  unitName: string;
  quantityScale: number;
  defaultSellingPriceFils: number;
  currentQuantity: number;
}

export interface CreateServiceVisitInput {
  motorcycleId: number;
  openedAt: number;
  odometerKm: number | null;
  customerComplaint: string;
  notes: string | null;
  createdAt: number;
}

export interface UpdateServiceVisitWorkInput {
  serviceVisitId: number;
  diagnosis: string | null;
  workPerformed: string | null;
  laborChargeFils: number;
  notes: string | null;
  odometerKm: number | null;
  updatedAt: number;
}

export interface AddServiceVisitPartInput {
  serviceVisitId: number;
  inventoryItemId: number;
  quantity: number;
  unitPriceFils: number;
  createdAt: number;
}

export interface VoidServiceVisitPartInput {
  serviceVisitId: number;
  serviceVisitPartId: number;
  voidedAt: number;
  reason: string | null;
}

export interface MarkServiceVisitReadyForPickupInput {
  serviceVisitId: number;
  completedAt: number;
  updatedAt: number;
}

export interface ReopenServiceVisitInput {
  serviceVisitId: number;
  updatedAt: number;
}

export interface CloseServiceVisitInput {
  serviceVisitId: number;
  closedAt: number;
  updatedAt: number;
}

export interface CancelServiceVisitInput {
  serviceVisitId: number;
  cancelledAt: number;
  reason: string;
  updatedAt: number;
}

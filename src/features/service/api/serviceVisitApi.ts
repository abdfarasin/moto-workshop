import { invoke } from "@tauri-apps/api/core";

import type {
  AddServiceVisitPartInput,
  CancelServiceVisitInput,
  CloseServiceVisitInput,
  CreateServiceVisitInput,
  CustomerMotorcycleLookup,
  CustomerSummary,
  InventoryItemSelection,
  MarkServiceVisitReadyForPickupInput,
  ReopenServiceVisitInput,
  SearchCustomersInput,
  ServiceVisitCommandErrorCategory,
  ServiceVisitCommandErrorPayload,
  ServiceVisitPart,
  ServiceVisitWorkspace,
  UpdateServiceVisitWorkInput,
  VoidServiceVisitPartInput,
} from "./serviceVisitApi.types";

const commandErrorCategories: readonly ServiceVisitCommandErrorCategory[] = [
  "customerNotFound",
  "motorcycleNotFound",
  "activeServiceVisitExists",
  "serviceVisitNotFound",
  "inventoryItemNotFound",
  "serviceVisitPartNotFound",
  "lifecycleRejected",
  "validationError",
  "databaseError",
];

export class ServiceVisitCommandError extends Error {
  readonly category: ServiceVisitCommandErrorCategory;

  constructor(error: ServiceVisitCommandErrorPayload) {
    super(error.message);
    this.name = "ServiceVisitCommandError";
    this.category = error.category;
  }
}

export class UnexpectedServiceVisitApiError extends Error {
  readonly cause: unknown;

  constructor(cause: unknown) {
    super("The Service Visit command failed unexpectedly.");
    this.name = "UnexpectedServiceVisitApiError";
    this.cause = cause;
  }
}

export function isServiceVisitCommandError(
  error: unknown,
): error is ServiceVisitCommandError {
  return error instanceof ServiceVisitCommandError;
}

function isCommandErrorPayload(
  error: unknown,
): error is ServiceVisitCommandErrorPayload {
  if (typeof error !== "object" || error === null) {
    return false;
  }

  const candidate = error as Partial<ServiceVisitCommandErrorPayload>;

  return (
    typeof candidate.message === "string" &&
    typeof candidate.category === "string" &&
    commandErrorCategories.includes(
      candidate.category as ServiceVisitCommandErrorCategory,
    )
  );
}

async function invokeServiceVisitCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return args === undefined
      ? await invoke<T>(command)
      : await invoke<T>(command, args);
  } catch (error: unknown) {
    if (isCommandErrorPayload(error)) {
      throw new ServiceVisitCommandError(error);
    }

    throw new UnexpectedServiceVisitApiError(error);
  }
}

export function loadServiceVisitWorkspace(
  serviceVisitId: number,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>(
    "load_service_visit_workspace",
    { serviceVisitId },
  );
}

export function createServiceVisit(
  input: CreateServiceVisitInput,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>("create_service_visit", {
    input,
  });
}

export function searchCustomers(
  input: SearchCustomersInput,
): Promise<CustomerSummary[]> {
  return invokeServiceVisitCommand<CustomerSummary[]>("search_customers", {
    input,
  });
}

export function listCustomerMotorcycles(
  customerId: number,
): Promise<CustomerMotorcycleLookup[]> {
  return invokeServiceVisitCommand<CustomerMotorcycleLookup[]>(
    "list_customer_motorcycles",
    { input: { customerId } },
  );
}

export function listServiceVisitInventoryItems(): Promise<
  InventoryItemSelection[]
> {
  return invokeServiceVisitCommand<InventoryItemSelection[]>(
    "list_service_visit_inventory_items",
  );
}

export function updateServiceVisitWork(
  input: UpdateServiceVisitWorkInput,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>(
    "update_service_visit_work",
    { input },
  );
}

export function addServiceVisitPart(
  input: AddServiceVisitPartInput,
): Promise<ServiceVisitPart> {
  return invokeServiceVisitCommand<ServiceVisitPart>("add_service_visit_part", {
    input,
  });
}

export function voidServiceVisitPart(
  input: VoidServiceVisitPartInput,
): Promise<ServiceVisitPart> {
  return invokeServiceVisitCommand<ServiceVisitPart>(
    "void_service_visit_part",
    { input },
  );
}

export function markServiceVisitReadyForPickup(
  input: MarkServiceVisitReadyForPickupInput,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>(
    "mark_service_visit_ready_for_pickup",
    { input },
  );
}

export function reopenServiceVisit(
  input: ReopenServiceVisitInput,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>(
    "reopen_service_visit",
    { input },
  );
}

export function closeServiceVisit(
  input: CloseServiceVisitInput,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>("close_service_visit", {
    input,
  });
}

export function cancelServiceVisit(
  input: CancelServiceVisitInput,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>(
    "cancel_service_visit",
    { input },
  );
}

export type {
  ActiveServiceVisitStatus,
  AddServiceVisitPartInput,
  CancelServiceVisitInput,
  CloseServiceVisitInput,
  CreateServiceVisitInput,
  CustomerMotorcycleLookup,
  CustomerSummary,
  InventoryItemSelection,
  MarkServiceVisitReadyForPickupInput,
  ReopenServiceVisitInput,
  SearchCustomersInput,
  ServiceVisitCommandErrorCategory,
  ServiceVisitCommandErrorPayload,
  ServiceVisitDetails,
  ServiceVisitMotorcycle,
  ServiceVisitOwner,
  ServiceVisitPart,
  ServiceVisitPartStatus,
  ServiceVisitStatus,
  ServiceVisitWorkspace,
  UpdateServiceVisitWorkInput,
  VoidServiceVisitPartInput,
} from "./serviceVisitApi.types";

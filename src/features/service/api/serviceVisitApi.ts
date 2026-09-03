import { invoke } from "@tauri-apps/api/core";

import type {
  AddServiceVisitPartInput,
  CancelServiceVisitInput,
  CloseServiceVisitInput,
  CreateCustomerInput,
  CreateMotorcycleInput,
  CreateServiceVisitInput,
  CustomerMotorcycleLookup,
  CustomerSummary,
  InventoryItemSelection,
  ListServiceVisitsInput,
  MarkServiceVisitReadyForPickupInput,
  MotorcycleRegistrationReferenceData,
  ReopenServiceVisitInput,
  SearchCustomersInput,
  ServiceVisitCommandErrorCategory,
  ServiceVisitCommandErrorPayload,
  ServiceVisitPart,
  ServiceVisitDirectoryEntry,
  ServiceVisitWorkspace,
  UpdateServiceVisitWorkInput,
  VoidServiceVisitPartInput,
} from "./serviceVisitApi.types";

const commandErrorCategories: readonly ServiceVisitCommandErrorCategory[] = [
  "customerNotFound",
  "customerPhoneAlreadyExists",
  "motorcycleIdentityAlreadyExists",
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

export function listServiceVisits(
  input: ListServiceVisitsInput,
): Promise<ServiceVisitDirectoryEntry[]> {
  return invokeServiceVisitCommand<ServiceVisitDirectoryEntry[]>(
    "list_service_visits",
    { input },
  );
}

export function createServiceVisit(
  input: CreateServiceVisitInput,
): Promise<ServiceVisitWorkspace> {
  return invokeServiceVisitCommand<ServiceVisitWorkspace>("create_service_visit", {
    input,
  });
}

export function createCustomer(
  input: CreateCustomerInput,
): Promise<CustomerSummary> {
  return invokeServiceVisitCommand<CustomerSummary>("create_customer", { input });
}

export function createMotorcycle(
  input: CreateMotorcycleInput,
): Promise<CustomerMotorcycleLookup> {
  return invokeServiceVisitCommand<CustomerMotorcycleLookup>(
    "create_motorcycle",
    { input },
  );
}

export function loadMotorcycleRegistrationReferenceData(): Promise<MotorcycleRegistrationReferenceData> {
  return invokeServiceVisitCommand<MotorcycleRegistrationReferenceData>(
    "load_motorcycle_registration_reference_data",
  );
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
  CreateCustomerInput,
  CreateMotorcycleInput,
  CreateServiceVisitInput,
  CustomerMotorcycleLookup,
  CustomerSummary,
  InventoryItemSelection,
  ListServiceVisitsInput,
  MarkServiceVisitReadyForPickupInput,
  MotorcycleColorReference,
  MotorcycleMakeReference,
  MotorcycleRegistrationReferenceData,
  ReopenServiceVisitInput,
  SearchCustomersInput,
  ServiceVisitCommandErrorCategory,
  ServiceVisitCommandErrorPayload,
  ServiceVisitDetails,
  ServiceVisitDirectoryEntry,
  ServiceVisitDirectoryStatusFilter,
  ServiceVisitMotorcycle,
  ServiceVisitOwner,
  ServiceVisitPart,
  ServiceVisitPartStatus,
  ServiceVisitStatus,
  ServiceVisitWorkspace,
  UpdateServiceVisitWorkInput,
  VoidServiceVisitPartInput,
} from "./serviceVisitApi.types";

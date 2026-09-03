import { invoke } from "@tauri-apps/api/core";

export type CustomerDirectoryEntry = {
  id: number;
  name: string;
  phone: string;
  motorcycleCount: number;
  lastVisitAt: number | null;
};

export type SearchCustomerDirectoryInput = {
  query: string;
  limit?: number;
};

export type CustomerDetailsMotorcycle = {
  id: number;
  makeName: string;
  model: string;
  year: number | null;
  plateNumber: string | null;
  vin: string | null;
  chassisNumber: string | null;
  colorName: string;
};

export type CustomerServiceHistoryEntry = {
  id: number;
  motorcycleId: number;
  openedAt: number;
  odometerKm: number | null;
  customerComplaint: string;
  status: string;
  totalFils: number;
};

export type CustomerDetails = {
  id: number;
  name: string;
  phone: string;
  motorcycles: CustomerDetailsMotorcycle[];
  serviceHistory: CustomerServiceHistoryEntry[];
};

export class CustomerDirectoryApiError extends Error {
  readonly cause: unknown;

  constructor(cause: unknown) {
    super("Could not load customer information.");
    this.name = "CustomerDirectoryApiError";
    this.cause = cause;
  }
}

export async function searchCustomerDirectory(
  input: SearchCustomerDirectoryInput,
): Promise<CustomerDirectoryEntry[]> {
  try {
    return await invoke<CustomerDirectoryEntry[]>(
      "search_customer_directory",
      {
        input: {
          query: input.query,
          limit: input.limit ?? null,
        },
      },
    );
  } catch (error: unknown) {
    throw new CustomerDirectoryApiError(error);
  }
}

export async function loadCustomerDetails(
  customerId: number,
): Promise<CustomerDetails> {
  try {
    return await invoke<CustomerDetails>(
      "load_customer_details",
      {
        input: {
          customerId,
        },
      },
    );
  } catch (error: unknown) {
    throw new CustomerDirectoryApiError(error);
  }
}
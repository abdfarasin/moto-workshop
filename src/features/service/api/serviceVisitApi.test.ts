import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, expectTypeOf, test, vi } from "vitest";

import {
  addServiceVisitPart,
  cancelServiceVisit,
  closeServiceVisit,
  createCustomer,
  createServiceVisit,
  isServiceVisitCommandError,
  listCustomerMotorcycles,
  listServiceVisitInventoryItems,
  loadServiceVisitWorkspace,
  loadMotorcycleRegistrationReferenceData,
  markServiceVisitReadyForPickup,
  reopenServiceVisit,
  ServiceVisitCommandError,
  searchCustomers,
  UnexpectedServiceVisitApiError,
  updateServiceVisitWork,
  voidServiceVisitPart,
} from "./serviceVisitApi";
import type {
  AddServiceVisitPartInput,
  CancelServiceVisitInput,
  CloseServiceVisitInput,
  CreateCustomerInput,
  CreateServiceVisitInput,
  InventoryItemSelection,
  CustomerMotorcycleLookup,
  CustomerSummary,
  MotorcycleRegistrationReferenceData,
  ServiceVisitPart,
  ServiceVisitStatus,
  ServiceVisitWorkspace,
  MarkServiceVisitReadyForPickupInput,
  ReopenServiceVisitInput,
  UpdateServiceVisitWorkInput,
  VoidServiceVisitPartInput,
} from "./serviceVisitApi.types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

const workspace: ServiceVisitWorkspace = {
  visit: {
    id: 7,
    motorcycleId: 11,
    ownerCustomerId: 13,
    status: "OPEN",
    openedAt: 1_000,
    completedAt: null,
    closedAt: null,
    cancelledAt: null,
    odometerKm: 42_000,
    customerComplaint: "Front brake noise",
    diagnosis: "Worn pads",
    workPerformed: null,
    laborChargeFils: 8_000,
    cancellationReason: null,
    notes: null,
    createdAt: 1_000,
    updatedAt: 1_100,
  },
  owner: {
    id: 13,
    name: "Ahmad Ali",
    phone: "+962791234567",
  },
  motorcycle: {
    id: 11,
    makeName: "Honda",
    model: "CB150R",
    year: 2022,
    plateCode: "29",
    plateNumber: 12345,
    vin: null,
    chassisNumber: null,
    colorName: "Black",
  },
  parts: [],
};

const part: ServiceVisitPart = {
  id: 31,
  serviceVisitId: 7,
  inventoryItemId: 17,
  itemName: "Engine Oil",
  unitName: "Liter",
  quantity: 2_500,
  quantityScale: 1_000,
  unitPriceFils: 4_000,
  lineTotalFils: 10_000,
  status: "ACTIVE",
  voidedAt: null,
  voidReason: null,
  createdAt: 1_200,
};

beforeEach(() => {
  invokeMock.mockReset();
});

describe("Service Visit API", () => {
  test("loads a strongly typed workspace with the command's direct argument", async () => {
    // Arrange
    invokeMock.mockResolvedValue(workspace);

    // Act
    const result = await loadServiceVisitWorkspace(7);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("load_service_visit_workspace", {
      serviceVisitId: 7,
    });
    expect(result).toEqual(workspace);
    expectTypeOf(result).toEqualTypeOf<ServiceVisitWorkspace>();
    expectTypeOf(result.visit.status).toEqualTypeOf<ServiceVisitStatus>();
  });

  test("lists inventory with currentQuantity and no command arguments", async () => {
    // Arrange
    const inventory: InventoryItemSelection[] = [
      {
        id: 17,
        itemName: "Engine Oil",
        sku: "OIL-10W40",
        unitId: 5,
        unitName: "Liter",
        quantityScale: 1_000,
        defaultSellingPriceFils: 4_000,
        currentQuantity: 7_500,
      },
    ];
    invokeMock.mockResolvedValue(inventory);

    // Act
    const result = await listServiceVisitInventoryItems();

    // Assert
    expect(invokeMock).toHaveBeenCalledWith(
      "list_service_visit_inventory_items",
    );
    expect(result[0].currentQuantity).toBe(7_500);
    expectTypeOf(result[0].currentQuantity).toEqualTypeOf<number>();
  });

  test("updates work through the Tauri input wrapper", async () => {
    // Arrange
    const input: UpdateServiceVisitWorkInput = {
      serviceVisitId: 7,
      diagnosis: "Worn pads",
      workPerformed: "Replaced front pads",
      laborChargeFils: 8_000,
      notes: null,
      odometerKm: 42_000,
      updatedAt: 1_300,
    };
    invokeMock.mockResolvedValue(workspace);

    // Act
    const result = await updateServiceVisitWork(input);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("update_service_visit_work", {
      input,
    });
    expect(result).toBe(workspace);
  });

  test("adds a part through the Tauri input wrapper", async () => {
    // Arrange
    const input: AddServiceVisitPartInput = {
      serviceVisitId: 7,
      inventoryItemId: 17,
      quantity: 2_500,
      unitPriceFils: 4_000,
      createdAt: 1_200,
    };
    invokeMock.mockResolvedValue(part);

    // Act
    const result = await addServiceVisitPart(input);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("add_service_visit_part", {
      input,
    });
    expect(result).toBe(part);
  });

  test("voids a part through the Tauri input wrapper", async () => {
    // Arrange
    const input: VoidServiceVisitPartInput = {
      serviceVisitId: 7,
      serviceVisitPartId: 31,
      voidedAt: 1_400,
      reason: "Wrong oil selected",
    };
    invokeMock.mockResolvedValue({
      ...part,
      status: "VOIDED",
      voidedAt: 1_400,
      voidReason: "Wrong oil selected",
    });

    // Act
    await voidServiceVisitPart(input);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("void_service_visit_part", {
      input,
    });
  });

  test("invokes each lifecycle command with its exact Tauri input wrapper", async () => {
    // Arrange
    const markReadyInput: MarkServiceVisitReadyForPickupInput = {
      serviceVisitId: 7,
      completedAt: 1_500,
      updatedAt: 1_510,
    };
    const reopenInput: ReopenServiceVisitInput = {
      serviceVisitId: 7,
      updatedAt: 1_520,
    };
    const closeInput: CloseServiceVisitInput = {
      serviceVisitId: 7,
      closedAt: 1_600,
      updatedAt: 1_610,
    };
    const cancelInput: CancelServiceVisitInput = {
      serviceVisitId: 7,
      cancelledAt: 1_700,
      reason: "Customer declined repair",
      updatedAt: 1_710,
    };
    invokeMock.mockResolvedValue(workspace);

    // Act
    const ready = await markServiceVisitReadyForPickup(markReadyInput);
    await reopenServiceVisit(reopenInput);
    await closeServiceVisit(closeInput);
    await cancelServiceVisit(cancelInput);

    // Assert
    expect(invokeMock).toHaveBeenNthCalledWith(
      1,
      "mark_service_visit_ready_for_pickup",
      { input: markReadyInput },
    );
    expect(invokeMock).toHaveBeenNthCalledWith(2, "reopen_service_visit", {
      input: reopenInput,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "close_service_visit", {
      input: closeInput,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(4, "cancel_service_visit", {
      input: cancelInput,
    });
    expectTypeOf(ready).toEqualTypeOf<ServiceVisitWorkspace>();
  });

  test("creates a visit with the exact safe Tauri input wrapper", async () => {
    // Arrange
    const input: CreateServiceVisitInput = {
      motorcycleId: 11,
      openedAt: 2_000,
      odometerKm: 18_750,
      customerComplaint: "Engine stalls",
      notes: null,
      createdAt: 2_100,
    };
    invokeMock.mockResolvedValue(workspace);

    // Act
    const result = await createServiceVisit(input);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("create_service_visit", { input });
    expectTypeOf(result).toEqualTypeOf<ServiceVisitWorkspace>();
  });

  test("creates a Customer with exactly the Tauri input wrapper", async () => {
    // Arrange
    const input: CreateCustomerInput = {
      name: "Ahmad Ali",
      phone: "0791234567",
      notes: null,
      createdAt: 1_234,
    };
    const customer: CustomerSummary = {
      id: 13,
      name: "Ahmad Ali",
      phone: "+962791234567",
    };
    invokeMock.mockResolvedValue(customer);

    // Act
    const result = await createCustomer(input);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("create_customer", { input });
    expect(result).toEqual(customer);
    expectTypeOf(result).toEqualTypeOf<CustomerSummary>();
  });

  test("preserves customerPhoneAlreadyExists across Customer creation", async () => {
    // Arrange
    const input: CreateCustomerInput = {
      name: "Duplicate",
      phone: "00962791234567",
      notes: null,
      createdAt: 2_000,
    };
    invokeMock.mockRejectedValue({
      category: "customerPhoneAlreadyExists",
      message: "A Customer with this phone number already exists.",
    });

    // Act
    const error = await createCustomer(input).catch(
      (rejection: unknown) => rejection,
    );

    // Assert
    expect(error).toBeInstanceOf(ServiceVisitCommandError);
    expect(isServiceVisitCommandError(error)).toBe(true);
    expect(error).toMatchObject({
      category: "customerPhoneAlreadyExists",
      message: "A Customer with this phone number already exists.",
    });
  });

  test("loads Motorcycle registration reference data without command arguments", async () => {
    // Arrange
    const referenceData: MotorcycleRegistrationReferenceData = {
      makes: [{ id: 1, name: "Honda" }],
      colors: [{ id: 2, name: "Black" }],
      plateCodes: [{ id: 3, code: "29" }],
    };
    invokeMock.mockResolvedValue(referenceData);

    // Act
    const result = await loadMotorcycleRegistrationReferenceData();

    // Assert
    expect(invokeMock).toHaveBeenCalledWith(
      "load_motorcycle_registration_reference_data",
    );
    expect(result).toEqual(referenceData);
    expectTypeOf(result).toEqualTypeOf<MotorcycleRegistrationReferenceData>();
  });

  test("searches Customers with the exact lookup command and input wrapper", async () => {
    // Arrange
    const customers: CustomerSummary[] = [
      { id: 13, name: "Ahmad Ali", phone: "+962791234567" },
    ];
    const input = { query: "Ahmad", limit: 25 };
    invokeMock.mockResolvedValue(customers);

    // Act
    const result = await searchCustomers(input);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("search_customers", { input });
    expect(result).toEqual(customers);
    expectTypeOf(result).toEqualTypeOf<CustomerSummary[]>();
  });

  test("lists a Customer's Motorcycles with active Visit state", async () => {
    // Arrange
    const motorcycles: CustomerMotorcycleLookup[] = [
      {
        id: 11,
        makeName: "Honda",
        model: "CB150R",
        year: 2022,
        colorName: "Black",
        plateCode: "29",
        plateNumber: 12345,
        vin: null,
        chassisNumber: null,
        activeServiceVisitId: 7,
        activeServiceVisitStatus: "OPEN",
      },
    ];
    invokeMock.mockResolvedValue(motorcycles);

    // Act
    const result = await listCustomerMotorcycles(13);

    // Assert
    expect(invokeMock).toHaveBeenCalledWith("list_customer_motorcycles", {
      input: { customerId: 13 },
    });
    expect(result).toEqual(motorcycles);
    expectTypeOf(result).toEqualTypeOf<CustomerMotorcycleLookup[]>();
  });

  test("preserves customerNotFound across the typed lookup boundary", async () => {
    // Arrange
    invokeMock.mockRejectedValue({
      category: "customerNotFound",
      message: "The Customer was not found.",
    });

    // Act
    const error = await listCustomerMotorcycles(999_999).catch(
      (rejection: unknown) => rejection,
    );

    // Assert
    expect(error).toBeInstanceOf(ServiceVisitCommandError);
    expect(isServiceVisitCommandError(error)).toBe(true);
    expect(error).toMatchObject({
      category: "customerNotFound",
      message: "The Customer was not found.",
    });
  });

  test.each(["motorcycleNotFound", "activeServiceVisitExists"] as const)(
    "preserves the new %s command error category",
    async (category) => {
      // Arrange
      const input: CreateServiceVisitInput = {
        motorcycleId: 11,
        openedAt: 2_000,
        odometerKm: null,
        customerComplaint: "Engine stalls",
        notes: null,
        createdAt: 2_000,
      };
      invokeMock.mockRejectedValue({ category, message: "Creation rejected." });

      // Act
      const error = await createServiceVisit(input).catch(
        (rejection: unknown) => rejection,
      );

      // Assert
      expect(error).toBeInstanceOf(ServiceVisitCommandError);
      expect(isServiceVisitCommandError(error)).toBe(true);
      expect(error).toMatchObject({ category, message: "Creation rejected." });
    },
  );

  test("preserves a typed backend command error's category and message", async () => {
    // Arrange
    invokeMock.mockRejectedValue({
      category: "lifecycleRejected",
      message: "The Service Visit status does not allow Part changes.",
    });

    // Act
    const rejection = loadServiceVisitWorkspace(7).catch((error: unknown) => error);
    const error = await rejection;

    // Assert
    expect(error).toBeInstanceOf(ServiceVisitCommandError);
    expect(isServiceVisitCommandError(error)).toBe(true);
    expect(error).toMatchObject({
      category: "lifecycleRejected",
      message: "The Service Visit status does not allow Part changes.",
    });
  });

  test("keeps unexpected invoke failures distinct from backend command errors", async () => {
    // Arrange
    const transportFailure = new Error("IPC channel closed");
    invokeMock.mockRejectedValue(transportFailure);

    // Act
    const rejection = listServiceVisitInventoryItems().catch(
      (error: unknown) => error,
    );
    const error = await rejection;

    // Assert
    expect(error).toBeInstanceOf(UnexpectedServiceVisitApiError);
    expect(isServiceVisitCommandError(error)).toBe(false);
    expect(error).toMatchObject({ cause: transportFailure });
  });
});

import { useCallback, useEffect, useRef, useState } from "react";

import { NewCustomerDialog } from "../../customers/new-customer/NewCustomerDialog";
import { NewMotorcycleDialog } from "../../motorcycles/new-motorcycle/NewMotorcycleDialog";
import {
  createServiceVisit,
  isServiceVisitCommandError,
  listCustomerMotorcycles,
  searchCustomers,
} from "../api/serviceVisitApi";
import type {
  CustomerMotorcycleLookup,
  CustomerSummary,
  ServiceVisitCommandErrorCategory,
  ServiceVisitWorkspace,
} from "../api/serviceVisitApi";
import { CustomerSearchStep } from "./CustomerSearchStep";
import { MotorcycleSelectionStep } from "./MotorcycleSelectionStep";
import { VisitDetailsStep } from "./VisitDetailsStep";
import "./NewServiceVisitDialog.css";

export type NewServiceVisitDialogProps = {
  open: boolean;
  initialCustomer?: CustomerSummary;
  initialMotorcycleId?: number;
  onClose: () => void;
  onCreated: (workspace: ServiceVisitWorkspace) => void;
};

const creationMessages: Partial<Record<ServiceVisitCommandErrorCategory, string>> = {
  customerNotFound: "The selected customer is no longer available.",
  motorcycleNotFound: "The selected motorcycle is no longer available.",
  activeServiceVisitExists: "This motorcycle now has an active Service Visit.",
  validationError: "Please review the Visit details and try again.",
  databaseError: "The Service Visit could not be saved. Please try again.",
};

export function NewServiceVisitDialog({
  open,
  initialCustomer,
  initialMotorcycleId,
  onClose,
  onCreated,
}: NewServiceVisitDialogProps) {
  const [query, setQuery] = useState("");
  const [customers, setCustomers] = useState<CustomerSummary[]>([]);
  const [customersLoading, setCustomersLoading] = useState(false);
  const [customerError, setCustomerError] = useState<string | null>(null);
  const [selectedCustomer, setSelectedCustomer] = useState<CustomerSummary | null>(null);
  const [motorcycles, setMotorcycles] = useState<CustomerMotorcycleLookup[]>([]);
  const [motorcyclesLoading, setMotorcyclesLoading] = useState(false);
  const [motorcycleError, setMotorcycleError] = useState<string | null>(null);
  const [selectedMotorcycle, setSelectedMotorcycle] =
    useState<CustomerMotorcycleLookup | null>(null);
  const [newCustomerOpen, setNewCustomerOpen] = useState(false);
  const [newMotorcycleOpen, setNewMotorcycleOpen] = useState(false);
  const [complaint, setComplaint] = useState("");
  const [odometer, setOdometer] = useState("");
  const [notes, setNotes] = useState("");
  const [complaintError, setComplaintError] = useState<string | null>(null);
  const [odometerError, setOdometerError] = useState<string | null>(null);
  const [creationError, setCreationError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const searchRequest = useRef(0);
  const motorcycleRequest = useRef(0);
  const submitGuard = useRef(false);

  const clearVisitDetails = useCallback(() => {
    setComplaint("");
    setOdometer("");
    setNotes("");
    setComplaintError(null);
    setOdometerError(null);
    setCreationError(null);
    setSubmitting(false);
    submitGuard.current = false;
  }, []);

  const resetState = useCallback(() => {
    searchRequest.current += 1;
    motorcycleRequest.current += 1;
    setQuery("");
    setCustomers([]);
    setCustomersLoading(false);
    setCustomerError(null);
    setSelectedCustomer(null);
    setMotorcycles([]);
    setMotorcyclesLoading(false);
    setMotorcycleError(null);
    setSelectedMotorcycle(null);
    setNewCustomerOpen(false);
    setNewMotorcycleOpen(false);
    clearVisitDetails();
  }, [clearVisitDetails]);

  const runCustomerSearch = useCallback(async (rawQuery: string) => {
    const request = ++searchRequest.current;
    setCustomersLoading(true);
    setCustomerError(null);
    setSelectedCustomer(null);
    setMotorcycles([]);
    setSelectedMotorcycle(null);
    motorcycleRequest.current += 1;
    clearVisitDetails();

    try {
      const result = await searchCustomers({ query: rawQuery.trim(), limit: 25 });
      if (request === searchRequest.current) setCustomers(result);
    } catch {
      if (request === searchRequest.current) {
        setCustomers([]);
        setCustomerError("Could not load customers. Please try again.");
      }
    } finally {
      if (request === searchRequest.current) setCustomersLoading(false);
    }
  }, [clearVisitDetails]);

  const selectCustomer = useCallback(async (customer: CustomerSummary) => {
    const request = ++motorcycleRequest.current;
    setSelectedCustomer(customer);
    setMotorcycles([]);
    setMotorcyclesLoading(true);
    setMotorcycleError(null);
    setSelectedMotorcycle(null);
    clearVisitDetails();

    try {
      const result = await listCustomerMotorcycles(customer.id);
      if (request === motorcycleRequest.current) {
        setMotorcycles(result);
        const initialMotorcycle = result.find(
          (motorcycle) =>
            motorcycle.id === initialMotorcycleId &&
            motorcycle.activeServiceVisitId === null,
        );
        if (initialMotorcycle !== undefined) setSelectedMotorcycle(initialMotorcycle);
      }
    } catch {
      if (request === motorcycleRequest.current) {
        setMotorcycles([]);
        setMotorcycleError("Could not load motorcycles. Please try again.");
      }
    } finally {
      if (request === motorcycleRequest.current) setMotorcyclesLoading(false);
    }
  }, [clearVisitDetails, initialMotorcycleId]);

  useEffect(() => {
    if (!open) {
      resetState();
      return;
    }

    resetState();
    if (initialCustomer !== undefined) {
      setQuery(initialCustomer.name);
      setCustomers([initialCustomer]);
      void selectCustomer(initialCustomer);
    } else {
      void runCustomerSearch("");
    }
  }, [initialCustomer, open, resetState, runCustomerSearch, selectCustomer]);

  useEffect(() => {
    if (!open) return;
    function handleKeyDown(event: KeyboardEvent) {
      if (
        event.key === "Escape" &&
        !submitGuard.current &&
        !newCustomerOpen &&
        !newMotorcycleOpen
      ) {
        resetState();
        onClose();
      }
    }
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [newCustomerOpen, newMotorcycleOpen, onClose, open, resetState]);

  function selectMotorcycle(motorcycle: CustomerMotorcycleLookup) {
    if (motorcycle.activeServiceVisitId !== null) return;
    setSelectedMotorcycle(motorcycle);
    clearVisitDetails();
  }

  function closeDialog() {
    if (submitGuard.current) return;
    resetState();
    onClose();
  }

  async function submitVisit() {
    if (submitGuard.current || selectedMotorcycle === null) return;

    const normalizedComplaint = complaint.trim();
    const normalizedOdometer = odometer.trim();
    const normalizedNotes = notes.trim();
    let parsedOdometer: number | null = null;
    let invalid = false;
    let odometerInvalid = false;

    if (normalizedComplaint.length === 0) {
      setComplaintError("Customer Complaint is required.");
      invalid = true;
    } else {
      setComplaintError(null);
    }

    if (normalizedOdometer.length > 0) {
      if (!/^\d+$/.test(normalizedOdometer)) {
        odometerInvalid = true;
      } else {
        parsedOdometer = Number(normalizedOdometer);
        if (!Number.isSafeInteger(parsedOdometer) || parsedOdometer < 0) {
          odometerInvalid = true;
        }
      }
    }
    if (odometerInvalid) invalid = true;
    setOdometerError(
      odometerInvalid ? "Odometer must be a nonnegative whole number." : null,
    );
    if (invalid) return;

    submitGuard.current = true;
    setSubmitting(true);
    setCreationError(null);
    const now = Date.now();

    try {
      const created = await createServiceVisit({
        motorcycleId: selectedMotorcycle.id,
        openedAt: now,
        odometerKm: parsedOdometer,
        customerComplaint: normalizedComplaint,
        notes: normalizedNotes.length === 0 ? null : normalizedNotes,
        createdAt: now,
      });
      onCreated(created);
      resetState();
      onClose();
    } catch (error: unknown) {
      const message = isServiceVisitCommandError(error)
        ? creationMessages[error.category] ?? "The Service Visit could not be created."
        : "Something went wrong. Please try again.";
      setCreationError(message);
      setSubmitting(false);
      submitGuard.current = false;
    }
  }

  if (!open) return null;

  return (
    <div
      className="new-visit-backdrop"
      onClick={(event) => {
        if (event.target === event.currentTarget) closeDialog();
      }}
    >
      <div
        className="new-visit-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="new-service-visit-title"
      >
        <header className="new-visit-header">
          <div>
            <span className="new-visit-eyebrow">Service workspace</span>
            <h2 id="new-service-visit-title">New Service Visit</h2>
            <p>
              {initialCustomer === undefined
                ? "Select or create a customer, choose a motorcycle, then record the complaint."
                : "Choose one of this customer's motorcycles, then record the complaint."}
            </p>
          </div>
          <button
            className="new-visit-close"
            type="button"
            aria-label="Close dialog"
            onClick={closeDialog}
          >
            ×
          </button>
        </header>

        <div className="new-visit-content">
          {initialCustomer === undefined ? (
            <CustomerSearchStep
              query={query}
              customers={customers}
              selectedCustomerId={selectedCustomer?.id ?? null}
              loading={customersLoading}
              error={customerError}
              onQueryChange={setQuery}
              onSearch={() => void runCustomerSearch(query)}
              onSelect={(customer) => void selectCustomer(customer)}
              onNewCustomer={() => setNewCustomerOpen(true)}
            />
          ) : (
            <section className="new-visit-step" aria-labelledby="selected-customer-heading">
              <div className="new-visit-step__heading">
                <span className="new-visit-step__number">1</span>
                <div>
                  <h3 id="selected-customer-heading">Customer</h3>
                  <p>{initialCustomer.name}</p>
                </div>
              </div>
            </section>
          )}

          {selectedCustomer ? (
            <MotorcycleSelectionStep
              customer={selectedCustomer}
              motorcycles={motorcycles}
              selectedMotorcycleId={selectedMotorcycle?.id ?? null}
              loading={motorcyclesLoading}
              error={motorcycleError}
              onSelect={selectMotorcycle}
              onAddMotorcycle={() => setNewMotorcycleOpen(true)}
            />
          ) : null}

          {selectedMotorcycle ? (
            <VisitDetailsStep
              motorcycle={selectedMotorcycle}
              complaint={complaint}
              odometer={odometer}
              notes={notes}
              complaintError={complaintError}
              odometerError={odometerError}
              creationError={creationError}
              submitting={submitting}
              onComplaintChange={(value) => {
                setComplaint(value);
                setComplaintError(null);
              }}
              onOdometerChange={(value) => {
                setOdometer(value);
                setOdometerError(null);
              }}
              onNotesChange={setNotes}
              onSubmit={() => void submitVisit()}
            />
          ) : null}
        </div>

        <NewCustomerDialog
          open={newCustomerOpen}
          onClose={() => setNewCustomerOpen(false)}
          onCreated={(customer) => {
            setNewCustomerOpen(false);
            setQuery(customer.name);
            setCustomers([customer]);
            void selectCustomer(customer);
          }}
        />
        {selectedCustomer ? (
          <NewMotorcycleDialog
            open={newMotorcycleOpen}
            customer={selectedCustomer}
            onClose={() => setNewMotorcycleOpen(false)}
            onCreated={(motorcycle) => {
              setMotorcycles((current) => [...current, motorcycle]);
              setSelectedMotorcycle(motorcycle);
              setNewMotorcycleOpen(false);
              clearVisitDetails();
            }}
          />
        ) : null}
      </div>
    </div>
  );
}

import { useCallback, useEffect, useRef, useState } from "react";

import {
  createCustomer,
  isServiceVisitCommandError,
} from "../../service/api/serviceVisitApi";
import type {
  CustomerSummary,
  ServiceVisitCommandErrorCategory,
} from "../../service/api/serviceVisitApi";
import "./NewCustomerDialog.css";

export type NewCustomerDialogProps = {
  open: boolean;
  onClose: () => void;
  onCreated: (customer: CustomerSummary) => void;
};

const creationMessages: Partial<Record<ServiceVisitCommandErrorCategory, string>> = {
  customerPhoneAlreadyExists: "A customer with this phone number already exists.",
  validationError: "Please review the customer details and try again.",
  databaseError: "The customer could not be saved. Please try again.",
};

export function NewCustomerDialog({
  open,
  onClose,
  onCreated,
}: NewCustomerDialogProps) {
  const [name, setName] = useState("");
  const [phone, setPhone] = useState("");
  const [notes, setNotes] = useState("");
  const [nameError, setNameError] = useState<string | null>(null);
  const [phoneError, setPhoneError] = useState<string | null>(null);
  const [creationError, setCreationError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const submitGuard = useRef(false);
  const submissionVersion = useRef(0);

  const resetState = useCallback(() => {
    submissionVersion.current += 1;
    submitGuard.current = false;
    setName("");
    setPhone("");
    setNotes("");
    setNameError(null);
    setPhoneError(null);
    setCreationError(null);
    setSubmitting(false);
  }, []);

  useEffect(() => {
    resetState();
  }, [open, resetState]);

  useEffect(() => {
    if (!open) return;

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        resetState();
        onClose();
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onClose, open, resetState]);

  function closeDialog() {
    resetState();
    onClose();
  }

  async function submitCustomer() {
    if (submitGuard.current) return;

    const trimmedName = name.trim();
    const phoneInput = phone.trim();
    const trimmedNotes = notes.trim();
    const missingName = trimmedName.length === 0;
    const missingPhone = phoneInput.length === 0;

    setNameError(missingName ? "Name is required." : null);
    setPhoneError(missingPhone ? "Phone is required." : null);
    if (missingName || missingPhone) return;

    submitGuard.current = true;
    setSubmitting(true);
    setCreationError(null);
    const requestVersion = ++submissionVersion.current;
    const now = Date.now();

    try {
      const customer = await createCustomer({
        name: trimmedName,
        phone: phoneInput,
        notes: trimmedNotes.length === 0 ? null : trimmedNotes,
        createdAt: now,
      });
      if (requestVersion !== submissionVersion.current) return;

      onCreated(customer);
      resetState();
      onClose();
    } catch (error: unknown) {
      if (requestVersion !== submissionVersion.current) return;

      const message = isServiceVisitCommandError(error)
        ? creationMessages[error.category] ?? "The customer could not be created."
        : "Something went wrong. Please try again.";
      setCreationError(message);
      setSubmitting(false);
      submitGuard.current = false;
    }
  }

  if (!open) return null;

  return (
    <div
      className="new-customer-backdrop"
      onClick={(event) => {
        if (event.target === event.currentTarget) closeDialog();
      }}
    >
      <div
        className="new-customer-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="new-customer-title"
      >
        <header className="new-customer-header">
          <div>
            <span className="new-customer-eyebrow">Customer records</span>
            <h2 id="new-customer-title">New Customer</h2>
            <p>Add the customer’s contact details to the workshop.</p>
          </div>
          <button
            className="new-customer-close"
            type="button"
            aria-label="Close dialog"
            onClick={closeDialog}
          >
            ×
          </button>
        </header>

        <form
          className="new-customer-form"
          noValidate
          onSubmit={(event) => {
            event.preventDefault();
            void submitCustomer();
          }}
        >
          <label className="new-customer-field" htmlFor="new-customer-name">
            <span>
              Name <b aria-hidden="true">*</b>
            </span>
            <input
              autoFocus
              aria-label="Name"
              id="new-customer-name"
              type="text"
              autoComplete="name"
              value={name}
              aria-invalid={nameError !== null}
              aria-describedby={nameError ? "new-customer-name-error" : undefined}
              onChange={(event) => {
                setName(event.target.value);
                setNameError(null);
              }}
            />
            {nameError ? (
              <small className="new-customer-field__error" id="new-customer-name-error">
                {nameError}
              </small>
            ) : null}
          </label>

          <label className="new-customer-field" htmlFor="new-customer-phone">
            <span>
              Phone <b aria-hidden="true">*</b>
            </span>
            <input
              aria-label="Phone"
              id="new-customer-phone"
              type="tel"
              autoComplete="tel"
              value={phone}
              placeholder="079…, +962…, or 00962…"
              aria-invalid={phoneError !== null}
              aria-describedby={phoneError ? "new-customer-phone-error" : undefined}
              onChange={(event) => {
                setPhone(event.target.value);
                setPhoneError(null);
              }}
            />
            {phoneError ? (
              <small className="new-customer-field__error" id="new-customer-phone-error">
                {phoneError}
              </small>
            ) : null}
          </label>

          <label className="new-customer-field" htmlFor="new-customer-notes">
            <span>
              Notes <em>Optional</em>
            </span>
            <textarea
              aria-label="Notes"
              id="new-customer-notes"
              rows={3}
              value={notes}
              onChange={(event) => setNotes(event.target.value)}
            />
          </label>

          {creationError ? (
            <p className="new-customer-submit-error" role="alert">
              {creationError}
            </p>
          ) : null}

          <footer className="new-customer-actions">
            <button
              className="new-customer-button new-customer-button--secondary"
              type="button"
              disabled={submitting}
              onClick={closeDialog}
            >
              Cancel
            </button>
            <button
              className="new-customer-button new-customer-button--primary"
              type="submit"
              disabled={submitting}
            >
              {submitting ? "Creating…" : "Create Customer"}
            </button>
          </footer>
        </form>
      </div>
    </div>
  );
}

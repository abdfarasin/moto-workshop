import { useCallback, useEffect, useRef, useState } from "react";

import {
  createMotorcycle,
  isServiceVisitCommandError,
  loadMotorcycleRegistrationReferenceData,
} from "../../service/api/serviceVisitApi";
import type {
  CustomerMotorcycleLookup,
  CustomerSummary,
  MotorcycleRegistrationReferenceData,
  ServiceVisitCommandErrorCategory,
} from "../../service/api/serviceVisitApi";
import {
  MotorcycleRegistrationForm,
  type MotorcycleFormErrors,
  type MotorcycleFormValues,
} from "./MotorcycleRegistrationForm";

import "./NewMotorcycleDialog.css";

export type NewMotorcycleDialogProps = {
  open: boolean;
  customer: CustomerSummary;
  onClose: () => void;
  onCreated: (motorcycle: CustomerMotorcycleLookup) => void;
};

const initialValues: MotorcycleFormValues = {
  makeId: "",
  model: "",
  year: "",
  colorId: "",
  plateNumber: "",
  vin: "",
  chassisNumber: "",
  notes: "",
};

const initialErrors: MotorcycleFormErrors = {
  make: null,
  model: null,
  year: null,
  color: null,
  plateNumber: null,
  vin: null,
  chassisNumber: null,
};

const errorFieldByValueField: Partial<
  Record<keyof MotorcycleFormValues, keyof MotorcycleFormErrors>
> = {
  makeId: "make",
  model: "model",
  year: "year",
  colorId: "color",
  plateNumber: "plateNumber",
  vin: "vin",
  chassisNumber: "chassisNumber",
};

const creationMessages: Partial<
  Record<ServiceVisitCommandErrorCategory, string>
> = {
  customerNotFound: "The selected customer is no longer available.",
  motorcycleIdentityAlreadyExists:
    "A motorcycle with this plate, VIN, or chassis number already exists.",
  validationError:
    "Some motorcycle details are invalid. Please review the highlighted fields.",
  databaseError:
    "The motorcycle could not be saved. Please try again.",
};

function optionalText(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length === 0 ? null : trimmed;
}

function validateMotorcycleForm(
  values: MotorcycleFormValues,
): MotorcycleFormErrors {
  const model = values.model.trim();
  const year = values.year.trim();
  const plateNumber = values.plateNumber.trim();
  const vin = values.vin.trim();
  const chassisNumber = values.chassisNumber.trim();

  const errors: MotorcycleFormErrors = {
    make: null,
    model: null,
    year: null,
    color: null,
    plateNumber: null,
    vin: null,
    chassisNumber: null,
  };

  if (values.makeId === "") {
    errors.make = "Make is required.";
  }

  if (model === "") {
    errors.model = "Model is required.";
  }

  if (values.colorId === "") {
    errors.color = "Color is required.";
  }

  if (year !== "") {
    if (!/^\d+$/.test(year)) {
      errors.year = "Year must be a whole number.";
    } else {
      const parsedYear = Number(year);

      if (!Number.isSafeInteger(parsedYear)) {
        errors.year = "Year must be a whole number.";
      }
    }
  }

 if (plateNumber === "") {
  errors.plateNumber = "Plate number is required.";
} else if (!/^[0-9-]+$/.test(plateNumber)) {
  errors.plateNumber =
    "Plate number may contain digits and dashes only.";
} else if (
  plateNumber.startsWith("-") ||
  plateNumber.endsWith("-") ||
  plateNumber.includes("--")
) {
  errors.plateNumber =
    "Use dashes only between number groups.";
}

  if (vin !== "") {
    if (!/^[A-Za-z0-9]+$/.test(vin)) {
      errors.vin = "VIN must contain letters and numbers only.";
    } else if (vin.length !== 17) {
      errors.vin = "VIN must be exactly 17 characters.";
    } else if (/[IOQioq]/.test(vin)) {
      errors.vin = "VIN cannot contain I, O, or Q.";
    }
  }

  if (chassisNumber !== "") {
    if (chassisNumber.length > 64) {
      errors.chassisNumber =
        "Chassis number cannot exceed 64 characters.";
    } else if (!/^[A-Za-z0-9./-]+$/.test(chassisNumber)) {
      errors.chassisNumber =
        "Chassis number may contain letters, numbers, -, /, and . only.";
    }
  }

  return errors;
}

function hasErrors(errors: MotorcycleFormErrors): boolean {
  return Object.values(errors).some((error) => error !== null);
}

export function NewMotorcycleDialog({
  open,
  customer,
  onClose,
  onCreated,
}: NewMotorcycleDialogProps) {
  const [references, setReferences] =
    useState<MotorcycleRegistrationReferenceData | null>(null);

  const [referenceLoading, setReferenceLoading] =
    useState(false);

  const [referenceError, setReferenceError] =
    useState<string | null>(null);

  const [values, setValues] =
    useState<MotorcycleFormValues>(initialValues);

  const [errors, setErrors] =
    useState<MotorcycleFormErrors>(initialErrors);

  const [creationError, setCreationError] =
    useState<string | null>(null);

  const [submitting, setSubmitting] = useState(false);

  const referenceVersion = useRef(0);
  const submissionVersion = useRef(0);
  const submitGuard = useRef(false);

  const resetState = useCallback(() => {
    referenceVersion.current += 1;
    submissionVersion.current += 1;
    submitGuard.current = false;

    setReferences(null);
    setReferenceLoading(false);
    setReferenceError(null);
    setValues(initialValues);
    setErrors(initialErrors);
    setCreationError(null);
    setSubmitting(false);
  }, []);

  useEffect(() => {
    resetState();

    if (!open) {
      return;
    }

    const requestVersion = ++referenceVersion.current;

    setReferenceLoading(true);

    void loadMotorcycleRegistrationReferenceData()
      .then((result) => {
        if (requestVersion !== referenceVersion.current) {
          return;
        }

        setReferences(result);

        window.setTimeout(() => {
          document
            .getElementById("new-motorcycle-make")
            ?.focus();
        }, 0);
      })
      .catch(() => {
        if (requestVersion !== referenceVersion.current) {
          return;
        }

        setReferenceError(
          "Could not load motorcycle registration options. Please try again.",
        );
      })
      .finally(() => {
        if (requestVersion === referenceVersion.current) {
          setReferenceLoading(false);
        }
      });
  }, [customer.id, open, resetState]);

  useEffect(() => {
    if (!open) {
      return;
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        resetState();
        onClose();
      }
    }

    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose, open, resetState]);

  function closeDialog() {
    resetState();
    onClose();
  }

  function changeValue(
    field: keyof MotorcycleFormValues,
    value: string,
  ) {
    setValues((current) => ({
      ...current,
      [field]: value,
    }));

    setCreationError(null);

    const errorField = errorFieldByValueField[field];

    if (errorField !== undefined) {
      setErrors((current) => ({
        ...current,
        [errorField]: null,
      }));
    }
  }

  async function submitMotorcycle() {
    if (submitGuard.current || references === null) {
      return;
    }

    const nextErrors = validateMotorcycleForm(values);

    setErrors(nextErrors);

    if (hasErrors(nextErrors)) {
      return;
    }

    const trimmedModel = values.model.trim();
    const trimmedYear = values.year.trim();

    const year =
      trimmedYear === ""
        ? null
        : Number(trimmedYear);

    submitGuard.current = true;
    setSubmitting(true);
    setCreationError(null);

    const requestVersion =
      ++submissionVersion.current;

    const now = Date.now();

    try {
      const motorcycle = await createMotorcycle({
        customerId: customer.id,
        makeId: Number(values.makeId),
        model: trimmedModel,
        year,
        plateNumber: values.plateNumber.trim(),
        vin: optionalText(values.vin),
        chassisNumber: optionalText(values.chassisNumber),
        colorId: Number(values.colorId),
        notes: optionalText(values.notes),
        createdAt: now,
      });

      if (
        requestVersion !== submissionVersion.current
      ) {
        return;
      }

      onCreated(motorcycle);
      resetState();
      onClose();
    } catch (error: unknown) {
      if (
        requestVersion !== submissionVersion.current
      ) {
        return;
      }

      const message =
        isServiceVisitCommandError(error)
          ? creationMessages[error.category] ??
            "The motorcycle could not be created."
          : "Something went wrong. Please try again.";

      setCreationError(message);
      setSubmitting(false);
      submitGuard.current = false;
    }
  }

  if (!open) {
    return null;
  }

  return (
    <div
      className="new-motorcycle-backdrop"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          closeDialog();
        }
      }}
    >
      <div
        className="new-motorcycle-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="new-motorcycle-title"
      >
        <header className="new-motorcycle-header">
          <div>
            <span className="new-motorcycle-eyebrow">
              Motorcycle registration
            </span>

            <h2 id="new-motorcycle-title">
              New Motorcycle
            </h2>

            <p>
              Register a motorcycle to the selected customer.
            </p>
          </div>

          <button
            className="new-motorcycle-close"
            type="button"
            aria-label="Close dialog"
            onClick={closeDialog}
          >
            ×
          </button>
        </header>

        <div className="new-motorcycle-content">
          <section
            className="new-motorcycle-customer"
            aria-label="Selected customer"
          >
            <span>Customer</span>
            <strong>{customer.name}</strong>
            <small>{customer.phone}</small>
          </section>

          {referenceLoading ? (
            <p className="new-motorcycle-state">
              Loading registration options…
            </p>
          ) : null}

          {!referenceLoading && referenceError ? (
            <p
              className="new-motorcycle-state new-motorcycle-state--error"
              role="alert"
            >
              {referenceError}
            </p>
          ) : null}

          {!referenceLoading && references ? (
            <>
              <div
                className="new-motorcycle-catalog-states"
                aria-live="polite"
              >
                {references.makes.length === 0 ? (
                  <p>
                    No motorcycle makes are available.
                  </p>
                ) : null}

                {references.colors.length === 0 ? (
                  <p>
                    No motorcycle colors are available.
                  </p>
                ) : null}


              </div>

              <MotorcycleRegistrationForm
                references={references}
                values={values}
                errors={errors}
                creationError={creationError}
                submitting={submitting}
                onChange={changeValue}
                onSubmit={() => {
                  void submitMotorcycle();
                }}
                onCancel={closeDialog}
              />
            </>
          ) : null}
        </div>
      </div>
    </div>
  );
}

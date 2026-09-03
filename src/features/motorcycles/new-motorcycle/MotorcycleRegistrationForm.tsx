import type { FormEvent } from "react";

import type { MotorcycleRegistrationReferenceData } from "../../service/api/serviceVisitApi";

export type MotorcycleFormValues = {
  makeId: string;
  model: string;
  year: string;
  colorId: string;
  plateNumber: string;
  vin: string;
  chassisNumber: string;
  notes: string;
};

export type MotorcycleFormErrors = {
  make: string | null;
  model: string | null;
  year: string | null;
  color: string | null;
  plateNumber: string | null;
  vin: string | null;
  chassisNumber: string | null;
};

type MotorcycleRegistrationFormProps = {
  references: MotorcycleRegistrationReferenceData;
  values: MotorcycleFormValues;
  errors: MotorcycleFormErrors;
  creationError: string | null;
  submitting: boolean;
  onChange: (field: keyof MotorcycleFormValues, value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
};

type FieldProps = {
  id: string;
  label: string;
  required?: boolean;
  optional?: boolean;
  error?: string | null;
  children: React.ReactNode;
};

function Field({ id, label, required, optional, error, children }: FieldProps) {
  return (
    <label className="new-motorcycle-field" htmlFor={id}>
      <span>
        {label} {required ? <b aria-hidden="true">*</b> : null}
        {optional ? <em>Optional</em> : null}
      </span>
      {children}
      {error ? (
        <small className="new-motorcycle-field__error" id={`${id}-error`}>
          {error}
        </small>
      ) : null}
    </label>
  );
}

export function MotorcycleRegistrationForm({
  references,
  values,
  errors,
  creationError,
  submitting,
  onChange,
  onSubmit,
  onCancel,
}: MotorcycleRegistrationFormProps) {
  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    onSubmit();
  }

  const requiredCatalogsAvailable =
    references.makes.length > 0 && references.colors.length > 0;

  return (
    <form className="new-motorcycle-form" noValidate onSubmit={handleSubmit}>
      <fieldset className="new-motorcycle-group">
        <legend>Motorcycle</legend>
        <div className="new-motorcycle-grid">
          <Field id="new-motorcycle-make" label="Make" required error={errors.make}>
            <select
              aria-label="Make"
              id="new-motorcycle-make"
              value={values.makeId}
              disabled={references.makes.length === 0}
              aria-invalid={errors.make !== null}
              aria-describedby={errors.make ? "new-motorcycle-make-error" : undefined}
              onChange={(event) => onChange("makeId", event.target.value)}
            >
              <option value="">Select make</option>
              {references.makes.map((make) => (
                <option key={make.id} value={make.id}>
                  {make.name}
                </option>
              ))}
            </select>
          </Field>

          <Field id="new-motorcycle-model" label="Model" required error={errors.model}>
            <input
              aria-label="Model"
              id="new-motorcycle-model"
              type="text"
              value={values.model}
              aria-invalid={errors.model !== null}
              aria-describedby={errors.model ? "new-motorcycle-model-error" : undefined}
              onChange={(event) => onChange("model", event.target.value)}
            />
          </Field>

          <Field id="new-motorcycle-year" label="Year" optional error={errors.year}>
            <input
              aria-label="Year"
              id="new-motorcycle-year"
              type="text"
              inputMode="numeric"
              value={values.year}
              placeholder="e.g. 2022"
              aria-invalid={errors.year !== null}
              aria-describedby={errors.year ? "new-motorcycle-year-error" : undefined}
              onChange={(event) => onChange("year", event.target.value)}
            />
          </Field>

          <Field id="new-motorcycle-color" label="Color" required error={errors.color}>
            <select
              aria-label="Color"
              id="new-motorcycle-color"
              value={values.colorId}
              disabled={references.colors.length === 0}
              aria-invalid={errors.color !== null}
              aria-describedby={errors.color ? "new-motorcycle-color-error" : undefined}
              onChange={(event) => onChange("colorId", event.target.value)}
            >
              <option value="">Select color</option>
              {references.colors.map((color) => (
                <option key={color.id} value={color.id}>
                  {color.name}
                </option>
              ))}
            </select>
          </Field>
        </div>
      </fieldset>

      <fieldset className="new-motorcycle-group">
  <legend>Identity</legend>

  <p className="new-motorcycle-hint">
    Plate number is required. VIN and chassis number are optional.
  </p>

  <div className="new-motorcycle-grid">
    <Field
      id="new-motorcycle-plate-number"
      label="Plate Number"
      required
      error={errors.plateNumber}
    >
      <input
        aria-label="Plate Number"
        id="new-motorcycle-plate-number"
        type="text"
        inputMode="numeric"
        placeholder="e.g. 47-122132"
        value={values.plateNumber}
        aria-invalid={errors.plateNumber !== null}
        aria-describedby={
          errors.plateNumber
            ? "new-motorcycle-plate-number-error"
            : undefined
        }
        onChange={(event) =>
          onChange("plateNumber", event.target.value)
        }
      />
    </Field>

    <Field
      id="new-motorcycle-vin"
      label="VIN"
      optional
      error={errors.vin}
    >
      <input
        aria-label="VIN"
        id="new-motorcycle-vin"
        type="text"
        value={values.vin}
        aria-invalid={errors.vin !== null}
        aria-describedby={
          errors.vin
            ? "new-motorcycle-vin-error"
            : undefined
        }
        onChange={(event) =>
          onChange("vin", event.target.value)
        }
      />
    </Field>

    <Field
      id="new-motorcycle-chassis"
      label="Chassis Number"
      optional
      error={errors.chassisNumber}
    >
      <input
        aria-label="Chassis Number"
        id="new-motorcycle-chassis"
        type="text"
        value={values.chassisNumber}
        aria-invalid={errors.chassisNumber !== null}
        aria-describedby={
          errors.chassisNumber
            ? "new-motorcycle-chassis-error"
            : undefined
        }
        onChange={(event) =>
          onChange("chassisNumber", event.target.value)
        }
      />
    </Field>
  </div>
</fieldset>

      <fieldset className="new-motorcycle-group">
        <legend>Notes</legend>
        <Field id="new-motorcycle-notes" label="Notes" optional>
          <textarea
            aria-label="Notes"
            id="new-motorcycle-notes"
            rows={3}
            value={values.notes}
            onChange={(event) => onChange("notes", event.target.value)}
          />
        </Field>
      </fieldset>

      {creationError ? (
        <p className="new-motorcycle-submit-error" role="alert">
          {creationError}
        </p>
      ) : null}

      <footer className="new-motorcycle-actions">
        <button
          className="new-motorcycle-button new-motorcycle-button--secondary"
          type="button"
          disabled={submitting}
          onClick={onCancel}
        >
          Cancel
        </button>
        <button
          className="new-motorcycle-button new-motorcycle-button--primary"
          type="submit"
          disabled={submitting || !requiredCatalogsAvailable}
        >
          {submitting ? "Creating…" : "Create Motorcycle"}
        </button>
      </footer>
    </form>
  );
}

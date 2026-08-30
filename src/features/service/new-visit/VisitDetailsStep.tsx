import type { FormEvent } from "react";

import type { CustomerMotorcycleLookup } from "../api/serviceVisitApi";

type VisitDetailsStepProps = {
  motorcycle: CustomerMotorcycleLookup;
  complaint: string;
  odometer: string;
  notes: string;
  complaintError: string | null;
  odometerError: string | null;
  creationError: string | null;
  submitting: boolean;
  onComplaintChange: (value: string) => void;
  onOdometerChange: (value: string) => void;
  onNotesChange: (value: string) => void;
  onSubmit: () => void;
};

export function VisitDetailsStep({
  motorcycle,
  complaint,
  odometer,
  notes,
  complaintError,
  odometerError,
  creationError,
  submitting,
  onComplaintChange,
  onOdometerChange,
  onNotesChange,
  onSubmit,
}: VisitDetailsStepProps) {
  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    onSubmit();
  }

  return (
    <section className="new-visit-step" aria-labelledby="new-visit-details-heading">
      <div className="new-visit-step__heading">
        <span className="new-visit-step__number">3</span>
        <div>
          <h3 id="new-visit-details-heading">Enter Visit Details</h3>
          <p>
            New Visit for {motorcycle.makeName} {motorcycle.model}.
          </p>
        </div>
      </div>

      <form className="new-visit-details" onSubmit={handleSubmit} noValidate>
        <label className="new-visit-field new-visit-field--wide">
          <span>Customer complaint <b aria-hidden="true">*</b></span>
          <textarea
            aria-label="Customer complaint"
            value={complaint}
            rows={3}
            aria-invalid={complaintError !== null}
            aria-describedby={complaintError ? "new-visit-complaint-error" : undefined}
            onChange={(event) => onComplaintChange(event.target.value)}
          />
          {complaintError ? (
            <small className="new-visit-field__error" id="new-visit-complaint-error">
              {complaintError}
            </small>
          ) : null}
        </label>

        <label className="new-visit-field">
          <span>Odometer (km) <em>Optional</em></span>
          <input
            aria-label="Odometer (km)"
            type="text"
            inputMode="numeric"
            value={odometer}
            placeholder="Whole kilometers"
            aria-invalid={odometerError !== null}
            aria-describedby={odometerError ? "new-visit-odometer-error" : undefined}
            onChange={(event) => onOdometerChange(event.target.value)}
          />
          {odometerError ? (
            <small className="new-visit-field__error" id="new-visit-odometer-error">
              {odometerError}
            </small>
          ) : null}
        </label>

        <label className="new-visit-field new-visit-field--wide">
          <span>Notes <em>Optional</em></span>
          <textarea
            aria-label="Notes"
            value={notes}
            rows={2}
            onChange={(event) => onNotesChange(event.target.value)}
          />
        </label>

        {creationError ? (
          <p className="new-visit-submit-error" role="alert">
            {creationError}
          </p>
        ) : null}

        <div className="new-visit-submit-row">
          <span>4</span>
          <button className="new-visit-button new-visit-button--primary" type="submit" disabled={submitting}>
            {submitting ? "Creating…" : "Create Service Visit"}
          </button>
        </div>
      </form>
    </section>
  );
}

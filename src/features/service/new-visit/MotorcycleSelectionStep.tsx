import type { CustomerMotorcycleLookup, CustomerSummary } from "../api/serviceVisitApi";

type MotorcycleSelectionStepProps = {
  customer: CustomerSummary;
  motorcycles: CustomerMotorcycleLookup[];
  selectedMotorcycleId: number | null;
  loading: boolean;
  error: string | null;
  onSelect: (motorcycle: CustomerMotorcycleLookup) => void;
};

function motorcycleDetails(motorcycle: CustomerMotorcycleLookup): string[] {
  const details: string[] = [];
  details.push(
    [motorcycle.year?.toString(), motorcycle.colorName].filter(Boolean).join(" • "),
  );
  if (motorcycle.plateCode !== null && motorcycle.plateNumber !== null) {
    details.push(`Plate ${motorcycle.plateCode}-${motorcycle.plateNumber}`);
  }
  if (motorcycle.vin !== null) {
    details.push(`VIN ${motorcycle.vin}`);
  } else if (motorcycle.chassisNumber !== null) {
    details.push(`Chassis ${motorcycle.chassisNumber}`);
  }
  return details;
}

export function MotorcycleSelectionStep({
  customer,
  motorcycles,
  selectedMotorcycleId,
  loading,
  error,
  onSelect,
}: MotorcycleSelectionStepProps) {
  return (
    <section className="new-visit-step" aria-labelledby="new-visit-motorcycle-heading">
      <div className="new-visit-step__heading">
        <span className="new-visit-step__number">2</span>
        <div>
          <h3 id="new-visit-motorcycle-heading">Choose Motorcycle</h3>
          <p>Motorcycles registered to {customer.name}.</p>
        </div>
      </div>

      <div className="new-visit-results" aria-live="polite">
        {loading ? <p className="new-visit-state">Loading motorcycles…</p> : null}
        {!loading && error ? <p className="new-visit-state new-visit-state--error">{error}</p> : null}
        {!loading && !error && motorcycles.length === 0 ? (
          <p className="new-visit-state">This customer has no motorcycles.</p>
        ) : null}
        {!loading && !error
          ? motorcycles.map((motorcycle) => {
              const hasActiveVisit = motorcycle.activeServiceVisitId !== null;
              return (
                <button
                  className="new-visit-choice new-visit-choice--motorcycle"
                  data-selected={selectedMotorcycleId === motorcycle.id}
                  disabled={hasActiveVisit}
                  key={motorcycle.id}
                  type="button"
                  onClick={() => onSelect(motorcycle)}
                >
                  <span className="new-visit-choice__body">
                    <span className="new-visit-choice__primary">
                      {motorcycle.makeName} {motorcycle.model}
                    </span>
                    {motorcycleDetails(motorcycle).map((detail) => (
                      <span className="new-visit-choice__secondary" key={detail}>
                        {detail}
                      </span>
                    ))}
                  </span>
                  {hasActiveVisit ? (
                    <span className="new-visit-active-warning">
                      Already has an active Visit: {motorcycle.activeServiceVisitStatus}
                    </span>
                  ) : (
                    <span className="new-visit-available">Available</span>
                  )}
                </button>
              );
            })
          : null}
      </div>
    </section>
  );
}

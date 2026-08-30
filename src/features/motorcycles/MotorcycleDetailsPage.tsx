import {
  ArrowLeft,
  Edit3,
  Gauge,
  Hash,
  Palette,
  User,
  Wrench,
} from "lucide-react";


import type {
  CustomerPreview,
  MotorcyclePreview,
  ServiceHistoryPreview,
} from "../customers/customerPreviewData";

type MotorcycleDetailsPageProps = {
  customer: CustomerPreview;
  motorcycle: MotorcyclePreview;
  onBack: () => void;
  onSelectVisit: (visit: ServiceHistoryPreview) => void;

};



function formatMoney(fils: number) {
  return `${(fils / 1000).toFixed(3)} JD`;
}

export function MotorcycleDetailsPage({
  customer,
  motorcycle,
  onBack,
  onSelectVisit,
}: MotorcycleDetailsPageProps) {
  const serviceHistory = customer.serviceHistory.filter(
    (visit) => visit.motorcycleId === motorcycle.id,
  );

  return (
    <section className="motorcycle-details-page">
      <button
        type="button"
        className="back-button"
        onClick={onBack}
      >
        <ArrowLeft size={17} />
        {customer.name}
      </button>

      <div className="motorcycle-profile-header">
        <div>
          <div className="motorcycle-title-row">
            <h1>
              {motorcycle.make} {motorcycle.model}
            </h1>
          </div>

          <div className="motorcycle-owner-line">
            <User size={15} />
            <span>{customer.name}</span>
            <span className="owner-separator">•</span>
            <span className="profile-phone">{customer.phone}</span>
          </div>
        </div>

        <div className="header-actions">
          <button className="secondary-button" type="button">
            <Edit3 size={16} />
            Edit Motorcycle
          </button>

          <button className="primary-button service-action" type="button">
            <Wrench size={17} />
            New Service Visit
          </button>
        </div>
      </div>

      <div className="motorcycle-info-grid">
        <div className="info-card">
          <div className="info-card-icon">
            <Hash size={18} />
          </div>

          <div>
            <span className="info-label">Plate</span>
            <strong>{motorcycle.plate ?? "Not recorded"}</strong>
          </div>
        </div>

        <div className="info-card">
          <div className="info-card-icon">
            <Gauge size={18} />
          </div>

          <div>
            <span className="info-label">Year</span>
            <strong>{motorcycle.year ?? "Not recorded"}</strong>
          </div>
        </div>

        <div className="info-card">
          <div className="info-card-icon">
            <Palette size={18} />
          </div>

          <div>
            <span className="info-label">Color</span>
            <strong>{motorcycle.color}</strong>
          </div>
        </div>
      </div>

      <section className="vehicle-identification-panel">
        <div className="section-header compact-section-header">
          <div>
            <h2>Identification</h2>
            <p>Recorded identifiers for this motorcycle.</p>
          </div>
        </div>

        <div className="vehicle-identity-list">
          <div className="identity-row">
            <span>VIN</span>
            <strong>{motorcycle.vin ?? "Not recorded"}</strong>
          </div>

          <div className="identity-row">
            <span>Chassis number</span>
            <strong>{motorcycle.chassis ?? "Not recorded"}</strong>
          </div>
        </div>
      </section>

      <section className="details-section">
        <div className="section-header">
          <div>
            <h2>Service History</h2>
            <p>Complete workshop history for this motorcycle.</p>
          </div>


        </div>

        <div className="content-panel">
          {serviceHistory.length > 0 ? (
            <div className="table-wrapper">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Visit</th>
                    <th>Date</th>
                    <th>Mileage</th>
                    <th>Complaint</th>
                    <th>Status</th>
                    <th className="money-column">Total</th>
                  </tr>
                </thead>

                <tbody>
                  {serviceHistory.map((visit) => (
                    <tr key={visit.id} onClick={() => onSelectVisit(visit)}>
                      <td>
                        <strong>#{visit.id}</strong>
                      </td>

                      <td className="muted-cell">
                        {visit.date}
                      </td>

                      <td className="odometer-cell">
                        {visit.odometerKm !== null
                        ? `${visit.odometerKm.toLocaleString()} km`
                        : "Not recorded"}
                    </td>


                      <td>{visit.complaint}</td>

                      <td>
                        <span
                          className={`status-badge status-${visit.status.toLowerCase()}`}
                        >
                          {visit.status.replace(/_/g, " ")}
                        </span>
                      </td>

                      <td className="money-column">
                        {formatMoney(visit.totalFils)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="section-empty-state large">
              <Wrench size={24} />

              <strong>No service history</strong>

              <span>
                This motorcycle has not visited the workshop yet.
              </span>
            </div>
          )}
        </div>
      </section>
    </section>
  );
}
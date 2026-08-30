import {
  ArrowLeft,
  Bike,
  Edit3,
  Plus,
  Wrench,
} from "lucide-react";
import type { CustomerPreview,
    MotorcyclePreview,

 } from "./customerPreviewData";

type CustomerDetailsPageProps = {
  customer: CustomerPreview;
  onBack: () => void;
  onSelectMotorcycle: (motorcycle: MotorcyclePreview) => void;

};

function formatMoney(fils: number) {
  return `${(fils / 1000).toFixed(3)} JD`;
}

export function CustomerDetailsPage({
  customer,
  onBack,
  onSelectMotorcycle
}: CustomerDetailsPageProps) {
  return (
    <section className="customer-details-page">
      <button
        type="button"
        className="back-button"
        onClick={onBack}
      >
        <ArrowLeft size={17} />
        Customers
      </button>

      <div className="customer-profile-header">
        <div className="customer-profile-identity">
          <div className="customer-profile-avatar">
            {customer.name.trim().charAt(0).toLocaleUpperCase()}
          </div>

          <div>
            <h1>{customer.name}</h1>
            <span className="profile-phone">{customer.phone}</span>
          </div>
        </div>

        <button className="secondary-button" type="button">
          <Edit3 size={16} />
          Edit Customer
        </button>
      </div>

      <section className="details-section">
        <div className="section-header">
          <div>
            <h2>Motorcycles</h2>
            <p>Motorcycles currently associated with this customer.</p>
          </div>

          <button className="secondary-button" type="button">
            <Plus size={17} />
            Add Motorcycle
          </button>
        </div>

        {customer.motorcycles.length > 0 ? (
          <div className="motorcycle-grid">
            {customer.motorcycles.map((motorcycle) => (
              <button
                type="button"
                className="motorcycle-card"
                key={motorcycle.id}
                onClick={() => onSelectMotorcycle(motorcycle)}>
                    <div className="motorcycle-icon">
                  <Bike size={20} />
                </div>

                <div className="motorcycle-card-content">
                  <div className="motorcycle-card-title">
                    {motorcycle.make} {motorcycle.model}
                  </div>

                  <div className="motorcycle-meta">
                    {motorcycle.year && <span>{motorcycle.year}</span>}
                    <span>{motorcycle.color}</span>

                    {motorcycle.plate && (
                      <span>Plate {motorcycle.plate}</span>
                    )}
                  </div>

                  {!motorcycle.plate && motorcycle.vin && (
                    <div className="motorcycle-identity">
                      VIN {motorcycle.vin}
                    </div>
                  )}
                </div>
              </button>
            ))}
          </div>
        ) : (
          <div className="section-empty-state">
            <Bike size={24} />
            <strong>No motorcycles yet</strong>
            <span>Add this customer's first motorcycle.</span>
          </div>
        )}
      </section>

      <section className="details-section">
        <div className="section-header">
          <div>
            <h2>Service History</h2>
            <p>Previous workshop visits for this customer.</p>
          </div>

          <button className="primary-button service-action" type="button">
            <Wrench size={17} />
            New Service Visit
          </button>
        </div>

        <div className="content-panel">
          {customer.serviceHistory.length > 0 ? (
            <div className="table-wrapper">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Motorcycle</th>
                    <th>Mileage</th>
                    <th>Complaint</th>
                    <th>Status</th>
                     <th className="money-column">Total</th>
                  </tr>
                </thead>

                <tbody>
                  {customer.serviceHistory.map((visit) => (
                    <tr key={visit.id}>
                    <td className="muted-cell">{visit.date}</td>

                    <td>
                        <strong>
                        {(() => {
                            const motorcycle = customer.motorcycles.find(
                            (item) => item.id === visit.motorcycleId,
                            );

                            return motorcycle
                            ? `${motorcycle.make} ${motorcycle.model}`
                            : "Unknown motorcycle";
                        })()}
                        </strong>
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
                Workshop visits for this customer will appear here.
              </span>
            </div>
          )}
        </div>
      </section>
    </section>
  );
}
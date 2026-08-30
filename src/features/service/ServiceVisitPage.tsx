import { useState } from "react";

import {
  ArrowLeft,
  Bike,
  Gauge,
  Package,
  User,
  Wrench,
} from "lucide-react";

import "./ServiceVisitPage.css";
import { JobDetailsCard } from "./components/JobDetailsCard";
import { LaborChargeField } from "./components/LaborChargeField";
import { AddPartDialog } from "./components/AddPartDialog";
import { canEditServiceVisit } from "./functions/canEditServiceVisit";
import { calculatePartsTotal } from "./functions/calculatePartsTotal";
import { formatServiceVisitStatus } from "./functions/formatServiceVisitStatus";
import { canAddServiceVisitPart } from "./functions/canAddServiceVisitPart";

import type {
  CustomerPreview,
  MotorcyclePreview,
  ServiceHistoryPreview,
} from "../customers/customerPreviewData";

import "./ServiceVisitPage.css";
type ServiceVisitPageProps = {
  customer: CustomerPreview;
  motorcycle: MotorcyclePreview;
  visit: ServiceHistoryPreview;
  onBack: () => void;
};

function formatMoney(fils: number) {
  return `${(fils / 1000).toFixed(3)} JD`;
}

export function ServiceVisitPage({
  customer,
  motorcycle,
  visit,
  onBack,
}: ServiceVisitPageProps) {
  const editable = canEditServiceVisit(visit);
  const canAddPart = canAddServiceVisitPart(visit);
  const [addPartOpen, setAddPartOpen] = useState(false);
  const partsTotalFils = calculatePartsTotal(visit.parts);

  return (
    <section className="service-visit-page">
      <button
        type="button"
        className="back-button"
        onClick={onBack}
      >
        <ArrowLeft size={17} />
        {motorcycle.make} {motorcycle.model}
      </button>

      <div className="service-visit-header">
        <div>
          <div className="visit-title-row">
            <h1>Service Visit #{visit.id}</h1>

            <span
              className={`status-badge status-${visit.status.toLowerCase()}`}
            >
              {formatServiceVisitStatus(visit.status)}
            </span>
          </div>

          <div className="visit-context">
            <span>
              <Bike size={14} />
              {motorcycle.make} {motorcycle.model}
            </span>

            <span>
              <User size={14} />
              {customer.name}
            </span>

            {visit.odometerKm !== null && (
              <span>
                <Gauge size={14} />
                {visit.odometerKm.toLocaleString()} km
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="service-workspace-grid">
        <div className="service-workspace-main">
            <JobDetailsCard visit={visit} />

          <section className="workspace-card">
            <div className="workspace-card-header">
              <div>
                <h2>Parts Used</h2>
                <p>Parts and materials recorded during this visit.</p>
              </div>
                    <button
                    type="button"
                    className="secondary-button"
                    disabled={!canAddPart}
                    onClick={() => setAddPartOpen(true)}
                    >
                <Package size={16} />
                Add Part
              </button>
            </div>

            {visit.parts.length > 0 ? (
              <div className="parts-table-wrapper">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Item</th>
                      <th>Quantity</th>
                      <th className="money-column">Unit Price</th>
                      <th className="money-column">Total</th>
                    </tr>
                  </thead>

                  <tbody>
                    {visit.parts.map((part) => (
                      <tr key={part.id}>
                        <td>
                          <strong>{part.name}</strong>
                        </td>

                        <td>{part.quantityLabel}</td>

                        <td className="money-column">
                          {formatMoney(part.unitPriceFils)}
                        </td>

                        <td className="money-column">
                          {formatMoney(part.lineTotalFils)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="section-empty-state large">
                <Package size={24} />
                <strong>No parts recorded</strong>
                <span>No inventory items were used on this visit.</span>
              </div>
            )}
          </section>
        </div>

        <aside className="service-summary-column">
          <section className="workspace-card sticky-summary">
            <div className="workspace-card-header">
              <h2>Visit Summary</h2>
            </div>

            <div className="summary-info-row">
              <span>Date</span>
              <strong>{visit.date}</strong>
            </div>

            <div className="summary-info-row">
              <span>Mileage</span>
              <strong>
                {visit.odometerKm !== null
                  ? `${visit.odometerKm.toLocaleString()} km`
                  : "Not recorded"}
              </strong>
            </div>

            <div className="summary-divider" />

            <div className="summary-info-row">
            <span>Labor</span>

            <LaborChargeField
                laborChargeFils={visit.laborChargeFils}
                editable={editable}
            />
            </div>

            <div className="summary-info-row">
              <span>Parts</span>
              <strong>{formatMoney(partsTotalFils)}</strong>
            </div>

            <div className="summary-total-row">
              <span>Total</span>
              <strong>{formatMoney(visit.totalFils)}</strong>
            </div>

            <button
              type="button"
              className="primary-button summary-button"
            >
              <Wrench size={17} />
              View Invoice
            </button>
          </section>
        </aside>
      </div>
      <AddPartDialog
  open={addPartOpen}
  onClose={() => setAddPartOpen(false)}
/>
    </section>
  );
}
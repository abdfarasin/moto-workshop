import { ArrowLeft, Bike, FileText, Gauge, Package, User } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import {
  issueInvoice,
  loadServiceVisitInvoice,
  type InvoiceDetails,
} from "../invoices/api/invoiceApi";
import type {
  ServiceVisitPart,
  ServiceVisitWorkspace,
} from "./api/serviceVisitApi";
import { formatServiceVisitStatus } from "./functions/formatServiceVisitStatus";
import "./ServiceVisitPage.css";

type ServiceVisitPageProps = {
  workspace: ServiceVisitWorkspace;
  onBack: () => void;
  onOpenInvoice?: (invoiceId: number) => void;
};

export function ServiceVisitPage({ workspace, onBack, onOpenInvoice }: ServiceVisitPageProps) {
  const { visit, owner, motorcycle, parts } = workspace;
  const [invoice, setInvoice] = useState<InvoiceDetails | null>(null);
  const [invoiceError, setInvoiceError] = useState<string | null>(null);
  const [issuing, setIssuing] = useState(false);
  const invoiceRequest = useRef(0);

  useEffect(() => {
    if (onOpenInvoice === undefined) return;
    const request = ++invoiceRequest.current;
    setInvoice(null);
    setInvoiceError(null);
    void loadServiceVisitInvoice(visit.id)
      .then((result) => { if (request === invoiceRequest.current) setInvoice(result); })
      .catch(() => { if (request === invoiceRequest.current) setInvoiceError("Invoice information could not be loaded."); });
    return () => { invoiceRequest.current += 1; };
  }, [onOpenInvoice, visit.id]);

  async function createInvoice() {
    if (issuing) return;
    setIssuing(true);
    setInvoiceError(null);
    try {
      const issued = await issueInvoice({ serviceVisitId: visit.id, issuedAt: Date.now() });
      setInvoice(issued);
      onOpenInvoice?.(issued.id);
    } catch {
      setInvoiceError("Invoice could not be issued. Confirm the Service Visit is completed.");
    } finally {
      setIssuing(false);
    }
  }
  const activePartsTotalFils = parts
    .filter((part) => part.status === "ACTIVE")
    .reduce((total, part) => total + part.lineTotalFils, 0);
  const serviceTotalFils = visit.laborChargeFils + activePartsTotalFils;

  return (
    <section className="service-visit-page">
      <button type="button" className="back-button" onClick={onBack}>
        <ArrowLeft size={17} />
        {owner.name}
      </button>

      <div className="service-visit-header">
        <div>
          <div className="visit-title-row">
            <h1>Service Visit #{visit.id}</h1>
            <span className={`status-badge status-${visit.status.toLowerCase()}`}>
              {formatServiceVisitStatus(visit.status)}
            </span>
          </div>

          <div className="visit-context">
            <span><Bike size={14} />{motorcycle.makeName} {motorcycle.model}</span>
            <span><User size={14} />{owner.name}</span>
            {visit.odometerKm !== null && (
              <span><Gauge size={14} />{visit.odometerKm.toLocaleString()} km</span>
            )}
          </div>
        </div>
        {invoice !== null && onOpenInvoice !== undefined && (
          invoice.status === "DRAFT" ? (
            (visit.status === "READY_FOR_PICKUP" || visit.status === "CLOSED") && (
              <button type="button" className="primary-button" disabled={issuing} onClick={() => void createInvoice()}>
                <FileText size={17} />{issuing ? "Creating Invoice..." : "Create Invoice"}
              </button>
            )
          ) : (
            <button type="button" className="secondary-button" onClick={() => onOpenInvoice(invoice.id)}>
              <FileText size={17} />View Invoice
            </button>
          )
        )}
      </div>

      {invoiceError !== null && <p className="service-visits-action-error" role="alert">{invoiceError}</p>}

      <div className="service-workspace-grid">
        <div className="service-workspace-main">
          <section className="workspace-card">
            <div className="workspace-card-header"><h2>Job Details</h2></div>
            <ReadOnlyField label="Customer Complaint" value={visit.customerComplaint} />
            <ReadOnlyField label="Diagnosis" value={visit.diagnosis} multiline />
            <ReadOnlyField label="Work Performed" value={visit.workPerformed} multiline />
            <ReadOnlyField label="Notes" value={visit.notes} multiline />
          </section>

          <section className="workspace-card">
            <div className="workspace-card-header">
              <div>
                <h2>Parts Used</h2>
                <p>Active and voided historical lines recorded for this visit.</p>
              </div>
            </div>

            {parts.length > 0 ? (
              <div className="parts-table-wrapper">
                <table className="data-table">
                  <thead>
                    <tr>
                      <th>Item</th>
                      <th>Quantity</th>
                      <th>Status</th>
                      <th className="money-column">Unit Price</th>
                      <th className="money-column">Total</th>
                    </tr>
                  </thead>
                  <tbody>
                    {parts.map((part) => (
                      <tr className={part.status === "VOIDED" ? "voided-part-row" : undefined} key={part.id}>
                        <td><strong>{part.itemName}</strong></td>
                        <td>{formatPartQuantity(part)}</td>
                        <td>
                          <span className={`part-status part-status--${part.status.toLowerCase()}`}>
                            {part.status}
                          </span>
                        </td>
                        <td className="money-column">{formatMoney(part.unitPriceFils)}</td>
                        <td className="money-column">{formatMoney(part.lineTotalFils)}</td>
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
            <div className="workspace-card-header"><h2>Visit Summary</h2></div>
            <SummaryRow label="Date" value={formatDate(visit.openedAt)} />
            <SummaryRow
              label="Mileage"
              value={visit.odometerKm !== null
                ? `${visit.odometerKm.toLocaleString()} km`
                : "Not recorded"}
            />
            <div className="summary-divider" />
            <SummaryRow label="Labor" value={formatMoney(visit.laborChargeFils)} />
            <SummaryRow label="Active parts" value={formatMoney(activePartsTotalFils)} />
            <div className="summary-total-row">
              <span>Service total</span>
              <strong>{formatMoney(serviceTotalFils)}</strong>
            </div>
          </section>
        </aside>
      </div>
    </section>
  );
}

function ReadOnlyField({
  label,
  value,
  multiline = false,
}: {
  label: string;
  value: string | null;
  multiline?: boolean;
}) {
  return (
    <div className="service-field">
      <label>{label}</label>
      <div className={`read-only-field${multiline ? " multiline" : ""}`}>
        {value ?? "Not recorded"}
      </div>
    </div>
  );
}

function SummaryRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="summary-info-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function formatPartQuantity(part: ServiceVisitPart): string {
  const decimals = Math.log10(part.quantityScale);
  return `${(part.quantity / part.quantityScale).toFixed(decimals)} ${part.unitName}`;
}

function formatMoney(fils: number): string {
  return `${(fils / 1000).toFixed(3)} JD`;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString();
}

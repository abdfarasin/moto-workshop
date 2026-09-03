import { ArrowLeft, Wrench } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { loadInvoiceDetails, type InvoiceDetails } from "./api/invoiceApi";
import "./Invoices.css";

type Props = { invoiceId: number; onBack: () => void; onOpenServiceVisit: (serviceVisitId: number) => void };
export function InvoiceDetailsPage({ invoiceId, onBack, onOpenServiceVisit }: Props) {
  const [invoice, setInvoice] = useState<InvoiceDetails | null>(null);
  const [error, setError] = useState(false);
  const request = useRef(0);
  useEffect(() => { const id = ++request.current; setInvoice(null); setError(false);
    void loadInvoiceDetails(invoiceId).then((result) => { if (id === request.current) setInvoice(result); })
      .catch(() => { if (id === request.current) setError(true); });
    return () => { request.current += 1; };
  }, [invoiceId]);
  if (error) return <section className="invoices-page"><button className="back-button" onClick={onBack}><ArrowLeft size={17} />Invoices</button><div className="empty-state" role="alert"><strong>Invoice could not be loaded</strong></div></section>;
  if (invoice === null) return <section className="invoices-page"><div className="empty-state"><strong>Loading Invoice...</strong></div></section>;
  return <section className="invoices-page">
    <button type="button" className="back-button" onClick={onBack}><ArrowLeft size={17} />Invoices</button>
    <div className="invoice-details-header"><div><h1>{invoice.invoiceNumber ?? `Draft Invoice #${invoice.id}`}</h1><p>Service Visit #{invoice.serviceVisitId}</p></div>
      <button type="button" className="secondary-button" onClick={() => onOpenServiceVisit(invoice.serviceVisitId)}><Wrench size={17} />Open Service Visit</button></div>
    {invoice.status === "DRAFT" && <p className="invoice-draft-note">This is a live draft preview. Customer, motorcycle, labor, and active parts are frozen only when the invoice is issued.</p>}
    <div className="invoice-details-grid"><div>
      <section className="content-panel invoice-identity"><div><span>Customer</span><strong>{invoice.customerName}</strong></div><div><span>Phone</span><strong>{invoice.customerPhone}</strong></div>
        <div><span>Motorcycle</span><strong>{invoice.motorcycleMakeName} {invoice.motorcycleModel}</strong></div><div><span>Identity</span><strong>{invoice.motorcyclePlateNumber ?? invoice.motorcycleVin ?? invoice.motorcycleChassisNumber ?? "Not recorded"}</strong></div></section>
      <section className="content-panel"><table className="data-table"><thead><tr><th>Part</th><th>Quantity</th><th className="money-column">Unit price</th><th className="money-column">Line total</th></tr></thead><tbody>
        {invoice.lines.map((line) => <tr key={line.serviceVisitPartId}><td><strong>{line.itemName}</strong></td><td>{formatQuantity(line.quantity, line.quantityScale)} {line.unitName}</td><td className="money-column">{formatMoney(line.unitPriceFils)}</td><td className="money-column">{formatMoney(line.lineTotalFils)}</td></tr>)}</tbody></table>
        {invoice.lines.length === 0 && <div className="empty-state"><strong>No active parts</strong></div>}</section></div>
      <aside><section className="content-panel invoice-summary"><div className="summary-info-row"><span>Labor</span><strong>{formatMoney(invoice.laborChargeFils)}</strong></div><div className="summary-info-row"><span>Parts</span><strong>{formatMoney(invoice.partsTotalFils)}</strong></div><div className="summary-divider" /><div className="summary-total-row"><span>Total</span><strong>{formatMoney(invoice.totalFils)}</strong></div></section></aside>
    </div>
  </section>;
}
const formatMoney = (fils: number) => `${(fils / 1000).toFixed(3)} JD`;
function formatQuantity(quantity: number, scale: number) { return (quantity / scale).toFixed(Math.log10(scale)); }

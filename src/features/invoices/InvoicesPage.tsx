import { ReceiptText, Search } from "lucide-react";
import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";

import { listInvoices, type InvoiceDirectoryEntry, type InvoiceDirectoryStatusFilter } from "./api/invoiceApi";
import "./Invoices.css";

type Props = {
  onSelectInvoice: (invoiceId: number) => void;
  initialStatusFilter?: InvoiceDirectoryStatusFilter;
};
const filters: Array<{ value: InvoiceDirectoryStatusFilter; label: string }> = [
  { value: "ALL", label: "All" }, { value: "DRAFT", label: "Draft" },
  { value: "ISSUED", label: "Issued" }, { value: "CANCELLED", label: "Cancelled" },
];

export function InvoicesPage({ onSelectInvoice, initialStatusFilter = "ALL" }: Props) {
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<InvoiceDirectoryStatusFilter>(initialStatusFilter);
  const [rows, setRows] = useState<InvoiceDirectoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const request = useRef(0);
  const load = useCallback(async () => {
    const id = ++request.current; setLoading(true); setError(false);
    try { const result = await listInvoices({ query: submittedQuery, statusFilter, limit: 50 });
      if (id === request.current) setRows(result);
    } catch { if (id === request.current) { setRows([]); setError(true); } }
    finally { if (id === request.current) setLoading(false); }
  }, [statusFilter, submittedQuery]);
  useEffect(() => { void load(); return () => { request.current += 1; }; }, [load]);
  function submit(event: FormEvent) { event.preventDefault(); setSubmittedQuery(query.trim()); }
  return <section className="invoices-page">
    <div className="page-header"><div><h1>Invoices</h1><p>Draft work and issued workshop invoices.</p></div></div>
    <div className="content-panel">
      <div className="invoices-toolbar">
        <form className="invoices-search" onSubmit={submit}><label><Search size={17} />
          <input aria-label="Search Invoices" type="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Invoice, customer, phone, plate, visit..." />
        </label><button className="secondary-button" type="submit">Search</button></form>
        <label className="invoices-filter"><span>Status</span><select aria-label="Invoice status" value={statusFilter} onChange={(event) => setStatusFilter(event.target.value as InvoiceDirectoryStatusFilter)}>
          {filters.map((filter) => <option key={filter.value} value={filter.value}>{filter.label}</option>)}</select></label>
      </div>
      {error ? <div className="empty-state" role="alert"><strong>Invoices could not be loaded</strong><span>Please try again.</span></div> :
        <div className="table-wrapper"><table className="data-table"><thead><tr><th>Invoice</th><th>Customer</th><th>Motorcycle</th><th>Issued</th><th>Status</th><th className="money-column">Total</th></tr></thead>
          <tbody>{rows.map((row) => <tr key={row.id} role="button" tabIndex={0} aria-label={`Open Invoice ${row.id}`} onClick={() => onSelectInvoice(row.id)} onKeyDown={(event) => { if (event.key === "Enter" || event.key === " ") { event.preventDefault(); onSelectInvoice(row.id); } }}>
            <td><strong className="invoice-number">{row.invoiceNumber ?? `Draft #${row.id}`}</strong><span className="invoice-secondary">Visit #{row.serviceVisitId}</span></td>
            <td><strong>{row.customerName}</strong><span className="invoice-secondary">{row.customerPhone}</span></td>
            <td><strong>{row.motorcycle}</strong><span className="invoice-secondary">{row.plateNumber ?? "No plate recorded"}</span></td>
            <td>{row.issuedAt === null ? "Not issued" : formatDate(row.issuedAt)}</td><td><span className={`status-badge status-${row.status.toLowerCase()}`}>{row.status}</span></td>
            <td className="money-column">{formatMoney(row.totalFils)}</td></tr>)}</tbody></table>
          {!loading && rows.length === 0 && <div className="empty-state"><ReceiptText size={24} /><strong>No invoices found</strong><span>Try a different search or status.</span></div>}
          {loading && <div className="empty-state"><strong>Loading Invoices...</strong></div>}
        </div>}
    </div>
  </section>;
}
const formatMoney = (fils: number) => `${(fils / 1000).toFixed(3)} JD`;
const formatDate = (timestamp: number) => new Date(timestamp).toLocaleString();

import { Bike, Boxes, FileText, Users, Wrench } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import type { InvoiceDirectoryStatusFilter } from "../invoices/api/invoiceApi";
import {
  loadServiceVisitWorkspace,
  type ServiceVisitDirectoryStatusFilter,
  type ServiceVisitWorkspace,
} from "../service/api/serviceVisitApi";
import { formatServiceVisitStatus } from "../service/functions/formatServiceVisitStatus";
import { loadDashboard, type DashboardData } from "./api/dashboardApi";
import "./DashboardPage.css";

type DashboardPageProps = {
  onOpenServiceVisit: (workspace: ServiceVisitWorkspace) => void;
  onOpenInvoice: (invoiceId: number) => void;
  onOpenInventoryItem: (inventoryItemId: number) => void;
  onShowService: (filter: ServiceVisitDirectoryStatusFilter) => void;
  onShowInventory: () => void;
  onShowInvoices: (filter: InvoiceDirectoryStatusFilter) => void;
};

export function DashboardPage({
  onOpenServiceVisit,
  onOpenInvoice,
  onOpenInventoryItem,
  onShowService,
  onShowInventory,
  onShowInvoices,
}: DashboardPageProps) {
  const [dashboard, setDashboard] = useState<DashboardData | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  const [workspaceError, setWorkspaceError] = useState(false);
  const [openingVisitId, setOpeningVisitId] = useState<number | null>(null);
  const request = useRef(0);
  const workspaceRequest = useRef(0);

  useEffect(() => {
    const id = ++request.current;
    setDashboard(null);
    setLoadFailed(false);
    void loadDashboard()
      .then((result) => { if (id === request.current) setDashboard(result); })
      .catch(() => { if (id === request.current) setLoadFailed(true); });
    return () => { request.current += 1; workspaceRequest.current += 1; };
  }, []);

  async function openServiceVisit(serviceVisitId: number) {
    if (openingVisitId !== null) return;
    const id = ++workspaceRequest.current;
    setOpeningVisitId(serviceVisitId);
    setWorkspaceError(false);
    try {
      const workspace = await loadServiceVisitWorkspace(serviceVisitId);
      if (id === workspaceRequest.current) onOpenServiceVisit(workspace);
    } catch {
      if (id === workspaceRequest.current) setWorkspaceError(true);
    } finally {
      if (id === workspaceRequest.current) setOpeningVisitId(null);
    }
  }

  if (loadFailed) {
    return <section className="dashboard-page"><div className="page-header"><div><h1>Dashboard</h1><p>Workshop operations at a glance.</p></div></div>
      <div className="empty-state" role="alert"><strong>Dashboard could not be loaded</strong><span>Please open it again to retry.</span></div></section>;
  }
  if (dashboard === null) {
    return <section className="dashboard-page"><div className="empty-state"><strong>Loading Dashboard...</strong></div></section>;
  }
  const { summary } = dashboard;
  return <section className="dashboard-page">
    <div className="page-header"><div><h1>Dashboard</h1><p>Current workshop activity and items needing attention.</p></div></div>
    <div className="dashboard-cards">
      <MetricButton label="Active Jobs" value={summary.activeServiceVisits} icon={<Wrench size={20} />} ariaLabel="Show active Service Visits" onClick={() => onShowService("ACTIVE")} />
      <MetricButton label="Ready for Pickup" value={summary.readyForPickupVisits} icon={<Bike size={20} />} ariaLabel="Show ready Service Visits" onClick={() => onShowService("READY_FOR_PICKUP")} />
      <MetricButton label="Low Stock" value={summary.lowStockItemCount} detail={`${summary.negativeStockItemCount} negative`} icon={<Boxes size={20} />} ariaLabel="Show low-stock Inventory" onClick={onShowInventory} />
      <MetricButton label="Issued Today" value={summary.issuedInvoiceCountToday} detail={formatMoney(summary.issuedInvoiceValueTodayFils)} icon={<FileText size={20} />} ariaLabel="Show issued Invoices" onClick={() => onShowInvoices("ISSUED")} />
    </div>
    <div className="dashboard-record-counts"><span><Users size={16} /><strong>{summary.customerCount}</strong> Customers</span><span><Bike size={16} /><strong>{summary.motorcycleCount}</strong> Motorcycles</span></div>
    {workspaceError && <p className="dashboard-action-error" role="alert">This Service Visit could not be opened.</p>}
    <div className="dashboard-grid">
      <section className="content-panel dashboard-panel dashboard-panel--wide"><div className="dashboard-panel-header"><div><h2>Recent Service Visits</h2><p>Newest workshop visits.</p></div></div>
        {dashboard.recentServiceVisits.length === 0 ? <DashboardEmpty text="No recent Service Visits" /> : <div className="table-wrapper"><table className="data-table"><thead><tr><th>Visit</th><th>Customer</th><th>Motorcycle</th><th>Opened</th><th>Status</th></tr></thead><tbody>
          {dashboard.recentServiceVisits.map((visit) => <tr key={visit.id} role="button" tabIndex={0} aria-label={`Open recent Service Visit ${visit.id}`} aria-disabled={openingVisitId !== null}
            onClick={() => void openServiceVisit(visit.id)} onKeyDown={(event) => activateRow(event, () => void openServiceVisit(visit.id))}>
            <td><strong className="dashboard-link">#{visit.id}</strong><span className="dashboard-secondary">{visit.complaint}</span></td><td>{visit.customerName}</td><td><strong>{visit.motorcycle}</strong><span className="dashboard-secondary">{visit.plateNumber ?? "No plate"}</span></td><td>{formatDateTime(visit.openedAt)}</td><td><span className={`status-badge status-${visit.status.toLowerCase()}`}>{formatServiceVisitStatus(visit.status)}</span></td></tr>)}</tbody></table></div>}
      </section>
      <section className="content-panel dashboard-panel"><div className="dashboard-panel-header"><div><h2>Inventory Alerts</h2><p>Negative and below-minimum stock.</p></div></div>
        {dashboard.inventoryAlerts.length === 0 ? <DashboardEmpty text="No Inventory alerts" /> : <div className="dashboard-compact-list">{dashboard.inventoryAlerts.map((item) => <button type="button" key={item.id} aria-label={`Open Inventory Item ${item.id}`} onClick={() => onOpenInventoryItem(item.id)}>
          <span><strong>{item.itemName}</strong><small>{item.sku ?? `Item #${item.id}`}</small></span><span className={item.negativeStock ? "dashboard-negative" : "dashboard-low"}>{formatQuantity(item.currentQuantity, item.quantityScale)} {item.unitName}<small>Minimum {formatQuantity(item.minimumStockQuantity, item.quantityScale)}</small></span></button>)}</div>}
      </section>
      <section className="content-panel dashboard-panel dashboard-panel--full"><div className="dashboard-panel-header"><div><h2>Recent Issued Invoices</h2><p>Latest immutable invoice snapshots.</p></div></div>
        {dashboard.recentInvoices.length === 0 ? <DashboardEmpty text="No issued invoices yet" /> : <div className="table-wrapper"><table className="data-table"><thead><tr><th>Invoice</th><th>Customer</th><th>Motorcycle</th><th>Issued</th><th className="money-column">Total</th></tr></thead><tbody>
          {dashboard.recentInvoices.map((invoice) => <tr key={invoice.id} role="button" tabIndex={0} aria-label={`Open recent Invoice ${invoice.invoiceNumber}`} onClick={() => onOpenInvoice(invoice.id)} onKeyDown={(event) => activateRow(event, () => onOpenInvoice(invoice.id))}>
            <td><strong className="dashboard-link">{invoice.invoiceNumber}</strong></td><td>{invoice.customerName}</td><td>{invoice.motorcycle}</td><td>{formatDateTime(invoice.issuedAt)}</td><td className="money-column">{formatMoney(invoice.totalFils)}</td></tr>)}</tbody></table></div>}
      </section>
    </div>
  </section>;
}

function MetricButton({ label, value, detail, icon, ariaLabel, onClick }: { label: string; value: number; detail?: string; icon: React.ReactNode; ariaLabel: string; onClick: () => void }) {
  return <button type="button" className="dashboard-card" aria-label={ariaLabel} onClick={onClick}><span className="dashboard-card-icon">{icon}</span><span><small>{label}</small><strong>{value.toLocaleString()}</strong>{detail && <em>{detail}</em>}</span></button>;
}
function DashboardEmpty({ text }: { text: string }) { return <div className="dashboard-empty"><span>{text}</span></div>; }
function activateRow(event: React.KeyboardEvent, action: () => void) { if (event.key === "Enter" || event.key === " ") { event.preventDefault(); action(); } }
function formatMoney(fils: number) { return `${(fils / 1_000).toFixed(3)} JD`; }
function formatDateTime(timestamp: number) { return new Date(timestamp).toLocaleString(); }
function formatQuantity(quantity: number, scale: number) { return (quantity / scale).toFixed(Math.log10(scale)); }

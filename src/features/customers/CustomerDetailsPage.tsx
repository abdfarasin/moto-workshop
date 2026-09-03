import { ArrowLeft, Bike, Plus, Wrench } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";

import { NewMotorcycleDialog } from "../motorcycles/new-motorcycle/NewMotorcycleDialog";
import {
  loadServiceVisitWorkspace,
  type CustomerSummary,
  type ServiceVisitWorkspace,
} from "../service/api/serviceVisitApi";
import { NewServiceVisitDialog } from "../service/new-visit/NewServiceVisitDialog";
import {
  loadCustomerDetails,
  type CustomerDetails,
} from "./api/customerDirectoryApi";

type CustomerDetailsPageProps = {
  customerId: number;
  onBack: () => void;
  onOpenServiceVisit: (workspace: ServiceVisitWorkspace) => void;
};

export function CustomerDetailsPage({
  customerId,
  onBack,
  onOpenServiceVisit,
}: CustomerDetailsPageProps) {
  const [customer, setCustomer] = useState<CustomerDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [reloadNumber, setReloadNumber] = useState(0);
  const [newMotorcycleOpen, setNewMotorcycleOpen] = useState(false);
  const [newServiceVisitOpen, setNewServiceVisitOpen] = useState(false);
  const [openingVisitId, setOpeningVisitId] = useState<number | null>(null);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const workspaceRequest = useRef(0);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      setError(null);
      setCustomer(null);

      try {
        const result = await loadCustomerDetails(customerId);
        if (!cancelled) setCustomer(result);
      } catch {
        if (!cancelled) {
          setError("Could not load this customer. Please try again.");
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [customerId, reloadNumber]);

  useEffect(() => () => {
    workspaceRequest.current += 1;
  }, []);

  function refreshCustomer() {
    setReloadNumber((current) => current + 1);
  }

  async function openServiceVisit(serviceVisitId: number) {
    if (openingVisitId !== null) return;

    const request = ++workspaceRequest.current;
    setOpeningVisitId(serviceVisitId);
    setHistoryError(null);

    try {
      const workspace = await loadServiceVisitWorkspace(serviceVisitId);
      if (request === workspaceRequest.current) onOpenServiceVisit(workspace);
    } catch {
      if (request === workspaceRequest.current) {
        setHistoryError("Could not open this Service Visit. Please try again.");
      }
    } finally {
      if (request === workspaceRequest.current) setOpeningVisitId(null);
    }
  }

  if (loading) {
    return (
      <DetailsState onBack={onBack}>
        <div className="section-empty-state large">
          <strong>Loading customer...</strong>
        </div>
      </DetailsState>
    );
  }

  if (error || customer === null) {
    return (
      <DetailsState onBack={onBack}>
        <div className="section-empty-state large" role="alert">
          <strong>Customer could not be loaded</strong>
          <span>{error ?? "The customer is no longer available."}</span>
        </div>
      </DetailsState>
    );
  }

  const customerSummary: CustomerSummary = {
    id: customer.id,
    name: customer.name,
    phone: customer.phone,
  };

  return (
    <section className="customer-details-page">
      <BackToCustomers onBack={onBack} />

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

        <button
          className="primary-button service-action"
          type="button"
          onClick={() => setNewServiceVisitOpen(true)}
        >
          <Wrench size={17} />
          New Service Visit
        </button>
      </div>

      <section className="details-section">
        <div className="section-header">
          <div>
            <h2>Motorcycles</h2>
            <p>Motorcycles currently associated with this customer.</p>
          </div>
          <button
            className="secondary-button"
            type="button"
            onClick={() => setNewMotorcycleOpen(true)}
          >
            <Plus size={17} />
            Add Motorcycle
          </button>
        </div>

        {customer.motorcycles.length > 0 ? (
          <div className="motorcycle-grid">
            {customer.motorcycles.map((motorcycle) => (
              <div className="motorcycle-card" key={motorcycle.id}>
                <div className="motorcycle-icon"><Bike size={20} /></div>
                <div className="motorcycle-card-content">
                  <div className="motorcycle-card-title">
                    {motorcycle.makeName} {motorcycle.model}
                  </div>
                  <div className="motorcycle-meta">
                    {motorcycle.year !== null && <span>{motorcycle.year}</span>}
                    <span>{motorcycle.colorName}</span>
                    {motorcycle.plateNumber !== null && (
                      <span>Plate {motorcycle.plateNumber}</span>
                    )}
                  </div>
                  {motorcycle.plateNumber === null && motorcycle.vin !== null && (
                    <div className="motorcycle-identity">VIN {motorcycle.vin}</div>
                  )}
                  {motorcycle.plateNumber === null &&
                    motorcycle.vin === null &&
                    motorcycle.chassisNumber !== null && (
                      <div className="motorcycle-identity">
                        Chassis {motorcycle.chassisNumber}
                      </div>
                    )}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="section-empty-state">
            <Bike size={24} />
            <strong>No motorcycles yet</strong>
            <span>This customer has no motorcycles.</span>
          </div>
        )}
      </section>

      <section className="details-section">
        <div className="section-header">
          <div>
            <h2>Service History</h2>
            <p>Workshop visits for this customer.</p>
          </div>
        </div>

        {historyError !== null && (
          <p className="customer-details-action-error" role="alert">
            {historyError}
          </p>
        )}

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
                  {customer.serviceHistory.map((visit) => {
                    const motorcycle = customer.motorcycles.find(
                      (item) => item.id === visit.motorcycleId,
                    );
                    const opening = openingVisitId === visit.id;
                    return (
                      <tr
                        key={visit.id}
                        role="button"
                        tabIndex={0}
                        aria-label={`Open Service Visit ${visit.id}`}
                        aria-disabled={openingVisitId !== null}
                        onClick={() => void openServiceVisit(visit.id)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            event.preventDefault();
                            void openServiceVisit(visit.id);
                          }
                        }}
                      >
                        <td className="muted-cell">{formatDate(visit.openedAt)}</td>
                        <td>
                          <strong>
                            {motorcycle
                              ? `${motorcycle.makeName} ${motorcycle.model}`
                              : "Unknown motorcycle"}
                          </strong>
                        </td>
                        <td className="odometer-cell">
                          {visit.odometerKm !== null
                            ? `${visit.odometerKm.toLocaleString()} km`
                            : "Not recorded"}
                        </td>
                        <td>{opening ? "Opening..." : visit.customerComplaint}</td>
                        <td>
                          <span className={`status-badge status-${visit.status.toLowerCase()}`}>
                            {visit.status.replace(/_/g, " ")}
                          </span>
                        </td>
                        <td className="money-column">{formatMoney(visit.totalFils)}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="section-empty-state large">
              <Wrench size={24} />
              <strong>No service history</strong>
              <span>Workshop visits for this customer will appear here.</span>
            </div>
          )}
        </div>
      </section>

      <NewMotorcycleDialog
        open={newMotorcycleOpen}
        customer={customerSummary}
        onClose={() => setNewMotorcycleOpen(false)}
        onCreated={() => {
          setNewMotorcycleOpen(false);
          refreshCustomer();
        }}
      />
      <NewServiceVisitDialog
        open={newServiceVisitOpen}
        initialCustomer={customerSummary}
        onClose={() => setNewServiceVisitOpen(false)}
        onCreated={(workspace) => {
          setNewServiceVisitOpen(false);
          refreshCustomer();
          onOpenServiceVisit(workspace);
        }}
      />
    </section>
  );
}

function DetailsState({
  onBack,
  children,
}: {
  onBack: () => void;
  children: ReactNode;
}) {
  return (
    <section className="customer-details-page">
      <BackToCustomers onBack={onBack} />
      {children}
    </section>
  );
}

function BackToCustomers({ onBack }: { onBack: () => void }) {
  return (
    <button type="button" className="back-button" onClick={onBack}>
      <ArrowLeft size={17} />
      Customers
    </button>
  );
}

function formatMoney(fils: number): string {
  return `${(fils / 1000).toFixed(3)} JD`;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString();
}

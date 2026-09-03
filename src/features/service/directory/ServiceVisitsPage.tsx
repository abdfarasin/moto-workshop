import { Search, Wrench } from "lucide-react";
import { useCallback, useEffect, useRef, useState, type FormEvent } from "react";

import {
  listServiceVisits,
  loadServiceVisitWorkspace,
} from "../api/serviceVisitApi";
import type {
  ServiceVisitDirectoryEntry,
  ServiceVisitDirectoryStatusFilter,
  ServiceVisitWorkspace,
} from "../api/serviceVisitApi";
import { formatServiceVisitStatus } from "../functions/formatServiceVisitStatus";
import "./ServiceVisitsPage.css";

const DIRECTORY_LIMIT = 50;

type ServiceVisitsPageProps = {
  onOpenServiceVisit: (workspace: ServiceVisitWorkspace) => void;
  initialStatusFilter?: ServiceVisitDirectoryStatusFilter;
};

const filters: Array<{
  value: ServiceVisitDirectoryStatusFilter;
  label: string;
}> = [
  { value: "ACTIVE", label: "Active work" },
  { value: "ALL", label: "All" },
  { value: "OPEN", label: "Open" },
  { value: "READY_FOR_PICKUP", label: "Ready for Pickup" },
  { value: "CLOSED", label: "Closed" },
  { value: "CANCELLED", label: "Cancelled" },
];

export function ServiceVisitsPage({
  onOpenServiceVisit,
  initialStatusFilter = "ACTIVE",
}: ServiceVisitsPageProps) {
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [statusFilter, setStatusFilter] =
    useState<ServiceVisitDirectoryStatusFilter>(initialStatusFilter);
  const [visits, setVisits] = useState<ServiceVisitDirectoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [listError, setListError] = useState<string | null>(null);
  const [openingVisitId, setOpeningVisitId] = useState<number | null>(null);
  const [workspaceError, setWorkspaceError] = useState<string | null>(null);
  const listRequest = useRef(0);
  const workspaceRequest = useRef(0);

  const loadDirectory = useCallback(async () => {
    const request = ++listRequest.current;
    setLoading(true);
    setListError(null);

    try {
      const result = await listServiceVisits({
        query: submittedQuery,
        statusFilter,
        limit: DIRECTORY_LIMIT,
      });
      if (request === listRequest.current) setVisits(result);
    } catch {
      if (request === listRequest.current) {
        setVisits([]);
        setListError("Could not load Service Visits. Please try again.");
      }
    } finally {
      if (request === listRequest.current) setLoading(false);
    }
  }, [statusFilter, submittedQuery]);

  useEffect(() => {
    void loadDirectory();
    return () => {
      listRequest.current += 1;
    };
  }, [loadDirectory]);

  useEffect(() => () => {
    workspaceRequest.current += 1;
  }, []);

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmittedQuery(query.trim());
  }

  async function openWorkspace(serviceVisitId: number) {
    if (openingVisitId !== null) return;
    const request = ++workspaceRequest.current;
    setOpeningVisitId(serviceVisitId);
    setWorkspaceError(null);

    try {
      const workspace = await loadServiceVisitWorkspace(serviceVisitId);
      if (request === workspaceRequest.current) onOpenServiceVisit(workspace);
    } catch {
      if (request === workspaceRequest.current) {
        setWorkspaceError("Could not open this Service Visit. Please try again.");
      }
    } finally {
      if (request === workspaceRequest.current) setOpeningVisitId(null);
    }
  }

  return (
    <section className="service-visits-directory">
      <div className="page-header">
        <div>
          <h1>Service Visits</h1>
          <p>Active workshop jobs and historical visits.</p>
        </div>
      </div>

      <div className="content-panel">
        <div className="service-visits-toolbar">
          <form className="service-visits-search" onSubmit={submitSearch}>
            <label>
              <span className="sr-only">Search Service Visits</span>
              <Search size={17} />
              <input
                type="search"
                aria-label="Search Service Visits"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Customer, phone, plate, motorcycle..."
              />
            </label>
            <button type="submit" className="secondary-button">Search</button>
          </form>

          <label className="service-visits-filter">
            <span>Status</span>
            <select
              aria-label="Status"
              value={statusFilter}
              onChange={(event) => setStatusFilter(
                event.target.value as ServiceVisitDirectoryStatusFilter,
              )}
            >
              {filters.map((filter) => (
                <option key={filter.value} value={filter.value}>{filter.label}</option>
              ))}
            </select>
          </label>
        </div>

        {listError !== null && (
          <div className="empty-state" role="alert">
            <strong>Service Visits could not be loaded</strong>
            <span>{listError}</span>
          </div>
        )}
        {workspaceError !== null && (
          <p className="service-visits-action-error" role="alert">{workspaceError}</p>
        )}

        {!listError && (
          <div className="table-wrapper">
            <table className="data-table service-visits-table">
              <thead>
                <tr>
                  <th>Visit</th>
                  <th>Customer</th>
                  <th>Motorcycle</th>
                  <th>Opened</th>
                  <th>Complaint</th>
                  <th>Status</th>
                  <th className="money-column">Total</th>
                </tr>
              </thead>
              <tbody>
                {visits.map((visit) => (
                  <tr
                    key={visit.id}
                    role="button"
                    tabIndex={0}
                    aria-label={`Open Service Visit ${visit.id}`}
                    aria-disabled={openingVisitId !== null}
                    onClick={() => void openWorkspace(visit.id)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        void openWorkspace(visit.id);
                      }
                    }}
                  >
                    <td>
                      <strong className="service-visit-id">#{visit.id}</strong>
                    </td>
                    <td>
                      <strong>{visit.customerName}</strong>
                      <span className="service-visit-secondary">{visit.customerPhone}</span>
                    </td>
                    <td>
                      <strong>{visit.makeName} {visit.model}</strong>
                      <span className="service-visit-secondary">
                        {visit.plateNumber ?? "No plate recorded"}
                      </span>
                    </td>
                    <td className="muted-cell">{formatDateTime(visit.openedAt)}</td>
                    <td className="service-visit-complaint">{visit.customerComplaint}</td>
                    <td>
                      <span className={`status-badge status-${visit.status.toLowerCase()}`}>
                        {formatServiceVisitStatus(visit.status)}
                      </span>
                    </td>
                    <td className="money-column">{formatMoney(visit.totalFils)}</td>
                  </tr>
                ))}
              </tbody>
            </table>

            {!loading && visits.length === 0 && (
              <div className="empty-state">
                <Wrench size={24} />
                <strong>No Service Visits found</strong>
                <span>Try a different search or status filter.</span>
              </div>
            )}
            {loading && (
              <div className="empty-state"><strong>Loading Service Visits...</strong></div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}

function formatMoney(fils: number): string {
  return `${(fils / 1000).toFixed(3)} JD`;
}

function formatDateTime(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

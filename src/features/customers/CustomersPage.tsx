import { Search } from "lucide-react";
import {
  useEffect,
  useRef,
  useState,
} from "react";

import {
  searchCustomerDirectory,
  type CustomerDirectoryEntry,
} from "./api/customerDirectoryApi";

const SEARCH_DELAY_MS = 200;
const CUSTOMER_LIMIT = 25;

  type CustomersPageProps = {
    onSelectCustomer: (customerId: number) => void;
  };

  export function CustomersPage({
    onSelectCustomer,
  }: CustomersPageProps) {
  const [query, setQuery] = useState("");
  const [customers, setCustomers] = useState<
    CustomerDirectoryEntry[]
  >([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const requestNumber = useRef(0);

  useEffect(() => {
    const currentRequest = ++requestNumber.current;

    const delay =
      query.trim().length === 0
        ? 0
        : SEARCH_DELAY_MS;

    const timeout = window.setTimeout(() => {
      async function loadCustomers() {
        setLoading(true);
        setError(null);

        try {
          const result = await searchCustomerDirectory({
            query: query.trim(),
            limit: CUSTOMER_LIMIT,
          });

          if (currentRequest !== requestNumber.current) {
            return;
          }

          setCustomers(result);
        } catch {
          if (currentRequest !== requestNumber.current) {
            return;
          }

          setCustomers([]);
          setError(
            "Could not load customers. Please try again.",
          );
        } finally {
          if (currentRequest === requestNumber.current) {
            setLoading(false);
          }
        }
      }

      void loadCustomers();
    }, delay);

    return () => {
      window.clearTimeout(timeout);
    };
  }, [query]);

  return (
    <section className="customers-page">
      <div className="page-header">
        <div>
          <h1>Customers</h1>
          <p>
            Manage workshop customers and their motorcycles.
          </p>
        </div>
      </div>

      <div className="content-panel">
        <div className="table-toolbar">
          <div className="table-search">
            <Search size={17} />

            <input
              type="search"
              value={query}
              onChange={(event) =>
                setQuery(event.target.value)
              }
              placeholder="Search customers..."
              aria-label="Search customers"
            />
          </div>

          <span className="result-count">
            {loading
              ? "Loading..."
              : `Showing ${customers.length} ${
                  customers.length === 1
                    ? "customer"
                    : "customers"
                }`}
          </span>
        </div>

        <div className="table-wrapper">
          {error ? (
            <div className="empty-state" role="alert">
              <strong>Customers could not be loaded</strong>
              <span>{error}</span>
            </div>
          ) : (
            <>
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Customer</th>
                    <th>Phone</th>
                    <th>Motorcycles</th>
                    <th>Last visit</th>
                  </tr>
                </thead>

                <tbody>
                  {customers.map((customer) => (
                    <tr
                      key={customer.id}
                      onClick={() => onSelectCustomer(customer.id)}
                    >
                      <td>
                        <div className="customer-cell">
                          <div className="customer-avatar">
                            {customer.name
                              .trim()
                              .charAt(0)
                              .toLocaleUpperCase()}
                          </div>

                          <strong>{customer.name}</strong>
                        </div>
                      </td>

                      <td className="phone-cell">
                        {customer.phone}
                      </td>

                      <td>
                        {customer.motorcycleCount}
                      </td>

                      <td className="muted-cell">
                        {formatLastVisit(
                          customer.lastVisitAt,
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>

              {!loading &&
                customers.length === 0 && (
                  <div className="empty-state">
                    <strong>
                      {query.trim()
                        ? "No customers found"
                        : "No customers yet"}
                    </strong>

                    <span>
                      {query.trim()
                        ? "Try a different name or phone number."
                        : "Customers will appear here after they are created."}
                    </span>
                  </div>
                )}
            </>
          )}
        </div>
      </div>
    </section>
  );
}

function formatLastVisit(
  timestamp: number | null,
): string {
  if (timestamp === null) {
    return "No visits yet";
  }

  return new Date(timestamp).toLocaleDateString();
}
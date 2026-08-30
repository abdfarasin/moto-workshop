import { Plus, Search } from "lucide-react";
import { useMemo, useState } from "react";
import {
  previewCustomers,
  type CustomerPreview,
} from "./customerPreviewData";

type CustomersPageProps = {
  onSelectCustomer: (customer: CustomerPreview) => void;
};

export function CustomersPage({
  onSelectCustomer,
}: CustomersPageProps) {
  const [query, setQuery] = useState("");

  const visibleCustomers = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();

    if (!normalizedQuery) {
      return previewCustomers;
    }

    return previewCustomers.filter((customer) => {
      return (
        customer.name
          .toLocaleLowerCase()
          .includes(normalizedQuery) ||
        customer.phone.includes(normalizedQuery)
      );
    });
  }, [query]);

  return (
    <section className="customers-page">
      <div className="page-header">
        <div>
          <h1>Customers</h1>
          <p>Manage workshop customers and their motorcycles.</p>
        </div>

        <button className="secondary-button" type="button">
          <Plus size={18} />
          New Customer
        </button>
      </div>

      <div className="content-panel">
        <div className="table-toolbar">
          <div className="table-search">
            <Search size={17} />

            <input
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search customers..."
              aria-label="Search customers"
            />
          </div>

          <span className="result-count">
            {visibleCustomers.length} customers
          </span>
        </div>

        <div className="table-wrapper">
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
              {visibleCustomers.map((customer) => (
                <tr
                  key={customer.id}
                  onClick={() => onSelectCustomer(customer)}
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

                  <td className="phone-cell">{customer.phone}</td>

                  <td>{customer.motorcycles.length}</td>

                  <td className="muted-cell">
                    {customer.serviceHistory[0]?.date ?? "No visits yet"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {visibleCustomers.length === 0 && (
            <div className="empty-state">
              <strong>No customers found</strong>
              <span>Try a different name or phone number.</span>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
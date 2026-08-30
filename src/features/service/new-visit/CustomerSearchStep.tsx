import type { FormEvent } from "react";

import type { CustomerSummary } from "../api/serviceVisitApi";

type CustomerSearchStepProps = {
  query: string;
  customers: CustomerSummary[];
  selectedCustomerId: number | null;
  loading: boolean;
  error: string | null;
  onQueryChange: (query: string) => void;
  onSearch: () => void;
  onSelect: (customer: CustomerSummary) => void;
};

export function CustomerSearchStep({
  query,
  customers,
  selectedCustomerId,
  loading,
  error,
  onQueryChange,
  onSearch,
  onSelect,
}: CustomerSearchStepProps) {
  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    onSearch();
  }

  return (
    <section className="new-visit-step" aria-labelledby="new-visit-customer-heading">
      <div className="new-visit-step__heading">
        <span className="new-visit-step__number">1</span>
        <div>
          <h3 id="new-visit-customer-heading">Find Customer</h3>
          <p>Search by customer name or phone number.</p>
        </div>
      </div>

      <form className="new-visit-search" onSubmit={handleSubmit}>
        <label className="new-visit-field new-visit-search__field">
          <span>Search customers</span>
          <input
            autoFocus
            type="search"
            value={query}
            placeholder="Name or phone number"
            onChange={(event) => onQueryChange(event.target.value)}
          />
        </label>
        <button className="new-visit-button new-visit-button--secondary" type="submit">
          Search
        </button>
      </form>

      <div className="new-visit-results" aria-live="polite">
        {loading ? <p className="new-visit-state">Loading customers…</p> : null}
        {!loading && error ? <p className="new-visit-state new-visit-state--error">{error}</p> : null}
        {!loading && !error && customers.length === 0 ? (
          <p className="new-visit-state">No customers found.</p>
        ) : null}
        {!loading && !error
          ? customers.map((customer) => (
              <button
                className="new-visit-choice"
                data-selected={selectedCustomerId === customer.id}
                key={customer.id}
                type="button"
                onClick={() => onSelect(customer)}
              >
                <span className="new-visit-choice__primary">{customer.name}</span>
                <span className="new-visit-choice__secondary">{customer.phone}</span>
              </button>
            ))
          : null}
      </div>
    </section>
  );
}

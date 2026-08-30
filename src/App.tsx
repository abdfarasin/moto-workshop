import { useState } from "react";

import { AppShell, type AppSection } from "./components/AppShell";
import { CustomerDetailsPage } from "./features/customers/CustomerDetailsPage";
import { CustomersPage } from "./features/customers/CustomersPage";
import { ServiceVisitPage } from "./features/service/ServiceVisitPage";

import type {
  CustomerPreview,
  MotorcyclePreview,
  ServiceHistoryPreview,

} from "./features/customers/customerPreviewData";

import { MotorcycleDetailsPage } from "./features/motorcycles/MotorcycleDetailsPage";

import "./App.css";

function App() {
  const [activeSection, setActiveSection] =
    useState<AppSection>("customers");

  const [selectedCustomer, setSelectedCustomer] =
    useState<CustomerPreview | null>(null);

  const [selectedMotorcycle, setSelectedMotorcycle] =
    useState<MotorcyclePreview | null>(null);

  const [selectedVisit, setSelectedVisit] =
  useState<ServiceHistoryPreview | null>(null);

  function handleSectionChange(section: AppSection) {
    setActiveSection(section);
    setSelectedCustomer(null);
    setSelectedMotorcycle(null);
  }

  function handleSelectCustomer(customer: CustomerPreview) {
    setSelectedCustomer(customer);
    setSelectedMotorcycle(null);
  }

  function handleBackToCustomer() {
    setSelectedMotorcycle(null);
  }

  function handleBackToCustomers() {
    setSelectedCustomer(null);
    setSelectedMotorcycle(null);
  }

  return (
    <AppShell
      activeSection={activeSection}
      onSectionChange={handleSectionChange}
    >
      {activeSection === "customers" ? (
        selectedCustomer ? (
          selectedMotorcycle ? (
            selectedVisit ? (
              <ServiceVisitPage
                customer={selectedCustomer}
                motorcycle={selectedMotorcycle}
                visit={selectedVisit}
                onBack={() => setSelectedVisit(null)}
              />
            ) : (
              <MotorcycleDetailsPage
                customer={selectedCustomer}
                motorcycle={selectedMotorcycle}
                onBack={handleBackToCustomer}
                onSelectVisit={setSelectedVisit}
              />
            )
          ) : (
            <CustomerDetailsPage
              customer={selectedCustomer}
              onBack={handleBackToCustomers}
              onSelectMotorcycle={setSelectedMotorcycle}
            />
          )
        ) : (
          <CustomersPage
            onSelectCustomer={handleSelectCustomer}
          />
        )
      ) : (
        <section className="placeholder-page">
          <h1>Coming next</h1>
          <p>
            This section will be implemented as its own application slice.
          </p>
        </section>
      )}
    </AppShell>
  );
}

export default App;

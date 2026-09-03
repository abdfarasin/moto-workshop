import { useState } from "react";

import { AppShell, type AppSection } from "./components/AppShell";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { CustomerDetailsPage } from "./features/customers/CustomerDetailsPage";
import { CustomersPage } from "./features/customers/CustomersPage";
import { InventoryItemDetailsPage } from "./features/inventory/InventoryItemDetailsPage";
import { InventoryPage } from "./features/inventory/InventoryPage";
import { InvoiceDetailsPage } from "./features/invoices/InvoiceDetailsPage";
import { InvoicesPage } from "./features/invoices/InvoicesPage";
import type { InvoiceDirectoryStatusFilter } from "./features/invoices/api/invoiceApi";
import { MotorcycleDetailsPage } from "./features/motorcycles/MotorcycleDetailsPage";
import { MotorcyclesPage } from "./features/motorcycles/MotorcyclesPage";
import {
  loadServiceVisitWorkspace,
  type ServiceVisitDirectoryStatusFilter,
  type ServiceVisitWorkspace,
} from "./features/service/api/serviceVisitApi";
import { NewServiceVisitDialog } from "./features/service/new-visit/NewServiceVisitDialog";
import { ServiceVisitPage } from "./features/service/ServiceVisitPage";
import { ServiceVisitsPage } from "./features/service/directory/ServiceVisitsPage";

import "./App.css";

function App() {
  const [activeSection, setActiveSection] = useState<AppSection>("customers");
  const [selectedCustomerId, setSelectedCustomerId] = useState<number | null>(null);
  const [selectedMotorcycleId, setSelectedMotorcycleId] = useState<number | null>(null);
  const [selectedInventoryItemId, setSelectedInventoryItemId] = useState<number | null>(null);
  const [inventoryReturnSection, setInventoryReturnSection] =
    useState<AppSection | null>(null);
  const [customerReturnMotorcycleId, setCustomerReturnMotorcycleId] =
    useState<number | null>(null);
  const [activeServiceVisitWorkspace, setActiveServiceVisitWorkspace] =
    useState<ServiceVisitWorkspace | null>(null);
  const [newServiceVisitOpen, setNewServiceVisitOpen] = useState(false);
  const [selectedInvoiceId, setSelectedInvoiceId] = useState<number | null>(null);
  const [invoiceReturnWorkspace, setInvoiceReturnWorkspace] =
    useState<ServiceVisitWorkspace | null>(null);
  const [invoiceReturnSection, setInvoiceReturnSection] =
    useState<AppSection | null>(null);
  const [serviceReturnInvoiceId, setServiceReturnInvoiceId] =
    useState<number | null>(null);
  const [serviceInitialFilter, setServiceInitialFilter] =
    useState<ServiceVisitDirectoryStatusFilter>("ACTIVE");
  const [invoiceInitialFilter, setInvoiceInitialFilter] =
    useState<InvoiceDirectoryStatusFilter>("ALL");

  function handleSectionChange(section: AppSection) {
    setActiveSection(section);
    setSelectedCustomerId(null);
    setSelectedMotorcycleId(null);
    setSelectedInventoryItemId(null);
    setInventoryReturnSection(null);
    setCustomerReturnMotorcycleId(null);
    setActiveServiceVisitWorkspace(null);
    setNewServiceVisitOpen(false);
    setSelectedInvoiceId(null);
    setInvoiceReturnWorkspace(null);
    setInvoiceReturnSection(null);
    setServiceReturnInvoiceId(null);
    if (section === "service") setServiceInitialFilter("ACTIVE");
    if (section === "invoices") setInvoiceInitialFilter("ALL");
  }

  return (
    <AppShell
      activeSection={activeSection}
      onSectionChange={handleSectionChange}
      onNewServiceVisit={() => setNewServiceVisitOpen(true)}
    >
      <>
        {activeServiceVisitWorkspace !== null ? (
          <ServiceVisitPage
            workspace={activeServiceVisitWorkspace}
            onBack={() => {
              setActiveServiceVisitWorkspace(null);
              if (serviceReturnInvoiceId !== null) {
                setSelectedInvoiceId(serviceReturnInvoiceId);
                setServiceReturnInvoiceId(null);
                setActiveSection("invoices");
              }
            }}
            onOpenInvoice={(invoiceId) => {
              setInvoiceReturnWorkspace(activeServiceVisitWorkspace);
              setInvoiceReturnSection(activeSection);
              setActiveServiceVisitWorkspace(null);
              setSelectedInvoiceId(invoiceId);
              setActiveSection("invoices");
            }}
          />
        ) : activeSection === "dashboard" ? (
          <DashboardPage
            onOpenServiceVisit={setActiveServiceVisitWorkspace}
            onOpenInvoice={(invoiceId) => {
              setInvoiceReturnWorkspace(null);
              setInvoiceReturnSection("dashboard");
              setSelectedInvoiceId(invoiceId);
              setActiveSection("invoices");
            }}
            onOpenInventoryItem={(inventoryItemId) => {
              setInventoryReturnSection("dashboard");
              setSelectedInventoryItemId(inventoryItemId);
              setActiveSection("inventory");
            }}
            onShowService={(filter) => {
              setServiceInitialFilter(filter);
              setActiveSection("service");
            }}
            onShowInventory={() => setActiveSection("inventory")}
            onShowInvoices={(filter) => {
              setInvoiceInitialFilter(filter);
              setActiveSection("invoices");
            }}
          />
        ) : activeSection === "customers" ? (
          selectedCustomerId === null ? (
            <CustomersPage onSelectCustomer={setSelectedCustomerId} />
          ) : (
            <CustomerDetailsPage
              customerId={selectedCustomerId}
              onBack={() => {
                setSelectedCustomerId(null);
                if (customerReturnMotorcycleId !== null) {
                  setActiveSection("motorcycles");
                  setSelectedMotorcycleId(customerReturnMotorcycleId);
                  setCustomerReturnMotorcycleId(null);
                }
              }}
              onOpenServiceVisit={setActiveServiceVisitWorkspace}
            />
          )
        ) : activeSection === "motorcycles" ? (
          selectedMotorcycleId === null ? (
            <MotorcyclesPage onSelectMotorcycle={setSelectedMotorcycleId} />
          ) : (
            <MotorcycleDetailsPage
              motorcycleId={selectedMotorcycleId}
              onBack={() => setSelectedMotorcycleId(null)}
              onOpenCustomer={(customerId) => {
                setCustomerReturnMotorcycleId(selectedMotorcycleId);
                setSelectedCustomerId(customerId);
                setActiveSection("customers");
              }}
              onOpenServiceVisit={setActiveServiceVisitWorkspace}
            />
          )
        ) : activeSection === "service" ? (
          <ServiceVisitsPage
            initialStatusFilter={serviceInitialFilter}
            onOpenServiceVisit={setActiveServiceVisitWorkspace}
          />
        ) : activeSection === "inventory" ? (
          selectedInventoryItemId === null ? (
            <InventoryPage onSelectItem={setSelectedInventoryItemId} />
          ) : (
            <InventoryItemDetailsPage
              inventoryItemId={selectedInventoryItemId}
              onBack={() => {
                setSelectedInventoryItemId(null);
                if (inventoryReturnSection !== null) {
                  setActiveSection(inventoryReturnSection);
                  setInventoryReturnSection(null);
                }
              }}
            />
          )
        ) : activeSection === "invoices" ? (
          selectedInvoiceId === null ? (
            <InvoicesPage
              initialStatusFilter={invoiceInitialFilter}
              onSelectInvoice={setSelectedInvoiceId}
            />
          ) : (
            <InvoiceDetailsPage
              invoiceId={selectedInvoiceId}
              onBack={() => {
                if (invoiceReturnWorkspace !== null) {
                  setActiveServiceVisitWorkspace(invoiceReturnWorkspace);
                  setActiveSection(invoiceReturnSection ?? "service");
                  setInvoiceReturnWorkspace(null);
                  setInvoiceReturnSection(null);
                } else if (invoiceReturnSection !== null) {
                  setActiveSection(invoiceReturnSection);
                  setInvoiceReturnSection(null);
                }
                setSelectedInvoiceId(null);
              }}
              onOpenServiceVisit={async (serviceVisitId) => {
                const workspace = await loadServiceVisitWorkspace(serviceVisitId);
                setServiceReturnInvoiceId(selectedInvoiceId);
                setActiveServiceVisitWorkspace(workspace);
                setActiveSection("service");
              }}
            />
          )
        ) : (
          <section className="placeholder-page">
            <h1>Coming next</h1>
            <p>This section will be implemented as its own application slice.</p>
          </section>
        )}

        <NewServiceVisitDialog
          open={newServiceVisitOpen}
          onClose={() => setNewServiceVisitOpen(false)}
          onCreated={(workspace) => {
            setNewServiceVisitOpen(false);
            setActiveServiceVisitWorkspace(workspace);
          }}
        />
      </>
    </AppShell>
  );
}

export default App;

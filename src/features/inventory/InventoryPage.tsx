import {
  type FormEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { Boxes, Plus, Search } from "lucide-react";

import {
  createInventoryItem,
  listInventoryUnits,
  searchInventoryItems,
} from "./api/inventoryApi";
import type { InventoryItemSummary, InventoryUnit } from "./api/inventoryApi";
import { parseJodInputToFils } from "../service/functions/parseJodInputToFils";
import { parseNonnegativeScaledQuantityInput } from "./functions/parseNonnegativeScaledQuantityInput";

import "./Inventory.css";

const INVENTORY_DIRECTORY_LIMIT = 50;

type InventoryPageProps = {
  onSelectItem: (inventoryItemId: number) => void;
};

export function InventoryPage({ onSelectItem }: InventoryPageProps) {
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [items, setItems] = useState<InventoryItemSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadFailed, setLoadFailed] = useState(false);
  const [newItemOpen, setNewItemOpen] = useState(false);
  const [revision, setRevision] = useState(0);
  const requestSequence = useRef(0);

  const loadItems = useCallback(async () => {
    const request = ++requestSequence.current;
    setLoading(true);
    setLoadFailed(false);

    try {
      const rows = await searchInventoryItems({
        query: submittedQuery,
        limit: INVENTORY_DIRECTORY_LIMIT,
      });
      if (request === requestSequence.current) {
        setItems(rows);
      }
    } catch {
      if (request === requestSequence.current) {
        setItems([]);
        setLoadFailed(true);
      }
    } finally {
      if (request === requestSequence.current) {
        setLoading(false);
      }
    }
  }, [revision, submittedQuery]);

  useEffect(() => {
    void loadItems();
    return () => {
      requestSequence.current += 1;
    };
  }, [loadItems]);

  function submitSearch(event: FormEvent) {
    event.preventDefault();
    setSubmittedQuery(query.trim());
  }

  return (
    <section className="inventory-page">
      <div className="page-header">
        <div>
          <h1>Inventory</h1>
          <p>Parts, materials, prices, and auditable stock.</p>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={() => setNewItemOpen(true)}
        >
          <Plus size={17} />
          New Inventory Item
        </button>
      </div>

      <div className="content-panel">
        <form className="inventory-toolbar" onSubmit={submitSearch}>
          <label>
            <Search size={17} />
            <input
              aria-label="Search Inventory"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Item name or SKU..."
            />
          </label>
          <button type="submit" className="secondary-button">Search</button>
        </form>

        {loadFailed ? (
          <div className="empty-state" role="alert">
            <strong>Inventory could not be loaded</strong>
            <span>Please try again.</span>
          </div>
        ) : (
          <div className="table-wrapper">
            <table className="data-table inventory-table">
              <thead>
                <tr>
                  <th>Item</th>
                  <th>SKU</th>
                  <th>Unit</th>
                  <th>Current quantity</th>
                  <th>Minimum</th>
                  <th className="money-column">Selling price</th>
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr
                    key={item.id}
                    role="button"
                    tabIndex={0}
                    aria-label={`Open Inventory Item ${item.id}`}
                    onClick={() => onSelectItem(item.id)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        onSelectItem(item.id);
                      }
                    }}
                  >
                    <td>
                      <strong>{item.name}</strong>
                      <span className="inventory-secondary">#{item.id}</span>
                    </td>
                    <td>{item.sku ?? "—"}</td>
                    <td>{item.unitName}</td>
                    <td>
                      <strong>{formatQuantity(item.currentQuantity, item.quantityScale)}</strong>
                      {item.lowStock && <span className="inventory-low">Low stock</span>}
                    </td>
                    <td>{formatQuantity(item.minimumStockQuantity, item.quantityScale)}</td>
                    <td className="money-column">{formatMoney(item.defaultSellingPriceFils)}</td>
                  </tr>
                ))}
              </tbody>
            </table>

            {loading && (
              <div className="empty-state">
                <strong>Loading Inventory...</strong>
              </div>
            )}
            {!loading && items.length === 0 && (
              <div className="empty-state">
                <Boxes size={24} />
                <strong>No Inventory Items found</strong>
              </div>
            )}
          </div>
        )}
      </div>

      {newItemOpen && (
        <NewInventoryDialog
          onClose={() => setNewItemOpen(false)}
          onCreated={() => {
            setNewItemOpen(false);
            setRevision((value) => value + 1);
          }}
        />
      )}
    </section>
  );
}

type NewInventoryDialogProps = {
  onClose: () => void;
  onCreated: () => void;
};

function NewInventoryDialog({ onClose, onCreated }: NewInventoryDialogProps) {
  const [units, setUnits] = useState<InventoryUnit[]>([]);
  const [name, setName] = useState("");
  const [sku, setSku] = useState("");
  const [unitId, setUnitId] = useState("");
  const [purchasePrice, setPurchasePrice] = useState("");
  const [sellingPrice, setSellingPrice] = useState("");
  const [minimumStock, setMinimumStock] = useState("0");
  const [openingStock, setOpeningStock] = useState("0");
  const [notes, setNotes] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    void listInventoryUnits()
      .then((rows) => {
        setUnits(rows);
        setUnitId(rows[0]?.id.toString() ?? "");
      })
      .catch(() => setError("Could not load Inventory Units."));
  }, []);

  const selectedUnit = units.find((unit) => unit.id === Number(unitId));

  async function save(event: FormEvent) {
    event.preventDefault();
    if (!selectedUnit) {
      setError("Select an active Inventory Unit.");
      return;
    }

    const sellingPriceFils = parseJodInputToFils(sellingPrice);
    const purchasePriceFils = purchasePrice.trim() === ""
      ? null
      : parseJodInputToFils(purchasePrice);
    const minimumStockQuantity = parseNonnegativeScaledQuantityInput(
      minimumStock,
      selectedUnit.quantityScale,
    );
    const openingQuantity = parseNonnegativeScaledQuantityInput(
      openingStock,
      selectedUnit.quantityScale,
    );

    if (
      name.trim() === "" ||
      sellingPriceFils === null ||
      (purchasePriceFils === null && purchasePrice.trim() !== "") ||
      minimumStockQuantity === null ||
      openingQuantity === null
    ) {
      setError("Review the required values, JOD prices, and Unit precision.");
      return;
    }

    setSaving(true);
    setError(null);
    try {
      await createInventoryItem({
        name: name.trim(),
        sku: blankToNull(sku),
        unitId: selectedUnit.id,
        defaultPurchasePriceFils: purchasePriceFils,
        defaultSellingPriceFils: sellingPriceFils,
        minimumStockQuantity,
        notes: blankToNull(notes),
        openingQuantity,
        createdAt: Date.now(),
      });
      onCreated();
    } catch {
      setError("The Inventory Item could not be saved.");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="inventory-backdrop">
      <form
        className="inventory-dialog"
        role="dialog"
        aria-label="New Inventory Item"
        onSubmit={save}
      >
        <h2>New Inventory Item</h2>
        <label>Name<input aria-label="Item name" value={name} onChange={(event) => setName(event.target.value)} /></label>
        <label>SKU<input aria-label="SKU" value={sku} onChange={(event) => setSku(event.target.value)} /></label>
        <label>
          Unit
          <select aria-label="Unit" value={unitId} onChange={(event) => setUnitId(event.target.value)}>
            {units.map((unit) => <option key={unit.id} value={unit.id}>{unit.name}</option>)}
          </select>
        </label>
        <label>Purchase price (JD)<input aria-label="Purchase price" value={purchasePrice} onChange={(event) => setPurchasePrice(event.target.value)} /></label>
        <label>Selling price (JD)<input aria-label="Selling price" value={sellingPrice} onChange={(event) => setSellingPrice(event.target.value)} /></label>
        <label>Minimum stock<input aria-label="Minimum stock" value={minimumStock} onChange={(event) => setMinimumStock(event.target.value)} /></label>
        <label>Opening stock<input aria-label="Opening stock" value={openingStock} onChange={(event) => setOpeningStock(event.target.value)} /></label>
        <label>Notes<textarea aria-label="Notes" value={notes} onChange={(event) => setNotes(event.target.value)} /></label>
        {error && <p role="alert">{error}</p>}
        <div className="inventory-dialog-actions">
          <button type="button" className="secondary-button" onClick={onClose}>Cancel</button>
          <button type="submit" className="primary-button" disabled={saving}>Create Item</button>
        </div>
      </form>
    </div>
  );
}

function blankToNull(value: string): string | null {
  return value.trim() === "" ? null : value.trim();
}

export function formatMoney(fils: number): string {
  return `${(fils / 1000).toFixed(3)} JD`;
}

export function formatQuantity(value: number, scale: number): string {
  return (value / scale).toFixed(Math.log10(scale));
}

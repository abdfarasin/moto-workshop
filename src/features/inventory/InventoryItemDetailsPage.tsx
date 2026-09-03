import { type FormEvent, useCallback, useEffect, useState } from "react";
import { ArrowLeft, Edit3 } from "lucide-react";

import {
  adjustInventoryStock,
  loadInventoryItemDetails,
  updateInventoryItem,
} from "./api/inventoryApi";
import type { InventoryItemDetails } from "./api/inventoryApi";
import { parseJodInputToFils } from "../service/functions/parseJodInputToFils";
import { parseScaledQuantityInput } from "../service/functions/parseScaledQuantityInput";
import { formatMoney, formatQuantity } from "./InventoryPage";
import { parseNonnegativeScaledQuantityInput } from "./functions/parseNonnegativeScaledQuantityInput";

import "./Inventory.css";

type InventoryItemDetailsPageProps = {
  inventoryItemId: number;
  onBack: () => void;
};

export function InventoryItemDetailsPage({
  inventoryItemId,
  onBack,
}: InventoryItemDetailsPageProps) {
  const [item, setItem] = useState<InventoryItemDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadFailed, setLoadFailed] = useState(false);
  const [adjustmentOpen, setAdjustmentOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);

  const loadItem = useCallback(async () => {
    setLoading(true);
    setLoadFailed(false);
    try {
      setItem(await loadInventoryItemDetails(inventoryItemId));
    } catch {
      setLoadFailed(true);
    } finally {
      setLoading(false);
    }
  }, [inventoryItemId]);

  useEffect(() => {
    void loadItem();
  }, [loadItem]);

  if (loading) {
    return <div className="empty-state"><strong>Loading Inventory Item...</strong></div>;
  }
  if (loadFailed || !item) {
    return (
      <div className="empty-state" role="alert">
        <strong>Inventory Item could not be loaded</strong>
        <button type="button" onClick={onBack}>Back</button>
      </div>
    );
  }

  return (
    <section className="inventory-details">
      <button type="button" className="back-button" onClick={onBack}>
        <ArrowLeft size={17} />
        Inventory
      </button>

      <div className="page-header">
        <div>
          <h1>{item.name}</h1>
          <p>{item.sku ?? "No SKU"} · {item.unitName}</p>
        </div>
        <div className="header-actions">
          <button type="button" className="secondary-button" onClick={() => setEditOpen(true)}>
            <Edit3 size={16} />
            Edit Item
          </button>
          <button type="button" className="primary-button" onClick={() => setAdjustmentOpen(true)}>
            <PlusMinus size={16} />
            Adjust Stock
          </button>
        </div>
      </div>

      {item.currentQuantity < 0 && (
        <p className="inventory-negative" role="alert">
          Stock is negative. The ledger remains valid, but a physical count is recommended.
        </p>
      )}

      <div className="inventory-info-grid">
        <Info label="Current quantity" value={formatQuantity(item.currentQuantity, item.quantityScale)} />
        <Info label="Selling price" value={formatMoney(item.defaultSellingPriceFils)} />
        <Info
          label="Purchase price"
          value={item.defaultPurchasePriceFils === null
            ? "Not recorded"
            : formatMoney(item.defaultPurchasePriceFils)}
        />
        <Info label="Minimum stock" value={formatQuantity(item.minimumStockQuantity, item.quantityScale)} />
      </div>

      {item.notes && (
        <div className="content-panel inventory-notes">
          <h2>Notes</h2>
          <p>{item.notes}</p>
        </div>
      )}

      <section>
        <div className="section-header">
          <div>
            <h2>Stock Movement History</h2>
            <p>Newest immutable ledger entries.</p>
          </div>
        </div>
        <div className="content-panel">
          <table className="data-table">
            <thead>
              <tr>
                <th>Date</th>
                <th>Type</th>
                <th>Quantity change</th>
                <th>Reason</th>
                <th>Service Part</th>
              </tr>
            </thead>
            <tbody>
              {item.movements.map((movement) => (
                <tr key={movement.id}>
                  <td>{new Date(movement.createdAt).toLocaleString()}</td>
                  <td>{movement.movementType.replace(/_/g, " ")}</td>
                  <td>
                    {movement.quantityDelta > 0 ? "+" : ""}
                    {formatQuantity(movement.quantityDelta, item.quantityScale)}
                  </td>
                  <td>{movement.notes ?? "—"}</td>
                  <td>{movement.serviceVisitPartId ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {item.movements.length === 0 && (
            <div className="empty-state"><strong>No stock movements yet</strong></div>
          )}
        </div>
      </section>

      {adjustmentOpen && (
        <AdjustmentDialog
          item={item}
          onClose={() => setAdjustmentOpen(false)}
          onSaved={async () => {
            setAdjustmentOpen(false);
            await loadItem();
          }}
        />
      )}
      {editOpen && (
        <EditDialog
          item={item}
          onClose={() => setEditOpen(false)}
          onSaved={async () => {
            setEditOpen(false);
            await loadItem();
          }}
        />
      )}
    </section>
  );
}

function PlusMinus({ size }: { size: number }) {
  return <span aria-hidden="true" style={{ fontSize: size }}>±</span>;
}

function Info({ label, value }: { label: string; value: string }) {
  return (
    <div className="info-card">
      <div>
        <span className="info-label">{label}</span>
        <strong>{value}</strong>
      </div>
    </div>
  );
}

type InventoryDialogProps = {
  item: InventoryItemDetails;
  onClose: () => void;
  onSaved: () => Promise<void>;
};

function AdjustmentDialog({ item, onClose, onSaved }: InventoryDialogProps) {
  const [delta, setDelta] = useState("");
  const [reason, setReason] = useState("");
  const [error, setError] = useState(false);

  async function save(event: FormEvent) {
    event.preventDefault();
    const raw = delta.trim();
    const sign = raw.startsWith("-") ? -1 : 1;
    const magnitude = raw.replace(/^[+-]/, "");
    const scaledMagnitude = parseScaledQuantityInput(magnitude, item.quantityScale);
    if (scaledMagnitude === null) {
      setError(true);
      return;
    }

    try {
      await adjustInventoryStock({
        inventoryItemId: item.id,
        quantityDelta: sign * scaledMagnitude,
        notes: reason.trim() === "" ? null : reason.trim(),
        createdAt: Date.now(),
      });
      await onSaved();
    } catch {
      setError(true);
    }
  }

  return (
    <div className="inventory-backdrop">
      <form className="inventory-dialog" role="dialog" aria-label="Adjust Stock" onSubmit={save}>
        <h2>Adjust Stock</h2>
        <p>Use a positive quantity to add stock or a negative quantity to remove it.</p>
        <label>Quantity delta<input aria-label="Quantity delta" value={delta} onChange={(event) => setDelta(event.target.value)} /></label>
        <label>Reason<textarea aria-label="Reason" value={reason} onChange={(event) => setReason(event.target.value)} /></label>
        {error && <p role="alert">Enter a non-zero quantity supported by this Unit.</p>}
        <div className="inventory-dialog-actions">
          <button type="button" onClick={onClose}>Cancel</button>
          <button type="submit" className="primary-button">Save Adjustment</button>
        </div>
      </form>
    </div>
  );
}

function EditDialog({ item, onClose, onSaved }: InventoryDialogProps) {
  const [name, setName] = useState(item.name);
  const [sku, setSku] = useState(item.sku ?? "");
  const [purchasePrice, setPurchasePrice] = useState(
    item.defaultPurchasePriceFils === null
      ? ""
      : (item.defaultPurchasePriceFils / 1000).toFixed(3),
  );
  const [sellingPrice, setSellingPrice] = useState(
    (item.defaultSellingPriceFils / 1000).toFixed(3),
  );
  const [minimumStock, setMinimumStock] = useState(
    formatQuantity(item.minimumStockQuantity, item.quantityScale),
  );
  const [notes, setNotes] = useState(item.notes ?? "");
  const [error, setError] = useState(false);

  async function save(event: FormEvent) {
    event.preventDefault();
    const sellingPriceFils = parseJodInputToFils(sellingPrice);
    const purchasePriceFils = purchasePrice.trim() === ""
      ? null
      : parseJodInputToFils(purchasePrice);
    const minimumStockQuantity = parseNonnegativeScaledQuantityInput(
      minimumStock,
      item.quantityScale,
    );

    if (
      name.trim() === "" ||
      sellingPriceFils === null ||
      (purchasePriceFils === null && purchasePrice.trim() !== "") ||
      minimumStockQuantity === null
    ) {
      setError(true);
      return;
    }

    try {
      await updateInventoryItem({
        inventoryItemId: item.id,
        name: name.trim(),
        sku: sku.trim() || null,
        defaultPurchasePriceFils: purchasePriceFils,
        defaultSellingPriceFils: sellingPriceFils,
        minimumStockQuantity,
        notes: notes.trim() || null,
        updatedAt: Date.now(),
      });
      await onSaved();
    } catch {
      setError(true);
    }
  }

  return (
    <div className="inventory-backdrop">
      <form className="inventory-dialog" role="dialog" aria-label="Edit Inventory Item" onSubmit={save}>
        <h2>Edit Inventory Item</h2>
        <p>Unit remains {item.unitName}; stock changes use the ledger.</p>
        <label>Name<input value={name} onChange={(event) => setName(event.target.value)} /></label>
        <label>SKU<input value={sku} onChange={(event) => setSku(event.target.value)} /></label>
        <label>Purchase price (JD)<input value={purchasePrice} onChange={(event) => setPurchasePrice(event.target.value)} /></label>
        <label>Selling price (JD)<input value={sellingPrice} onChange={(event) => setSellingPrice(event.target.value)} /></label>
        <label>Minimum stock<input value={minimumStock} onChange={(event) => setMinimumStock(event.target.value)} /></label>
        <label>Notes<textarea value={notes} onChange={(event) => setNotes(event.target.value)} /></label>
        {error && <p role="alert">Review the Inventory Item values.</p>}
        <div className="inventory-dialog-actions">
          <button type="button" onClick={onClose}>Cancel</button>
          <button type="submit" className="primary-button">Save Item</button>
        </div>
      </form>
    </div>
  );
}

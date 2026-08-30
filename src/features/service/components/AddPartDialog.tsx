import { useState } from "react";
import { PackagePlus, X } from "lucide-react";

import "./AddPartDialog.css";
import { previewInventoryItems } from "../../inventory/inventoryPreviewData";
import { SelectedInventoryItemInfo } from "./SelectedInventoryItemInfo";

type AddPartDialogProps = {
  open: boolean;
  onClose: () => void;
};

export function AddPartDialog({
  open,
  onClose,
}: AddPartDialogProps) {
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    const selectedItem =
    previewInventoryItems.find((item) => item.id === selectedItemId) ?? null;
    function handleClose() {
    setSelectedItemId(null);
    onClose();
}
  if (!open) {
    return null;
  }
  

  return (
    <div
        className="dialog-backdrop"
        onClick={(event) => {
            if (event.target === event.currentTarget) {
            handleClose();
            }
        }}
        >
      <section
        className="add-part-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="add-part-dialog-title"
      >
        <div className="add-part-dialog-header">
          <div className="add-part-dialog-title">
            <PackagePlus size={19} />

            <div>
              <h2 id="add-part-dialog-title">Add Part</h2>
              <p>Add an inventory item to this service visit.</p>
            </div>
          </div>

          <button
            type="button"
            className="dialog-close-button"
            aria-label="Close"
            onClick={handleClose}
          >
            <X size={18} />
          </button>
        </div>

        <div className="add-part-dialog-body">
        <div className="add-part-field">
            <label htmlFor="inventory-item">Inventory Item</label>

            <select
            id="inventory-item"
            className="add-part-select"
            defaultValue=""
            onChange={(e) => setSelectedItemId(Number(e.target.value))}
            >
            <option value="" disabled>
                Select an item...
            </option>

            {previewInventoryItems.map((item) => (
                <option key={item.id} value={item.id}>
                {item.name} — {item.unitName}
                </option>
            ))}
            </select>
            {selectedItem && (
            <SelectedInventoryItemInfo item={selectedItem} />
            )}
        </div>
        </div>
      </section>
    </div>
  );
}
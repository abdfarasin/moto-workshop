import { useState } from "react";
import { PackagePlus, X } from "lucide-react";

import "./AddPartDialog.css";
import { previewInventoryItems } from "../../inventory/inventoryPreviewData";
import { SelectedInventoryItemInfo } from "./SelectedInventoryItemInfo";
import { PartQuantityField } from "./PartQuantityField";
import { PartUnitPriceField } from "./PartUnitPriceField";
import { formatFilsForInput } from "../functions/formatFilsForInput";
import { PartLineTotalPreview } from "./PartLineTotalPreview";
import { calculatePartLineTotalPreview } from "../functions/calculatePartLineTotalPreview";
import { parseJodInputToFils } from "../functions/parseJodInputToFils";
import { parseScaledQuantityInput } from "../functions/parseScaledQuantityInput";
import { AddPartDialogActions } from "./AddPartDialogActions";
import { PartStockWarning } from "./PartStockWarning";
import { isPartQuantityAboveStock } from "../functions/isPartQuantityAboveStock";

type AddPartDialogSubmit = {
  inventoryItemId: number;
  quantity: number;
  unitPriceFils: number;
};

type AddPartDialogProps = {
  open: boolean;
  onClose: () => void;
  onAdd?: (input: AddPartDialogSubmit) => void;
};

export function AddPartDialog({
  open,
  onClose,
  onAdd,
}: AddPartDialogProps) {
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);
    const [quantity, setQuantity] = useState("");
    const [unitPrice, setUnitPrice] = useState("");

    const selectedItem =
    previewInventoryItems.find((item) => item.id === selectedItemId) ?? null;
    const scaledQuantity = selectedItem
  ? parseScaledQuantityInput(quantity, selectedItem.quantityScale)
  : null;

    const unitPriceFils = parseJodInputToFils(unitPrice);

    const lineTotalFils =
      selectedItem &&
      scaledQuantity !== null &&
      unitPriceFils !== null
        ? calculatePartLineTotalPreview(
            scaledQuantity,
            selectedItem.quantityScale,
            unitPriceFils,
          )
        : null;
        const isAboveStock =
        selectedItem !== null &&
        scaledQuantity !== null &&
        isPartQuantityAboveStock(
          scaledQuantity,
          selectedItem.currentQuantity,
        );
      const canAdd =
        onAdd !== undefined &&
        selectedItem !== null &&
        scaledQuantity !== null &&
        unitPriceFils !== null;
        function handleAdd() {
          if (
            onAdd === undefined ||
            selectedItem === null ||
            scaledQuantity === null ||
            unitPriceFils === null
          ) {
            return;
          }

          onAdd({
            inventoryItemId: selectedItem.id,
            quantity: scaledQuantity,
            unitPriceFils,
          });
        }
    function handleClose() {
    setSelectedItemId(null);
    setQuantity("");
    setUnitPrice("");
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
            onChange={(event) => {
            const itemId = Number(event.target.value);

            const item =
                previewInventoryItems.find((candidate) => candidate.id === itemId) ?? null;

            setSelectedItemId(itemId);
            setQuantity("");

            setUnitPrice(
                item ? formatFilsForInput(item.defaultSellingPriceFils) : "",
            );
            }}           >
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
            {selectedItem && (
            <PartQuantityField
                unitName={selectedItem.unitName}
                quantityScale={selectedItem.quantityScale}
                value={quantity}
                onChange={setQuantity}
            />
            )}

            {selectedItem && (
            <PartUnitPriceField
                value={unitPrice}
                onChange={setUnitPrice}
            />
            )}
            {selectedItem && (
            <PartLineTotalPreview lineTotalFils={lineTotalFils} />
            )}
            <PartStockWarning visible={isAboveStock} />

            </div>
            <AddPartDialogActions
            canAdd={canAdd}
            onCancel={handleClose}
            onAdd={handleAdd}
          />
            </section>
    </div>
  );
}
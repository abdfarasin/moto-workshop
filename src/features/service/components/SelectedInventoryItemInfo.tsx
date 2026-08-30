import type { InventoryItemPreview } from "../../inventory/inventoryPreviewData";

type SelectedInventoryItemInfoProps = {
  item: InventoryItemPreview;
};

function formatQuantity(item: InventoryItemPreview): string {
  const whole = Math.floor(item.currentQuantity / item.quantityScale);
  const remainder = item.currentQuantity % item.quantityScale;

  if (item.quantityScale === 1) {
    return `${whole} ${item.unitName}`;
  }

  const decimalPlaces = Math.log10(item.quantityScale);

  return `${whole}.${remainder
    .toString()
    .padStart(decimalPlaces, "0")} ${item.unitName}`;
}

function formatMoney(fils: number): string {
  const whole = Math.floor(fils / 1000);
  const remainder = fils % 1000;

  return `${whole}.${remainder.toString().padStart(3, "0")} JD`;
}

export function SelectedInventoryItemInfo({
  item,
}: SelectedInventoryItemInfoProps) {
  return (
    <div className="selected-inventory-info">
      <div>
        <span>Available stock</span>
        <strong>{formatQuantity(item)}</strong>
      </div>

      <div>
        <span>Default price</span>
        <strong>
          {formatMoney(item.defaultSellingPriceFils)} / {item.unitName}
        </strong>
      </div>
    </div>
  );
}
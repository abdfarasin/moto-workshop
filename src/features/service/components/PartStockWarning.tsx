import { AlertTriangle } from "lucide-react";

import "./PartStockWarning.css";

type PartStockWarningProps = {
  visible: boolean;
};

export function PartStockWarning({
  visible,
}: PartStockWarningProps) {
  if (!visible) {
    return null;
  }

  return (
    <div className="part-stock-warning" role="status">
      <AlertTriangle size={16} />
      <span>
        Requested quantity exceeds current stock. Inventory will become negative.
      </span>
    </div>
  );
}
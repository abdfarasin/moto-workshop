import "./LaborChargeField.css";

type LaborChargeFieldProps = {
  laborChargeFils: number;
  editable: boolean;
};

function formatMoney(fils: number) {
  return `${(fils / 1000).toFixed(3)} JD`;
}

export function LaborChargeField({
  laborChargeFils,
  editable,
}: LaborChargeFieldProps) {
  if (!editable) {
    return <strong>{formatMoney(laborChargeFils)}</strong>;
  }

  return (
    <div className="labor-charge-input">
      <input
        type="number"
        min="0"
        step="1"
        defaultValue={(laborChargeFils / 1000).toFixed(3)}
      />

      <span>JD</span>
    </div>
  );
}
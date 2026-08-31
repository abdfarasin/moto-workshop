import "./PartQuantityField.css";

type PartQuantityFieldProps = {
  unitName: string;
  quantityScale: 1 | 10 | 100 | 1000;
  value: string;
  onChange: (value: string) => void;
};

export function PartQuantityField({
  unitName,
  quantityScale,
  value,
  onChange,
}: PartQuantityFieldProps) {
  const placeholder =
    quantityScale === 1
      ? "1"
      : quantityScale === 10
        ? "1.0"
        : quantityScale === 100
          ? "1.00"
          : "1.000";

  return (
    <div className="part-quantity-field">
      <label htmlFor="part-quantity">Quantity</label>

      <div className="part-quantity-input">
        <input
          id="part-quantity"
          type="text"
          inputMode="decimal"
          value={value}
          placeholder={placeholder}
          onChange={(event) => onChange(event.target.value)}
        />

        <span>{unitName}</span>
      </div>
    </div>
  );
}
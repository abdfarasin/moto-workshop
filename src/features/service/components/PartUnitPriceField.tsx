import "./PartUnitPriceField.css";

type PartUnitPriceFieldProps = {
  value: string;
  onChange: (value: string) => void;
};

export function PartUnitPriceField({
  value,
  onChange,
}: PartUnitPriceFieldProps) {
  return (
    <div className="part-unit-price-field">
      <label htmlFor="part-unit-price">Unit Price</label>

      <div className="part-unit-price-input">
        <input
          id="part-unit-price"
          type="text"
          inputMode="decimal"
          value={value}
          placeholder="0.000"
          onChange={(event) => onChange(event.target.value)}
        />

        <span>JD</span>
      </div>
    </div>
  );
}
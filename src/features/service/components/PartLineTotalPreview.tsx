import "./PartLineTotalPreview.css";

type PartLineTotalPreviewProps = {
  lineTotalFils: number | null;
};

export function PartLineTotalPreview({
  lineTotalFils,
}: PartLineTotalPreviewProps) {
  if (lineTotalFils === null) {
    return null;
  }

  const whole = Math.floor(lineTotalFils / 1000);
  const remainder = lineTotalFils % 1000;

  const formattedTotal =
    `${whole}.${remainder.toString().padStart(3, "0")}`;

  return (
    <div className="part-line-total-preview">
      <span>Line Total</span>

      <strong>{formattedTotal} JD</strong>
    </div>
  );
}
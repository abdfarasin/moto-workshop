import "./AddPartDialogActions.css";

type AddPartDialogActionsProps = {
  canAdd: boolean;
  onCancel: () => void;
  onAdd: () => void;
};

export function AddPartDialogActions({
  canAdd,
  onCancel,
  onAdd,
}: AddPartDialogActionsProps) {
  return (
    <div className="add-part-dialog-actions">
      <button
        type="button"
        className="secondary-button"
        onClick={onCancel}
      >
        Cancel
      </button>

      <button
        type="button"
        className="primary-button"
        disabled={!canAdd}
        onClick={onAdd}
      >
        Add Part
      </button>
    </div>
  );
}
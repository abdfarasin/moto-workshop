import type { ServiceHistoryPreview } from "../../customers/customerPreviewData";

type JobDetailsCardProps = {
  visit: ServiceHistoryPreview;
};

export function JobDetailsCard({ visit }: JobDetailsCardProps) {
  return (
    <section className="workspace-card">
      <div className="workspace-card-header">
        <h2>Job Details</h2>
      </div>

      <div className="service-field">
        <label>Customer Complaint</label>

        <div className="read-only-field">
          {visit.complaint}
        </div>
      </div>

      <div className="service-field">
        <label>Diagnosis</label>

        {visit.status === "OPEN" ? (
          <textarea
            className="service-textarea"
            defaultValue={visit.diagnosis ?? ""}
            placeholder="Enter diagnosis..."
            rows={3}
          />
        ) : (
          <div className="read-only-field multiline">
            {visit.diagnosis ?? "Not recorded"}
          </div>
        )}
      </div>

      <div className="service-field">
        <label>Work Performed</label>

        {visit.status === "OPEN" ? (
          <textarea
            className="service-textarea"
            defaultValue={visit.workPerformed ?? ""}
            placeholder="Describe the work performed..."
            rows={3}
          />
        ) : (
          <div className="read-only-field multiline">
            {visit.workPerformed ?? "Not recorded"}
          </div>
        )}
      </div>
    </section>
  );
}
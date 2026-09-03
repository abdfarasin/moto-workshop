use crate::domain::service_visit::ServiceVisitStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceIssueInput {
    pub invoice_id: i64,
    pub service_visit_id: i64,
    pub service_visit_status: ServiceVisitStatus,
    pub completed_at: Option<i64>,
    pub issued_at: i64,
    pub labor_charge_fils: i64,
    pub active_part_line_totals_fils: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceIssueError {
    InvalidInvoiceId,
    InvalidServiceVisitId,
    ServiceVisitNotInvoiceable,
    InvalidTimestamp,
    InvalidMoney,
    TotalOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceIssue {
    invoice_number: String,
    issued_at: i64,
    labor_charge_fils: i64,
    parts_total_fils: i64,
    total_fils: i64,
}

impl InvoiceIssue {
    pub fn new(input: InvoiceIssueInput) -> Result<Self, InvoiceIssueError> {
        if input.invoice_id <= 0 {
            return Err(InvoiceIssueError::InvalidInvoiceId);
        }
        if input.service_visit_id <= 0 {
            return Err(InvoiceIssueError::InvalidServiceVisitId);
        }
        if !matches!(
            input.service_visit_status,
            ServiceVisitStatus::ReadyForPickup | ServiceVisitStatus::Closed
        ) {
            return Err(InvoiceIssueError::ServiceVisitNotInvoiceable);
        }
        let completed_at = input
            .completed_at
            .ok_or(InvoiceIssueError::InvalidTimestamp)?;
        if input.issued_at < completed_at {
            return Err(InvoiceIssueError::InvalidTimestamp);
        }
        if input.labor_charge_fils < 0
            || input
                .active_part_line_totals_fils
                .iter()
                .any(|value| *value < 0)
        {
            return Err(InvoiceIssueError::InvalidMoney);
        }

        let parts_total_fils = input
            .active_part_line_totals_fils
            .iter()
            .try_fold(0_i64, |total, value| total.checked_add(*value))
            .ok_or(InvoiceIssueError::TotalOverflow)?;
        let total_fils = input
            .labor_charge_fils
            .checked_add(parts_total_fils)
            .ok_or(InvoiceIssueError::TotalOverflow)?;

        Ok(Self {
            invoice_number: format!("INV-{:06}", input.invoice_id),
            issued_at: input.issued_at,
            labor_charge_fils: input.labor_charge_fils,
            parts_total_fils,
            total_fils,
        })
    }

    pub fn invoice_number(&self) -> &str {
        &self.invoice_number
    }

    pub fn issued_at(&self) -> i64 {
        self.issued_at
    }

    pub fn labor_charge_fils(&self) -> i64 {
        self.labor_charge_fils
    }

    pub fn parts_total_fils(&self) -> i64 {
        self.parts_total_fils
    }

    pub fn total_fils(&self) -> i64 {
        self.total_fils
    }
}

use std::{error::Error, fmt};

use rusqlite::{Connection, TransactionBehavior};

use crate::{
    domain::invoice::{InvoiceIssue, InvoiceIssueError, InvoiceIssueInput},
    repositories::invoice::{
        InvoiceDetailsRow, InvoiceDirectoryRow, InvoiceRepository, InvoiceStatusFilter,
    },
};

pub const DEFAULT_INVOICE_DIRECTORY_LIMIT: u32 = 50;
pub const MAX_INVOICE_DIRECTORY_LIMIT: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceDirectoryStatusFilter {
    All,
    Draft,
    Issued,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListInvoicesInput {
    pub query: String,
    pub status_filter: Option<InvoiceDirectoryStatusFilter>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceDirectoryEntry {
    pub id: i64,
    pub service_visit_id: i64,
    pub status: InvoiceStatus,
    pub invoice_number: Option<String>,
    pub issued_at: Option<i64>,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle: String,
    pub plate_number: Option<String>,
    pub total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceDetails {
    pub id: i64,
    pub service_visit_id: i64,
    pub status: InvoiceStatus,
    pub invoice_number: Option<String>,
    pub issued_at: Option<i64>,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle_make_name: String,
    pub motorcycle_model: String,
    pub motorcycle_plate_number: Option<String>,
    pub motorcycle_vin: Option<String>,
    pub motorcycle_chassis_number: Option<String>,
    pub labor_charge_fils: i64,
    pub parts_total_fils: i64,
    pub total_fils: i64,
    pub notes: Option<String>,
    pub lines: Vec<InvoiceLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceLine {
    pub service_visit_part_id: i64,
    pub item_name: String,
    pub unit_name: String,
    pub quantity: i64,
    pub quantity_scale: i64,
    pub unit_price_fils: i64,
    pub line_total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueInvoiceInput {
    pub service_visit_id: i64,
    pub issued_at: i64,
}

#[derive(Debug)]
pub enum InvoiceApplicationError {
    InvoiceNotFound(i64),
    InvoiceAlreadyIssued(i64),
    ServiceVisitNotInvoiceable,
    Validation(InvoiceIssueError),
    Database(rusqlite::Error),
}

impl fmt::Display for InvoiceApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvoiceNotFound(id) => write!(formatter, "invoice {id} was not found"),
            Self::InvoiceAlreadyIssued(id) => write!(formatter, "invoice {id} is already issued"),
            Self::ServiceVisitNotInvoiceable => {
                write!(formatter, "service visit is not ready for invoicing")
            }
            Self::Validation(error) => write!(formatter, "invalid invoice: {error:?}"),
            Self::Database(error) => write!(formatter, "database operation failed: {error}"),
        }
    }
}

impl Error for InvoiceApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for InvoiceApplicationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub struct InvoiceApplicationService<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> InvoiceApplicationService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }

    pub fn list(
        &self,
        input: ListInvoicesInput,
    ) -> Result<Vec<InvoiceDirectoryEntry>, InvoiceApplicationError> {
        let query = input.query.trim();
        let filter = input
            .status_filter
            .unwrap_or(InvoiceDirectoryStatusFilter::All);
        let limit = input
            .limit
            .unwrap_or(DEFAULT_INVOICE_DIRECTORY_LIMIT)
            .min(MAX_INVOICE_DIRECTORY_LIMIT);
        InvoiceRepository::new(self.connection)
            .list(query, filter.into(), i64::from(limit))?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    pub fn load(&self, invoice_id: i64) -> Result<InvoiceDetails, InvoiceApplicationError> {
        InvoiceRepository::new(self.connection)
            .find_details_by_id(invoice_id)?
            .ok_or(InvoiceApplicationError::InvoiceNotFound(invoice_id))?
            .try_into()
    }

    pub fn load_for_service_visit(
        &self,
        service_visit_id: i64,
    ) -> Result<InvoiceDetails, InvoiceApplicationError> {
        InvoiceRepository::new(self.connection)
            .find_details_by_service_visit(service_visit_id)?
            .ok_or(InvoiceApplicationError::InvoiceNotFound(service_visit_id))?
            .try_into()
    }

    pub fn issue(
        &mut self,
        input: IssueInvoiceInput,
    ) -> Result<InvoiceDetails, InvoiceApplicationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let repository = InvoiceRepository::new(&transaction);
        let source = repository
            .find_issue_source(input.service_visit_id)?
            .ok_or(InvoiceApplicationError::InvoiceNotFound(
                input.service_visit_id,
            ))?;
        if source.invoice_status != "DRAFT" {
            return Err(InvoiceApplicationError::InvoiceAlreadyIssued(
                source.invoice_id,
            ));
        }
        let issue = InvoiceIssue::new(InvoiceIssueInput {
            invoice_id: source.invoice_id,
            service_visit_id: source.service_visit_id,
            service_visit_status: source.service_visit_status,
            completed_at: source.completed_at,
            issued_at: input.issued_at,
            labor_charge_fils: source.labor_charge_fils,
            active_part_line_totals_fils: source
                .lines
                .iter()
                .map(|line| line.line_total_fils)
                .collect(),
        })
        .map_err(|error| match error {
            InvoiceIssueError::ServiceVisitNotInvoiceable => {
                InvoiceApplicationError::ServiceVisitNotInvoiceable
            }
            other => InvoiceApplicationError::Validation(other),
        })?;
        for line in &source.lines {
            repository.insert_snapshot_line(source.invoice_id, line, input.issued_at)?;
        }
        repository.mark_issued(
            &source,
            issue.invoice_number(),
            issue.issued_at(),
            issue.parts_total_fils(),
            issue.total_fils(),
        )?;
        let details: InvoiceDetails = repository
            .find_details_by_id(source.invoice_id)?
            .ok_or(InvoiceApplicationError::InvoiceNotFound(source.invoice_id))?
            .try_into()?;
        transaction.commit()?;
        Ok(details)
    }
}

impl From<InvoiceDirectoryStatusFilter> for InvoiceStatusFilter {
    fn from(value: InvoiceDirectoryStatusFilter) -> Self {
        match value {
            InvoiceDirectoryStatusFilter::All => Self::All,
            InvoiceDirectoryStatusFilter::Draft => Self::Draft,
            InvoiceDirectoryStatusFilter::Issued => Self::Issued,
            InvoiceDirectoryStatusFilter::Cancelled => Self::Cancelled,
        }
    }
}

impl TryFrom<InvoiceDirectoryRow> for InvoiceDirectoryEntry {
    type Error = InvoiceApplicationError;
    fn try_from(row: InvoiceDirectoryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            service_visit_id: row.service_visit_id,
            status: parse_status(&row.status)?,
            invoice_number: row.invoice_number,
            issued_at: row.issued_at,
            customer_name: row.customer_name,
            customer_phone: row.customer_phone,
            motorcycle: row.motorcycle,
            plate_number: row.plate_number,
            total_fils: row.total_fils,
        })
    }
}

impl TryFrom<InvoiceDetailsRow> for InvoiceDetails {
    type Error = InvoiceApplicationError;
    fn try_from(row: InvoiceDetailsRow) -> Result<Self, Self::Error> {
        let header = row.header;
        Ok(Self {
            id: header.id,
            service_visit_id: header.service_visit_id,
            status: parse_status(&header.status)?,
            invoice_number: header.invoice_number,
            issued_at: header.issued_at,
            customer_name: header.customer_name,
            customer_phone: header.customer_phone,
            motorcycle_make_name: header.motorcycle_make_name,
            motorcycle_model: header.motorcycle_model,
            motorcycle_plate_number: header.motorcycle_plate_number,
            motorcycle_vin: header.motorcycle_vin,
            motorcycle_chassis_number: header.motorcycle_chassis_number,
            labor_charge_fils: header.labor_charge_fils,
            parts_total_fils: header.parts_total_fils,
            total_fils: header.total_fils,
            notes: header.notes,
            lines: row
                .lines
                .into_iter()
                .map(|line| InvoiceLine {
                    service_visit_part_id: line.service_visit_part_id,
                    item_name: line.item_name,
                    unit_name: line.unit_name,
                    quantity: line.quantity,
                    quantity_scale: line.quantity_scale,
                    unit_price_fils: line.unit_price_fils,
                    line_total_fils: line.line_total_fils,
                })
                .collect(),
        })
    }
}

fn parse_status(value: &str) -> Result<InvoiceStatus, InvoiceApplicationError> {
    match value {
        "DRAFT" => Ok(InvoiceStatus::Draft),
        "ISSUED" => Ok(InvoiceStatus::Issued),
        "CANCELLED" => Ok(InvoiceStatus::Cancelled),
        _ => Err(InvoiceApplicationError::Database(
            rusqlite::Error::InvalidQuery,
        )),
    }
}

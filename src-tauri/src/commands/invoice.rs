use serde::{Deserialize, Serialize};

use crate::{
    application::invoice::{
        InvoiceApplicationError, InvoiceApplicationService, InvoiceDetails, InvoiceDirectoryEntry,
        InvoiceDirectoryStatusFilter, InvoiceLine, InvoiceStatus, IssueInvoiceInput,
        ListInvoicesInput,
    },
    commands::service_visit_workspace::{CommandError, CommandErrorCategory, CommandResult},
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceDirectoryStatusFilterDto {
    All,
    Draft,
    Issued,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListInvoicesCommandInput {
    pub query: String,
    pub status_filter: Option<InvoiceDirectoryStatusFilterDto>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IssueInvoiceCommandInput {
    pub service_visit_id: i64,
    pub issued_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatusDto {
    Draft,
    Issued,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceDirectoryEntryDto {
    pub id: i64,
    pub service_visit_id: i64,
    pub status: InvoiceStatusDto,
    pub invoice_number: Option<String>,
    pub issued_at: Option<i64>,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle: String,
    pub plate_number: Option<String>,
    pub total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceDetailsDto {
    pub id: i64,
    pub service_visit_id: i64,
    pub status: InvoiceStatusDto,
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
    pub lines: Vec<InvoiceLineDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineDto {
    pub service_visit_part_id: i64,
    pub item_name: String,
    pub unit_name: String,
    pub quantity: i64,
    pub quantity_scale: i64,
    pub unit_price_fils: i64,
    pub line_total_fils: i64,
}

#[tauri::command]
pub fn list_invoices(
    database: tauri::State<'_, RuntimeDatabase>,
    input: ListInvoicesCommandInput,
) -> CommandResult<Vec<InvoiceDirectoryEntryDto>> {
    handle_list_invoices(&database, input)
}

#[tauri::command]
pub fn load_invoice_details(
    database: tauri::State<'_, RuntimeDatabase>,
    invoice_id: i64,
) -> CommandResult<InvoiceDetailsDto> {
    handle_load_invoice_details(&database, invoice_id)
}

#[tauri::command]
pub fn load_service_visit_invoice(
    database: tauri::State<'_, RuntimeDatabase>,
    service_visit_id: i64,
) -> CommandResult<InvoiceDetailsDto> {
    handle_load_service_visit_invoice(&database, service_visit_id)
}

#[tauri::command]
pub fn issue_invoice(
    database: tauri::State<'_, RuntimeDatabase>,
    input: IssueInvoiceCommandInput,
) -> CommandResult<InvoiceDetailsDto> {
    handle_issue_invoice(&database, input)
}

pub fn handle_list_invoices(
    database: &RuntimeDatabase,
    input: ListInvoicesCommandInput,
) -> CommandResult<Vec<InvoiceDirectoryEntryDto>> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InvoiceApplicationService::new(&mut connection)
        .list(input.into())
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

pub fn handle_load_invoice_details(
    database: &RuntimeDatabase,
    invoice_id: i64,
) -> CommandResult<InvoiceDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InvoiceApplicationService::new(&mut connection)
        .load(invoice_id)
        .map(Into::into)
        .map_err(Into::into)
}

pub fn handle_load_service_visit_invoice(
    database: &RuntimeDatabase,
    service_visit_id: i64,
) -> CommandResult<InvoiceDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InvoiceApplicationService::new(&mut connection)
        .load_for_service_visit(service_visit_id)
        .map(Into::into)
        .map_err(Into::into)
}

pub fn handle_issue_invoice(
    database: &RuntimeDatabase,
    input: IssueInvoiceCommandInput,
) -> CommandResult<InvoiceDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InvoiceApplicationService::new(&mut connection)
        .issue(input.into())
        .map(Into::into)
        .map_err(Into::into)
}

impl From<ListInvoicesCommandInput> for ListInvoicesInput {
    fn from(value: ListInvoicesCommandInput) -> Self {
        Self {
            query: value.query,
            status_filter: value.status_filter.map(Into::into),
            limit: value.limit,
        }
    }
}
impl From<InvoiceDirectoryStatusFilterDto> for InvoiceDirectoryStatusFilter {
    fn from(value: InvoiceDirectoryStatusFilterDto) -> Self {
        match value {
            InvoiceDirectoryStatusFilterDto::All => Self::All,
            InvoiceDirectoryStatusFilterDto::Draft => Self::Draft,
            InvoiceDirectoryStatusFilterDto::Issued => Self::Issued,
            InvoiceDirectoryStatusFilterDto::Cancelled => Self::Cancelled,
        }
    }
}
impl From<IssueInvoiceCommandInput> for IssueInvoiceInput {
    fn from(value: IssueInvoiceCommandInput) -> Self {
        Self {
            service_visit_id: value.service_visit_id,
            issued_at: value.issued_at,
        }
    }
}
impl From<InvoiceStatus> for InvoiceStatusDto {
    fn from(value: InvoiceStatus) -> Self {
        match value {
            InvoiceStatus::Draft => Self::Draft,
            InvoiceStatus::Issued => Self::Issued,
            InvoiceStatus::Cancelled => Self::Cancelled,
        }
    }
}
impl From<InvoiceDirectoryEntry> for InvoiceDirectoryEntryDto {
    fn from(value: InvoiceDirectoryEntry) -> Self {
        Self {
            id: value.id,
            service_visit_id: value.service_visit_id,
            status: value.status.into(),
            invoice_number: value.invoice_number,
            issued_at: value.issued_at,
            customer_name: value.customer_name,
            customer_phone: value.customer_phone,
            motorcycle: value.motorcycle,
            plate_number: value.plate_number,
            total_fils: value.total_fils,
        }
    }
}
impl From<InvoiceDetails> for InvoiceDetailsDto {
    fn from(value: InvoiceDetails) -> Self {
        Self {
            id: value.id,
            service_visit_id: value.service_visit_id,
            status: value.status.into(),
            invoice_number: value.invoice_number,
            issued_at: value.issued_at,
            customer_name: value.customer_name,
            customer_phone: value.customer_phone,
            motorcycle_make_name: value.motorcycle_make_name,
            motorcycle_model: value.motorcycle_model,
            motorcycle_plate_number: value.motorcycle_plate_number,
            motorcycle_vin: value.motorcycle_vin,
            motorcycle_chassis_number: value.motorcycle_chassis_number,
            labor_charge_fils: value.labor_charge_fils,
            parts_total_fils: value.parts_total_fils,
            total_fils: value.total_fils,
            notes: value.notes,
            lines: value.lines.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<InvoiceLine> for InvoiceLineDto {
    fn from(value: InvoiceLine) -> Self {
        Self {
            service_visit_part_id: value.service_visit_part_id,
            item_name: value.item_name,
            unit_name: value.unit_name,
            quantity: value.quantity,
            quantity_scale: value.quantity_scale,
            unit_price_fils: value.unit_price_fils,
            line_total_fils: value.line_total_fils,
        }
    }
}
impl From<InvoiceApplicationError> for CommandError {
    fn from(error: InvoiceApplicationError) -> Self {
        match error {
            InvoiceApplicationError::InvoiceNotFound(id) => Self {
                category: CommandErrorCategory::InvoiceNotFound,
                message: format!("Invoice {id} was not found."),
            },
            InvoiceApplicationError::InvoiceAlreadyIssued(id) => Self {
                category: CommandErrorCategory::InvoiceAlreadyIssued,
                message: format!("Invoice {id} has already been issued."),
            },
            InvoiceApplicationError::ServiceVisitNotInvoiceable => Self {
                category: CommandErrorCategory::ServiceVisitNotInvoiceable,
                message: "Only completed Service Visits can be invoiced.".into(),
            },
            InvoiceApplicationError::Validation(_) => Self {
                category: CommandErrorCategory::ValidationError,
                message: "The supplied Invoice data is invalid.".into(),
            },
            InvoiceApplicationError::Database(_) => CommandError::database(),
        }
    }
}

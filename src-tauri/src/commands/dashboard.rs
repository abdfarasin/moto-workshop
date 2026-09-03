use serde::{Deserialize, Serialize};

use crate::{
    application::dashboard::{
        Dashboard, DashboardApplicationError, DashboardApplicationService, DashboardDayRange,
        DashboardInventoryAlert, DashboardInvoice, DashboardServiceVisit, DashboardSummary,
    },
    commands::service_visit_workspace::{
        CommandError, CommandErrorCategory, CommandResult, ServiceVisitStatusDto,
    },
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoadDashboardCommandInput {
    pub day_start_ms: i64,
    pub day_end_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardDto {
    pub summary: DashboardSummaryDto,
    pub recent_service_visits: Vec<DashboardServiceVisitDto>,
    pub recent_invoices: Vec<DashboardInvoiceDto>,
    pub inventory_alerts: Vec<DashboardInventoryAlertDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummaryDto {
    pub active_service_visits: i64,
    pub ready_for_pickup_visits: i64,
    pub customer_count: i64,
    pub motorcycle_count: i64,
    pub low_stock_item_count: i64,
    pub negative_stock_item_count: i64,
    pub issued_invoice_count_today: i64,
    pub issued_invoice_value_today_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardServiceVisitDto {
    pub id: i64,
    pub customer_name: String,
    pub motorcycle: String,
    pub plate_number: Option<String>,
    pub opened_at: i64,
    pub status: ServiceVisitStatusDto,
    pub complaint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardInvoiceDto {
    pub id: i64,
    pub invoice_number: String,
    pub issued_at: i64,
    pub customer_name: String,
    pub motorcycle: String,
    pub total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardInventoryAlertDto {
    pub id: i64,
    pub item_name: String,
    pub sku: Option<String>,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub current_quantity: i64,
    pub minimum_stock_quantity: i64,
    pub negative_stock: bool,
}

#[tauri::command]
pub fn load_dashboard(
    database: tauri::State<'_, RuntimeDatabase>,
    input: LoadDashboardCommandInput,
) -> CommandResult<DashboardDto> {
    handle_load_dashboard(&database, input)
}

pub fn handle_load_dashboard(
    database: &RuntimeDatabase,
    input: LoadDashboardCommandInput,
) -> CommandResult<DashboardDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    DashboardApplicationService::new(&mut connection)
        .load(input.into())
        .map(Into::into)
        .map_err(Into::into)
}

impl From<LoadDashboardCommandInput> for DashboardDayRange {
    fn from(input: LoadDashboardCommandInput) -> Self {
        Self {
            start_ms: input.day_start_ms,
            end_ms: input.day_end_ms,
        }
    }
}

impl From<Dashboard> for DashboardDto {
    fn from(value: Dashboard) -> Self {
        Self {
            summary: value.summary.into(),
            recent_service_visits: value
                .recent_service_visits
                .into_iter()
                .map(Into::into)
                .collect(),
            recent_invoices: value.recent_invoices.into_iter().map(Into::into).collect(),
            inventory_alerts: value.inventory_alerts.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<DashboardSummary> for DashboardSummaryDto {
    fn from(value: DashboardSummary) -> Self {
        Self {
            active_service_visits: value.active_service_visits,
            ready_for_pickup_visits: value.ready_for_pickup_visits,
            customer_count: value.customer_count,
            motorcycle_count: value.motorcycle_count,
            low_stock_item_count: value.low_stock_item_count,
            negative_stock_item_count: value.negative_stock_item_count,
            issued_invoice_count_today: value.issued_invoice_count_today,
            issued_invoice_value_today_fils: value.issued_invoice_value_today_fils,
        }
    }
}
impl From<DashboardServiceVisit> for DashboardServiceVisitDto {
    fn from(value: DashboardServiceVisit) -> Self {
        Self {
            id: value.id,
            customer_name: value.customer_name,
            motorcycle: value.motorcycle,
            plate_number: value.plate_number,
            opened_at: value.opened_at,
            status: value.status.into(),
            complaint: value.complaint,
        }
    }
}
impl From<DashboardInvoice> for DashboardInvoiceDto {
    fn from(value: DashboardInvoice) -> Self {
        Self {
            id: value.id,
            invoice_number: value.invoice_number,
            issued_at: value.issued_at,
            customer_name: value.customer_name,
            motorcycle: value.motorcycle,
            total_fils: value.total_fils,
        }
    }
}
impl From<DashboardInventoryAlert> for DashboardInventoryAlertDto {
    fn from(value: DashboardInventoryAlert) -> Self {
        Self {
            id: value.id,
            item_name: value.item_name,
            sku: value.sku,
            unit_name: value.unit_name,
            quantity_scale: value.quantity_scale,
            current_quantity: value.current_quantity,
            minimum_stock_quantity: value.minimum_stock_quantity,
            negative_stock: value.negative_stock,
        }
    }
}
impl From<DashboardApplicationError> for CommandError {
    fn from(error: DashboardApplicationError) -> Self {
        match error {
            DashboardApplicationError::InvalidDayRange => Self {
                category: CommandErrorCategory::ValidationError,
                message: "The supplied local day range is invalid.".into(),
            },
            DashboardApplicationError::Database(_) => Self::database(),
        }
    }
}

use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::{
    domain::service_visit::ServiceVisitStatus,
    repositories::dashboard::{
        DashboardInventoryAlertRow, DashboardInvoiceRow, DashboardRepository,
        DashboardServiceVisitRow, DashboardSummaryRow,
    },
};

const DASHBOARD_LIST_LIMIT: i64 = 5;
const MAX_LOCAL_DAY_MILLISECONDS: i64 = 26 * 60 * 60 * 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardDayRange {
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dashboard {
    pub summary: DashboardSummary,
    pub recent_service_visits: Vec<DashboardServiceVisit>,
    pub recent_invoices: Vec<DashboardInvoice>,
    pub inventory_alerts: Vec<DashboardInventoryAlert>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSummary {
    pub active_service_visits: i64,
    pub ready_for_pickup_visits: i64,
    pub customer_count: i64,
    pub motorcycle_count: i64,
    pub low_stock_item_count: i64,
    pub negative_stock_item_count: i64,
    pub issued_invoice_count_today: i64,
    pub issued_invoice_value_today_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardServiceVisit {
    pub id: i64,
    pub customer_name: String,
    pub motorcycle: String,
    pub plate_number: Option<String>,
    pub opened_at: i64,
    pub status: ServiceVisitStatus,
    pub complaint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardInvoice {
    pub id: i64,
    pub invoice_number: String,
    pub issued_at: i64,
    pub customer_name: String,
    pub motorcycle: String,
    pub total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardInventoryAlert {
    pub id: i64,
    pub item_name: String,
    pub sku: Option<String>,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub current_quantity: i64,
    pub minimum_stock_quantity: i64,
    pub negative_stock: bool,
}

#[derive(Debug)]
pub enum DashboardApplicationError {
    InvalidDayRange,
    Database(rusqlite::Error),
}

impl fmt::Display for DashboardApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDayRange => write!(formatter, "invalid local calendar day range"),
            Self::Database(error) => write!(formatter, "database operation failed: {error}"),
        }
    }
}

impl Error for DashboardApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::InvalidDayRange => None,
        }
    }
}

impl From<rusqlite::Error> for DashboardApplicationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub struct DashboardApplicationService<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> DashboardApplicationService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }

    pub fn load(&self, day: DashboardDayRange) -> Result<Dashboard, DashboardApplicationError> {
        let duration = day.end_ms.checked_sub(day.start_ms);
        if day.start_ms < 0
            || !matches!(duration, Some(value) if value > 0 && value <= MAX_LOCAL_DAY_MILLISECONDS)
        {
            return Err(DashboardApplicationError::InvalidDayRange);
        }
        let repository = DashboardRepository::new(self.connection);
        Ok(Dashboard {
            summary: repository.load_summary(day.start_ms, day.end_ms)?.into(),
            recent_service_visits: repository
                .list_recent_service_visits(DASHBOARD_LIST_LIMIT)?
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            recent_invoices: repository
                .list_recent_issued_invoices(DASHBOARD_LIST_LIMIT)?
                .into_iter()
                .map(Into::into)
                .collect(),
            inventory_alerts: repository
                .list_inventory_alerts(DASHBOARD_LIST_LIMIT)?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl From<DashboardSummaryRow> for DashboardSummary {
    fn from(row: DashboardSummaryRow) -> Self {
        Self {
            active_service_visits: row.active_service_visits,
            ready_for_pickup_visits: row.ready_for_pickup_visits,
            customer_count: row.customer_count,
            motorcycle_count: row.motorcycle_count,
            low_stock_item_count: row.low_stock_item_count,
            negative_stock_item_count: row.negative_stock_item_count,
            issued_invoice_count_today: row.issued_invoice_count_today,
            issued_invoice_value_today_fils: row.issued_invoice_value_today_fils,
        }
    }
}

impl TryFrom<DashboardServiceVisitRow> for DashboardServiceVisit {
    type Error = DashboardApplicationError;
    fn try_from(row: DashboardServiceVisitRow) -> Result<Self, Self::Error> {
        let status = match row.status.as_str() {
            "OPEN" => ServiceVisitStatus::Open,
            "READY_FOR_PICKUP" => ServiceVisitStatus::ReadyForPickup,
            "CLOSED" => ServiceVisitStatus::Closed,
            "CANCELLED" => ServiceVisitStatus::Cancelled,
            _ => {
                return Err(DashboardApplicationError::Database(
                    rusqlite::Error::InvalidQuery,
                ))
            }
        };
        Ok(Self {
            id: row.id,
            customer_name: row.customer_name,
            motorcycle: row.motorcycle,
            plate_number: row.plate_number,
            opened_at: row.opened_at,
            status,
            complaint: row.complaint,
        })
    }
}

impl From<DashboardInvoiceRow> for DashboardInvoice {
    fn from(row: DashboardInvoiceRow) -> Self {
        Self {
            id: row.id,
            invoice_number: row.invoice_number,
            issued_at: row.issued_at,
            customer_name: row.customer_name,
            motorcycle: row.motorcycle,
            total_fils: row.total_fils,
        }
    }
}

impl From<DashboardInventoryAlertRow> for DashboardInventoryAlert {
    fn from(row: DashboardInventoryAlertRow) -> Self {
        Self {
            id: row.id,
            item_name: row.item_name,
            sku: row.sku,
            unit_name: row.unit_name,
            quantity_scale: row.quantity_scale,
            current_quantity: row.current_quantity,
            minimum_stock_quantity: row.minimum_stock_quantity,
            negative_stock: row.current_quantity < 0,
        }
    }
}

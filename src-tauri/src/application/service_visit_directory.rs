use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::{
    domain::service_visit::ServiceVisitStatus,
    repositories::service_visit_directory::{
        ServiceVisitDirectoryFilter, ServiceVisitDirectoryRepository, ServiceVisitDirectoryRow,
    },
};

pub const DEFAULT_SERVICE_VISIT_DIRECTORY_LIMIT: u32 = 50;
pub const MAX_SERVICE_VISIT_DIRECTORY_LIMIT: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVisitDirectoryStatusFilter {
    Active,
    All,
    Open,
    ReadyForPickup,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListServiceVisitsInput {
    pub query: String,
    pub status_filter: Option<ServiceVisitDirectoryStatusFilter>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVisitDirectoryEntry {
    pub id: i64,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle_id: i64,
    pub make_name: String,
    pub model: String,
    pub plate_number: Option<String>,
    pub opened_at: i64,
    pub customer_complaint: String,
    pub status: ServiceVisitStatus,
    pub total_fils: i64,
}

#[derive(Debug)]
pub struct ServiceVisitDirectoryApplicationError(rusqlite::Error);

impl fmt::Display for ServiceVisitDirectoryApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "database operation failed: {}", self.0)
    }
}

impl Error for ServiceVisitDirectoryApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<rusqlite::Error> for ServiceVisitDirectoryApplicationError {
    fn from(error: rusqlite::Error) -> Self {
        Self(error)
    }
}

pub struct ServiceVisitDirectoryApplicationService<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> ServiceVisitDirectoryApplicationService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }

    pub fn list(
        &self,
        input: ListServiceVisitsInput,
    ) -> Result<Vec<ServiceVisitDirectoryEntry>, ServiceVisitDirectoryApplicationError> {
        let query = input.query.trim();
        let filter = input
            .status_filter
            .unwrap_or(ServiceVisitDirectoryStatusFilter::Active);
        let limit = input
            .limit
            .unwrap_or(DEFAULT_SERVICE_VISIT_DIRECTORY_LIMIT)
            .min(MAX_SERVICE_VISIT_DIRECTORY_LIMIT);

        Ok(ServiceVisitDirectoryRepository::new(self.connection)
            .list(query, filter.into(), i64::from(limit))?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

impl From<ServiceVisitDirectoryStatusFilter> for ServiceVisitDirectoryFilter {
    fn from(filter: ServiceVisitDirectoryStatusFilter) -> Self {
        match filter {
            ServiceVisitDirectoryStatusFilter::Active => Self::Active,
            ServiceVisitDirectoryStatusFilter::All => Self::All,
            ServiceVisitDirectoryStatusFilter::Open => Self::Open,
            ServiceVisitDirectoryStatusFilter::ReadyForPickup => Self::ReadyForPickup,
            ServiceVisitDirectoryStatusFilter::Closed => Self::Closed,
            ServiceVisitDirectoryStatusFilter::Cancelled => Self::Cancelled,
        }
    }
}

impl From<ServiceVisitDirectoryRow> for ServiceVisitDirectoryEntry {
    fn from(row: ServiceVisitDirectoryRow) -> Self {
        Self {
            id: row.id,
            customer_name: row.customer_name,
            customer_phone: row.customer_phone,
            motorcycle_id: row.motorcycle_id,
            make_name: row.make_name,
            model: row.model,
            plate_number: row.plate_number,
            opened_at: row.opened_at,
            customer_complaint: row.customer_complaint,
            status: row.status,
            total_fils: row.total_fils,
        }
    }
}

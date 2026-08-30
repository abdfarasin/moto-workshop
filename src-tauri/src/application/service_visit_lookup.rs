use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::repositories::service_visit_lookup::{
    ActiveServiceVisitStatusRow, CustomerMotorcycleLookupRow, CustomerSummaryRow,
    ServiceVisitLookupRepository,
};

pub const DEFAULT_CUSTOMER_SEARCH_LIMIT: u32 = 25;
pub const MAX_CUSTOMER_SEARCH_LIMIT: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCustomersInput {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerSummary {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveServiceVisitStatus {
    Open,
    ReadyForPickup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerMotorcycleLookup {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub color_name: String,
    pub plate_code: Option<String>,
    pub plate_number: Option<i64>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub active_service_visit_id: Option<i64>,
    pub active_service_visit_status: Option<ActiveServiceVisitStatus>,
}

#[derive(Debug)]
pub enum ServiceVisitLookupError {
    CustomerNotFound(i64),
    Database(rusqlite::Error),
}

impl fmt::Display for ServiceVisitLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomerNotFound(id) => write!(formatter, "customer {id} was not found"),
            Self::Database(error) => write!(formatter, "database operation failed: {error}"),
        }
    }
}

impl Error for ServiceVisitLookupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::CustomerNotFound(_) => None,
        }
    }
}

impl From<rusqlite::Error> for ServiceVisitLookupError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub struct ServiceVisitLookupService<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ServiceVisitLookupService<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn search_customers(
        &self,
        input: SearchCustomersInput,
    ) -> Result<Vec<CustomerSummary>, ServiceVisitLookupError> {
        let limit = input
            .limit
            .unwrap_or(DEFAULT_CUSTOMER_SEARCH_LIMIT)
            .min(MAX_CUSTOMER_SEARCH_LIMIT);
        Ok(ServiceVisitLookupRepository::new(self.connection)
            .search_customers(input.query.trim(), i64::from(limit))?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn list_customer_motorcycles(
        &self,
        customer_id: i64,
    ) -> Result<Vec<CustomerMotorcycleLookup>, ServiceVisitLookupError> {
        let repository = ServiceVisitLookupRepository::new(self.connection);
        if !repository.customer_exists(customer_id)? {
            return Err(ServiceVisitLookupError::CustomerNotFound(customer_id));
        }
        Ok(repository
            .list_customer_motorcycles(customer_id)?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

impl From<CustomerSummaryRow> for CustomerSummary {
    fn from(row: CustomerSummaryRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            phone: row.phone,
        }
    }
}

impl From<CustomerMotorcycleLookupRow> for CustomerMotorcycleLookup {
    fn from(row: CustomerMotorcycleLookupRow) -> Self {
        Self {
            id: row.id,
            make_name: row.make_name,
            model: row.model,
            year: row.year,
            color_name: row.color_name,
            plate_code: row.plate_code,
            plate_number: row.plate_number,
            vin: row.vin,
            chassis_number: row.chassis_number,
            active_service_visit_id: row.active_service_visit_id,
            active_service_visit_status: row.active_service_visit_status.map(Into::into),
        }
    }
}

impl From<ActiveServiceVisitStatusRow> for ActiveServiceVisitStatus {
    fn from(status: ActiveServiceVisitStatusRow) -> Self {
        match status {
            ActiveServiceVisitStatusRow::Open => Self::Open,
            ActiveServiceVisitStatusRow::ReadyForPickup => Self::ReadyForPickup,
        }
    }
}

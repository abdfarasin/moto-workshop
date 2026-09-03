use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::{
    domain::service_visit::ServiceVisitStatus,
    repositories::motorcycle_directory::{
        MotorcycleDirectoryRepository, MotorcycleDirectoryRow, MotorcycleServiceHistoryRow,
    },
};

pub const DEFAULT_MOTORCYCLE_DIRECTORY_LIMIT: u32 = 50;
pub const MAX_MOTORCYCLE_DIRECTORY_LIMIT: u32 = 100;
const MOTORCYCLE_HISTORY_LIMIT: i64 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMotorcycleDirectoryInput {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadMotorcycleDetailsInput {
    pub motorcycle_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleDirectoryEntry {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub color_name: String,
    pub plate_number: Option<String>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub owner_customer_id: i64,
    pub owner_name: String,
    pub owner_phone: String,
    pub latest_service_visit_at: Option<i64>,
    pub active_service_visit_id: Option<i64>,
    pub active_service_visit_status: Option<ServiceVisitStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleDetails {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub color_name: String,
    pub plate_number: Option<String>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub owner_customer_id: i64,
    pub owner_name: String,
    pub owner_phone: String,
    pub active_service_visit_id: Option<i64>,
    pub active_service_visit_status: Option<ServiceVisitStatus>,
    pub service_history: Vec<MotorcycleServiceHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleServiceHistoryEntry {
    pub id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub status: ServiceVisitStatus,
    pub total_fils: i64,
}

#[derive(Debug)]
pub struct MotorcycleDirectoryApplicationError(rusqlite::Error);
impl fmt::Display for MotorcycleDirectoryApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "database operation failed: {}", self.0)
    }
}
impl Error for MotorcycleDirectoryApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}
impl From<rusqlite::Error> for MotorcycleDirectoryApplicationError {
    fn from(value: rusqlite::Error) -> Self {
        Self(value)
    }
}

pub struct MotorcycleDirectoryApplicationService<'connection> {
    connection: &'connection mut Connection,
}
impl<'connection> MotorcycleDirectoryApplicationService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }
    pub fn search(
        &self,
        input: SearchMotorcycleDirectoryInput,
    ) -> Result<Vec<MotorcycleDirectoryEntry>, MotorcycleDirectoryApplicationError> {
        let limit = input
            .limit
            .unwrap_or(DEFAULT_MOTORCYCLE_DIRECTORY_LIMIT)
            .min(MAX_MOTORCYCLE_DIRECTORY_LIMIT);
        Ok(MotorcycleDirectoryRepository::new(self.connection)
            .search(input.query.trim(), i64::from(limit))?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub fn load(
        &self,
        input: LoadMotorcycleDetailsInput,
    ) -> Result<Option<MotorcycleDetails>, MotorcycleDirectoryApplicationError> {
        let repository = MotorcycleDirectoryRepository::new(self.connection);
        let Some(row) = repository.find(input.motorcycle_id)? else {
            return Ok(None);
        };
        let history = repository
            .list_service_history(input.motorcycle_id, MOTORCYCLE_HISTORY_LIMIT)?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Some(MotorcycleDetails {
            id: row.id,
            make_name: row.make_name,
            model: row.model,
            year: row.year,
            color_name: row.color_name,
            plate_number: row.plate_number,
            vin: row.vin,
            chassis_number: row.chassis_number,
            owner_customer_id: row.owner_customer_id,
            owner_name: row.owner_name,
            owner_phone: row.owner_phone,
            active_service_visit_id: row.active_service_visit_id,
            active_service_visit_status: row.active_service_visit_status,
            service_history: history,
        }))
    }
}
impl From<MotorcycleDirectoryRow> for MotorcycleDirectoryEntry {
    fn from(r: MotorcycleDirectoryRow) -> Self {
        Self {
            id: r.id,
            make_name: r.make_name,
            model: r.model,
            year: r.year,
            color_name: r.color_name,
            plate_number: r.plate_number,
            vin: r.vin,
            chassis_number: r.chassis_number,
            owner_customer_id: r.owner_customer_id,
            owner_name: r.owner_name,
            owner_phone: r.owner_phone,
            latest_service_visit_at: r.latest_service_visit_at,
            active_service_visit_id: r.active_service_visit_id,
            active_service_visit_status: r.active_service_visit_status,
        }
    }
}
impl From<MotorcycleServiceHistoryRow> for MotorcycleServiceHistoryEntry {
    fn from(r: MotorcycleServiceHistoryRow) -> Self {
        Self {
            id: r.id,
            opened_at: r.opened_at,
            odometer_km: r.odometer_km,
            customer_complaint: r.customer_complaint,
            status: r.status,
            total_fils: r.total_fils,
        }
    }
}

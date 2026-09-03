use serde::{Deserialize, Serialize};

use crate::{
    application::motorcycle_directory::{
        LoadMotorcycleDetailsInput, MotorcycleDetails, MotorcycleDirectoryApplicationService,
        MotorcycleDirectoryEntry, MotorcycleServiceHistoryEntry, SearchMotorcycleDirectoryInput,
    },
    commands::service_visit_workspace::{
        CommandError, CommandErrorCategory, CommandResult, ServiceVisitStatusDto,
    },
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchMotorcycleDirectoryCommandInput {
    pub query: String,
    pub limit: Option<u32>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoadMotorcycleDetailsCommandInput {
    pub motorcycle_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorcycleDirectoryEntryDto {
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
    pub active_service_visit_status: Option<ServiceVisitStatusDto>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorcycleDetailsDto {
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
    pub active_service_visit_status: Option<ServiceVisitStatusDto>,
    pub service_history: Vec<MotorcycleServiceHistoryEntryDto>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorcycleServiceHistoryEntryDto {
    pub id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub status: ServiceVisitStatusDto,
    pub total_fils: i64,
}

#[tauri::command]
pub fn search_motorcycle_directory(
    database: tauri::State<'_, RuntimeDatabase>,
    input: SearchMotorcycleDirectoryCommandInput,
) -> CommandResult<Vec<MotorcycleDirectoryEntryDto>> {
    handle_search_motorcycle_directory(&database, input)
}
pub fn handle_search_motorcycle_directory(
    database: &RuntimeDatabase,
    input: SearchMotorcycleDirectoryCommandInput,
) -> CommandResult<Vec<MotorcycleDirectoryEntryDto>> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    MotorcycleDirectoryApplicationService::new(&mut connection)
        .search(input.into())
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|_| CommandError::database())
}
#[tauri::command]
pub fn load_motorcycle_details(
    database: tauri::State<'_, RuntimeDatabase>,
    input: LoadMotorcycleDetailsCommandInput,
) -> CommandResult<MotorcycleDetailsDto> {
    handle_load_motorcycle_details(&database, input)
}
pub fn handle_load_motorcycle_details(
    database: &RuntimeDatabase,
    input: LoadMotorcycleDetailsCommandInput,
) -> CommandResult<MotorcycleDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    match MotorcycleDirectoryApplicationService::new(&mut connection).load(input.into()) {
        Ok(Some(details)) => Ok(details.into()),
        Ok(None) => Err(CommandError {
            category: CommandErrorCategory::MotorcycleNotFound,
            message: "The Motorcycle could not be found.".into(),
        }),
        Err(_) => Err(CommandError::database()),
    }
}
impl From<SearchMotorcycleDirectoryCommandInput> for SearchMotorcycleDirectoryInput {
    fn from(i: SearchMotorcycleDirectoryCommandInput) -> Self {
        Self {
            query: i.query,
            limit: i.limit,
        }
    }
}
impl From<LoadMotorcycleDetailsCommandInput> for LoadMotorcycleDetailsInput {
    fn from(i: LoadMotorcycleDetailsCommandInput) -> Self {
        Self {
            motorcycle_id: i.motorcycle_id,
        }
    }
}
impl From<MotorcycleDirectoryEntry> for MotorcycleDirectoryEntryDto {
    fn from(r: MotorcycleDirectoryEntry) -> Self {
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
            active_service_visit_status: r.active_service_visit_status.map(Into::into),
        }
    }
}
impl From<MotorcycleDetails> for MotorcycleDetailsDto {
    fn from(r: MotorcycleDetails) -> Self {
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
            active_service_visit_id: r.active_service_visit_id,
            active_service_visit_status: r.active_service_visit_status.map(Into::into),
            service_history: r.service_history.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<MotorcycleServiceHistoryEntry> for MotorcycleServiceHistoryEntryDto {
    fn from(r: MotorcycleServiceHistoryEntry) -> Self {
        Self {
            id: r.id,
            opened_at: r.opened_at,
            odometer_km: r.odometer_km,
            customer_complaint: r.customer_complaint,
            status: r.status.into(),
            total_fils: r.total_fils,
        }
    }
}

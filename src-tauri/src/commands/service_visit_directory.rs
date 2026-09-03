use serde::{Deserialize, Serialize};

use crate::{
    application::service_visit_directory::{
        ListServiceVisitsInput, ServiceVisitDirectoryApplicationService,
        ServiceVisitDirectoryEntry, ServiceVisitDirectoryStatusFilter,
    },
    commands::service_visit_workspace::{CommandError, CommandResult, ServiceVisitStatusDto},
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceVisitDirectoryStatusFilterDto {
    Active,
    All,
    Open,
    ReadyForPickup,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListServiceVisitsCommandInput {
    pub query: String,
    pub status_filter: Option<ServiceVisitDirectoryStatusFilterDto>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceVisitDirectoryEntryDto {
    pub id: i64,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle_id: i64,
    pub make_name: String,
    pub model: String,
    pub plate_number: Option<String>,
    pub opened_at: i64,
    pub customer_complaint: String,
    pub status: ServiceVisitStatusDto,
    pub total_fils: i64,
}

#[tauri::command]
pub fn list_service_visits(
    database: tauri::State<'_, RuntimeDatabase>,
    input: ListServiceVisitsCommandInput,
) -> CommandResult<Vec<ServiceVisitDirectoryEntryDto>> {
    handle_list_service_visits(&database, input)
}

pub fn handle_list_service_visits(
    database: &RuntimeDatabase,
    input: ListServiceVisitsCommandInput,
) -> CommandResult<Vec<ServiceVisitDirectoryEntryDto>> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitDirectoryApplicationService::new(&mut connection).list(input.into())
    };

    result
        .map(|visits| visits.into_iter().map(Into::into).collect())
        .map_err(|_| CommandError::database())
}

impl From<ListServiceVisitsCommandInput> for ListServiceVisitsInput {
    fn from(input: ListServiceVisitsCommandInput) -> Self {
        Self {
            query: input.query,
            status_filter: input.status_filter.map(Into::into),
            limit: input.limit,
        }
    }
}

impl From<ServiceVisitDirectoryStatusFilterDto> for ServiceVisitDirectoryStatusFilter {
    fn from(filter: ServiceVisitDirectoryStatusFilterDto) -> Self {
        match filter {
            ServiceVisitDirectoryStatusFilterDto::Active => Self::Active,
            ServiceVisitDirectoryStatusFilterDto::All => Self::All,
            ServiceVisitDirectoryStatusFilterDto::Open => Self::Open,
            ServiceVisitDirectoryStatusFilterDto::ReadyForPickup => Self::ReadyForPickup,
            ServiceVisitDirectoryStatusFilterDto::Closed => Self::Closed,
            ServiceVisitDirectoryStatusFilterDto::Cancelled => Self::Cancelled,
        }
    }
}

impl From<ServiceVisitDirectoryEntry> for ServiceVisitDirectoryEntryDto {
    fn from(visit: ServiceVisitDirectoryEntry) -> Self {
        Self {
            id: visit.id,
            customer_name: visit.customer_name,
            customer_phone: visit.customer_phone,
            motorcycle_id: visit.motorcycle_id,
            make_name: visit.make_name,
            model: visit.model,
            plate_number: visit.plate_number,
            opened_at: visit.opened_at,
            customer_complaint: visit.customer_complaint,
            status: visit.status.into(),
            total_fils: visit.total_fils,
        }
    }
}

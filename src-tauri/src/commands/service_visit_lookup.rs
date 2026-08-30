use serde::{Deserialize, Serialize};

use crate::{
    application::service_visit_lookup::{
        ActiveServiceVisitStatus, CustomerMotorcycleLookup, CustomerSummary, SearchCustomersInput,
        ServiceVisitLookupError, ServiceVisitLookupService,
    },
    commands::service_visit_workspace::{CommandError, CommandErrorCategory, CommandResult},
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchCustomersCommandInput {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListCustomerMotorcyclesCommandInput {
    pub customer_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerSummaryDto {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActiveServiceVisitStatusDto {
    Open,
    ReadyForPickup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerMotorcycleLookupDto {
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
    pub active_service_visit_status: Option<ActiveServiceVisitStatusDto>,
}

#[tauri::command]
pub fn search_customers(
    database: tauri::State<'_, RuntimeDatabase>,
    input: SearchCustomersCommandInput,
) -> CommandResult<Vec<CustomerSummaryDto>> {
    handle_search_customers(&database, input)
}

#[tauri::command]
pub fn list_customer_motorcycles(
    database: tauri::State<'_, RuntimeDatabase>,
    input: ListCustomerMotorcyclesCommandInput,
) -> CommandResult<Vec<CustomerMotorcycleLookupDto>> {
    handle_list_customer_motorcycles(&database, input)
}

pub fn handle_search_customers(
    database: &RuntimeDatabase,
    input: SearchCustomersCommandInput,
) -> CommandResult<Vec<CustomerSummaryDto>> {
    let result = {
        let connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitLookupService::new(&connection).search_customers(input.into())
    };
    result
        .map(|customers| customers.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

pub fn handle_list_customer_motorcycles(
    database: &RuntimeDatabase,
    input: ListCustomerMotorcyclesCommandInput,
) -> CommandResult<Vec<CustomerMotorcycleLookupDto>> {
    let result = {
        let connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitLookupService::new(&connection).list_customer_motorcycles(input.customer_id)
    };
    result
        .map(|motorcycles| motorcycles.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

impl From<SearchCustomersCommandInput> for SearchCustomersInput {
    fn from(input: SearchCustomersCommandInput) -> Self {
        Self {
            query: input.query,
            limit: input.limit,
        }
    }
}

impl From<CustomerSummary> for CustomerSummaryDto {
    fn from(customer: CustomerSummary) -> Self {
        Self {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
        }
    }
}

impl From<CustomerMotorcycleLookup> for CustomerMotorcycleLookupDto {
    fn from(motorcycle: CustomerMotorcycleLookup) -> Self {
        Self {
            id: motorcycle.id,
            make_name: motorcycle.make_name,
            model: motorcycle.model,
            year: motorcycle.year,
            color_name: motorcycle.color_name,
            plate_code: motorcycle.plate_code,
            plate_number: motorcycle.plate_number,
            vin: motorcycle.vin,
            chassis_number: motorcycle.chassis_number,
            active_service_visit_id: motorcycle.active_service_visit_id,
            active_service_visit_status: motorcycle.active_service_visit_status.map(Into::into),
        }
    }
}

impl From<ActiveServiceVisitStatus> for ActiveServiceVisitStatusDto {
    fn from(status: ActiveServiceVisitStatus) -> Self {
        match status {
            ActiveServiceVisitStatus::Open => Self::Open,
            ActiveServiceVisitStatus::ReadyForPickup => Self::ReadyForPickup,
        }
    }
}

impl From<ServiceVisitLookupError> for CommandError {
    fn from(error: ServiceVisitLookupError) -> Self {
        match error {
            ServiceVisitLookupError::CustomerNotFound(_) => Self {
                category: CommandErrorCategory::CustomerNotFound,
                message: "The Customer was not found.".into(),
            },
            ServiceVisitLookupError::Database(_) => Self::database(),
        }
    }
}

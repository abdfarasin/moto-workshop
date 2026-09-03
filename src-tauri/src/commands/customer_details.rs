use serde::{Deserialize, Serialize};

use crate::{
    application::customer_details::{
        CustomerDetails, CustomerDetailsApplicationError, CustomerDetailsApplicationService,
        CustomerDetailsMotorcycle, CustomerServiceHistoryEntry, LoadCustomerDetailsInput,
    },
    commands::service_visit_workspace::{CommandError, CommandErrorCategory, CommandResult},
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoadCustomerDetailsCommandInput {
    pub customer_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerDetailsDto {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub motorcycles: Vec<CustomerDetailsMotorcycleDto>,
    pub service_history: Vec<CustomerServiceHistoryEntryDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerDetailsMotorcycleDto {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub plate_number: Option<String>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub color_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerServiceHistoryEntryDto {
    pub id: i64,
    pub motorcycle_id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub status: String,
    pub total_fils: i64,
}

#[tauri::command]
pub fn load_customer_details(
    database: tauri::State<'_, RuntimeDatabase>,
    input: LoadCustomerDetailsCommandInput,
) -> CommandResult<CustomerDetailsDto> {
    handle_load_customer_details(&database, input)
}

pub fn handle_load_customer_details(
    database: &RuntimeDatabase,
    input: LoadCustomerDetailsCommandInput,
) -> CommandResult<CustomerDetailsDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;

        CustomerDetailsApplicationService::new(&mut connection).load(input.into())
    };

    match result {
        Ok(Some(details)) => Ok(details.into()),

        Ok(None) => Err(CommandError {
            category: CommandErrorCategory::CustomerNotFound,
            message: "The Customer could not be found.".into(),
        }),

        Err(CustomerDetailsApplicationError::Database(_)) => Err(CommandError::database()),
    }
}

impl From<LoadCustomerDetailsCommandInput> for LoadCustomerDetailsInput {
    fn from(input: LoadCustomerDetailsCommandInput) -> Self {
        Self {
            customer_id: input.customer_id,
        }
    }
}

impl From<CustomerDetails> for CustomerDetailsDto {
    fn from(details: CustomerDetails) -> Self {
        Self {
            id: details.id,
            name: details.name,
            phone: details.phone,
            motorcycles: details.motorcycles.into_iter().map(Into::into).collect(),
            service_history: details
                .service_history
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<CustomerDetailsMotorcycle> for CustomerDetailsMotorcycleDto {
    fn from(motorcycle: CustomerDetailsMotorcycle) -> Self {
        Self {
            id: motorcycle.id,
            make_name: motorcycle.make_name,
            model: motorcycle.model,
            year: motorcycle.year,
            plate_number: motorcycle.plate_number,
            vin: motorcycle.vin,
            chassis_number: motorcycle.chassis_number,
            color_name: motorcycle.color_name,
        }
    }
}

impl From<CustomerServiceHistoryEntry> for CustomerServiceHistoryEntryDto {
    fn from(visit: CustomerServiceHistoryEntry) -> Self {
        Self {
            id: visit.id,
            motorcycle_id: visit.motorcycle_id,
            opened_at: visit.opened_at,
            odometer_km: visit.odometer_km,
            customer_complaint: visit.customer_complaint,
            status: visit.status,
            total_fils: visit.total_fils,
        }
    }
}

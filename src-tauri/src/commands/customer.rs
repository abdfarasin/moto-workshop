use serde::{Deserialize, Serialize};

use crate::{
    application::customer::{
        CreateCustomerInput, CustomerApplicationError, CustomerApplicationService,
        CustomerDirectoryEntry, CustomerSummary, SearchCustomerDirectoryInput,
    },
    commands::service_visit_workspace::{CommandError, CommandErrorCategory, CommandResult},
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCustomerCommandInput {
    pub name: String,
    pub phone: String,
    pub notes: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchCustomerDirectoryCommandInput {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerSummaryDto {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerDirectoryEntryDto {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub motorcycle_count: i64,
    pub last_visit_at: Option<i64>,
}

#[tauri::command]
pub fn create_customer(
    database: tauri::State<'_, RuntimeDatabase>,
    input: CreateCustomerCommandInput,
) -> CommandResult<CustomerSummaryDto> {
    handle_create_customer(&database, input)
}

#[tauri::command]
pub fn search_customer_directory(
    database: tauri::State<'_, RuntimeDatabase>,
    input: SearchCustomerDirectoryCommandInput,
) -> CommandResult<Vec<CustomerDirectoryEntryDto>> {
    handle_search_customer_directory(&database, input)
}

pub fn handle_create_customer(
    database: &RuntimeDatabase,
    input: CreateCustomerCommandInput,
) -> CommandResult<CustomerSummaryDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;

        CustomerApplicationService::new(&mut connection).create_customer(input.into())
    };

    result.map(Into::into).map_err(Into::into)
}

pub fn handle_search_customer_directory(
    database: &RuntimeDatabase,
    input: SearchCustomerDirectoryCommandInput,
) -> CommandResult<Vec<CustomerDirectoryEntryDto>> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;

        CustomerApplicationService::new(&mut connection).search_directory(input.into())
    };

    result
        .map(|customers| customers.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

impl From<CreateCustomerCommandInput> for CreateCustomerInput {
    fn from(input: CreateCustomerCommandInput) -> Self {
        Self {
            name: input.name,
            phone: input.phone,
            notes: input.notes,
            created_at: input.created_at,
        }
    }
}

impl From<SearchCustomerDirectoryCommandInput> for SearchCustomerDirectoryInput {
    fn from(input: SearchCustomerDirectoryCommandInput) -> Self {
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

impl From<CustomerDirectoryEntry> for CustomerDirectoryEntryDto {
    fn from(customer: CustomerDirectoryEntry) -> Self {
        Self {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
            motorcycle_count: customer.motorcycle_count,
            last_visit_at: customer.last_visit_at,
        }
    }
}

impl From<CustomerApplicationError> for CommandError {
    fn from(error: CustomerApplicationError) -> Self {
        match error {
            CustomerApplicationError::PhoneAlreadyExists => Self {
                category: CommandErrorCategory::CustomerPhoneAlreadyExists,
                message: "A Customer with this phone number already exists.".into(),
            },

            CustomerApplicationError::InvalidTimestamp
            | CustomerApplicationError::Validation(_) => Self {
                category: CommandErrorCategory::ValidationError,
                message: "The supplied Customer data is invalid.".into(),
            },

            CustomerApplicationError::Database(_) => Self::database(),
        }
    }
}

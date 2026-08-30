use serde::{Deserialize, Serialize};

use crate::{
    application::customer::{
        CreateCustomerInput, CustomerApplicationError, CustomerApplicationService, CustomerSummary,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerSummaryDto {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[tauri::command]
pub fn create_customer(
    database: tauri::State<'_, RuntimeDatabase>,
    input: CreateCustomerCommandInput,
) -> CommandResult<CustomerSummaryDto> {
    handle_create_customer(&database, input)
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

impl From<CustomerSummary> for CustomerSummaryDto {
    fn from(customer: CustomerSummary) -> Self {
        Self {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
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

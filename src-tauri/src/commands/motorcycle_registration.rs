use serde::{Deserialize, Serialize};

use crate::{
    application::motorcycle_registration::{
        CreateMotorcycleInput, MotorcycleColorReference, MotorcycleMakeReference,
        MotorcycleRegistrationError, MotorcycleRegistrationReferenceData,
        MotorcycleRegistrationService,
    },
    commands::{
        service_visit_lookup::CustomerMotorcycleLookupDto,
        service_visit_workspace::{CommandError, CommandErrorCategory, CommandResult},
    },
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateMotorcycleCommandInput {
    pub customer_id: i64,
    pub make_id: i64,
    pub model: String,
    pub year: Option<i32>,
    pub plate_number: String,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub color_id: i64,
    pub notes: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorcycleRegistrationReferenceDataDto {
    pub makes: Vec<MotorcycleMakeReferenceDto>,
    pub colors: Vec<MotorcycleColorReferenceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorcycleMakeReferenceDto {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorcycleColorReferenceDto {
    pub id: i64,
    pub name: String,
}

#[tauri::command]
pub fn load_motorcycle_registration_reference_data(
    database: tauri::State<'_, RuntimeDatabase>,
) -> CommandResult<MotorcycleRegistrationReferenceDataDto> {
    handle_load_motorcycle_registration_reference_data(&database)
}

#[tauri::command]
pub fn create_motorcycle(
    database: tauri::State<'_, RuntimeDatabase>,
    input: CreateMotorcycleCommandInput,
) -> CommandResult<CustomerMotorcycleLookupDto> {
    handle_create_motorcycle(&database, input)
}

pub fn handle_create_motorcycle(
    database: &RuntimeDatabase,
    input: CreateMotorcycleCommandInput,
) -> CommandResult<CustomerMotorcycleLookupDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;

        MotorcycleRegistrationService::new(&mut connection).create_motorcycle(input.into())
    };

    result.map(Into::into).map_err(Into::into)
}

pub fn handle_load_motorcycle_registration_reference_data(
    database: &RuntimeDatabase,
) -> CommandResult<MotorcycleRegistrationReferenceDataDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;

        MotorcycleRegistrationService::new(&mut connection).load_reference_data()
    };

    result.map(Into::into).map_err(Into::into)
}

impl From<CreateMotorcycleCommandInput> for CreateMotorcycleInput {
    fn from(input: CreateMotorcycleCommandInput) -> Self {
        Self {
            customer_id: input.customer_id,
            make_id: input.make_id,
            model: input.model,
            year: input.year,
            plate_number: input.plate_number,
            vin: input.vin,
            chassis_number: input.chassis_number,
            color_id: input.color_id,
            notes: input.notes,
            created_at: input.created_at,
        }
    }
}

impl From<MotorcycleRegistrationReferenceData> for MotorcycleRegistrationReferenceDataDto {
    fn from(reference_data: MotorcycleRegistrationReferenceData) -> Self {
        Self {
            makes: reference_data.makes.into_iter().map(Into::into).collect(),

            colors: reference_data.colors.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<MotorcycleMakeReference> for MotorcycleMakeReferenceDto {
    fn from(make: MotorcycleMakeReference) -> Self {
        Self {
            id: make.id,
            name: make.name,
        }
    }
}

impl From<MotorcycleColorReference> for MotorcycleColorReferenceDto {
    fn from(color: MotorcycleColorReference) -> Self {
        Self {
            id: color.id,
            name: color.name,
        }
    }
}

impl From<MotorcycleRegistrationError> for CommandError {
    fn from(error: MotorcycleRegistrationError) -> Self {
        match error {
            MotorcycleRegistrationError::CustomerNotFound(_) => Self {
                category: CommandErrorCategory::CustomerNotFound,
                message: "The Customer was not found.".into(),
            },

            MotorcycleRegistrationError::InvalidTimestamp
            | MotorcycleRegistrationError::InvalidReference(_)
            | MotorcycleRegistrationError::Validation(_) => Self {
                category: CommandErrorCategory::ValidationError,
                message: "The supplied Motorcycle data is invalid.".into(),
            },

            MotorcycleRegistrationError::IdentityAlreadyExists => Self {
                category: CommandErrorCategory::MotorcycleIdentityAlreadyExists,
                message: "A Motorcycle with this identity already exists.".into(),
            },

            MotorcycleRegistrationError::Database(_) => Self::database(),
        }
    }
}

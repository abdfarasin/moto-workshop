use serde::Serialize;

use crate::{
    application::motorcycle_registration::{
        JordanPlateCodeReference, MotorcycleColorReference, MotorcycleMakeReference,
        MotorcycleRegistrationError, MotorcycleRegistrationReferenceData,
        MotorcycleRegistrationService,
    },
    commands::service_visit_workspace::{CommandError, CommandResult},
    runtime::database::RuntimeDatabase,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorcycleRegistrationReferenceDataDto {
    pub makes: Vec<MotorcycleMakeReferenceDto>,
    pub colors: Vec<MotorcycleColorReferenceDto>,
    pub plate_codes: Vec<JordanPlateCodeReferenceDto>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JordanPlateCodeReferenceDto {
    pub id: i64,
    pub code: String,
}

#[tauri::command]
pub fn load_motorcycle_registration_reference_data(
    database: tauri::State<'_, RuntimeDatabase>,
) -> CommandResult<MotorcycleRegistrationReferenceDataDto> {
    handle_load_motorcycle_registration_reference_data(&database)
}

pub fn handle_load_motorcycle_registration_reference_data(
    database: &RuntimeDatabase,
) -> CommandResult<MotorcycleRegistrationReferenceDataDto> {
    let result = {
        let connection = database.lock().map_err(|_| CommandError::database())?;
        MotorcycleRegistrationService::new(&connection).load_reference_data()
    };
    result.map(Into::into).map_err(Into::into)
}

impl From<MotorcycleRegistrationReferenceData> for MotorcycleRegistrationReferenceDataDto {
    fn from(reference_data: MotorcycleRegistrationReferenceData) -> Self {
        Self {
            makes: reference_data.makes.into_iter().map(Into::into).collect(),
            colors: reference_data.colors.into_iter().map(Into::into).collect(),
            plate_codes: reference_data
                .plate_codes
                .into_iter()
                .map(Into::into)
                .collect(),
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

impl From<JordanPlateCodeReference> for JordanPlateCodeReferenceDto {
    fn from(plate_code: JordanPlateCodeReference) -> Self {
        Self {
            id: plate_code.id,
            code: plate_code.code,
        }
    }
}

impl From<MotorcycleRegistrationError> for CommandError {
    fn from(_: MotorcycleRegistrationError) -> Self {
        Self::database()
    }
}

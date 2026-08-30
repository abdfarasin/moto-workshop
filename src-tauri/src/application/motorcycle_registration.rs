use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::repositories::motorcycle_registration::{
    MotorcycleColorRow, MotorcycleMakeRow, MotorcycleRegistrationRepository, PlateCodeRow,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleMakeReference {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleColorReference {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JordanPlateCodeReference {
    pub id: i64,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleRegistrationReferenceData {
    pub makes: Vec<MotorcycleMakeReference>,
    pub colors: Vec<MotorcycleColorReference>,
    pub plate_codes: Vec<JordanPlateCodeReference>,
}

#[derive(Debug)]
pub struct MotorcycleRegistrationError(rusqlite::Error);

impl fmt::Display for MotorcycleRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "database operation failed: {}", self.0)
    }
}

impl Error for MotorcycleRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<rusqlite::Error> for MotorcycleRegistrationError {
    fn from(error: rusqlite::Error) -> Self {
        Self(error)
    }
}

pub struct MotorcycleRegistrationService<'connection> {
    connection: &'connection Connection,
}

impl<'connection> MotorcycleRegistrationService<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn load_reference_data(
        &self,
    ) -> Result<MotorcycleRegistrationReferenceData, MotorcycleRegistrationError> {
        let repository = MotorcycleRegistrationRepository::new(self.connection);
        Ok(MotorcycleRegistrationReferenceData {
            makes: repository
                .list_active_makes()?
                .into_iter()
                .map(Into::into)
                .collect(),
            colors: repository
                .list_active_colors()?
                .into_iter()
                .map(Into::into)
                .collect(),
            plate_codes: repository
                .list_active_plate_codes()?
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl From<MotorcycleMakeRow> for MotorcycleMakeReference {
    fn from(row: MotorcycleMakeRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
        }
    }
}

impl From<MotorcycleColorRow> for MotorcycleColorReference {
    fn from(row: MotorcycleColorRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
        }
    }
}

impl From<PlateCodeRow> for JordanPlateCodeReference {
    fn from(row: PlateCodeRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
        }
    }
}

use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::{
    domain::motorcycle::{MotorcycleValidationError, NewMotorcycle, NewMotorcycleInput},
    repositories::{
        motorcycle_registration::{
            MotorcycleColorRow, MotorcycleInsertError, MotorcycleMakeRow,
            MotorcycleRegistrationRepository,
        },
        service_visit_lookup::ServiceVisitLookupRepository,
    },
};

use super::service_visit_lookup::CustomerMotorcycleLookup;

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
pub struct MotorcycleRegistrationReferenceData {
    pub makes: Vec<MotorcycleMakeReference>,
    pub colors: Vec<MotorcycleColorReference>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CreateMotorcycleInput {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorcycleRegistrationReference {
    Make,
    Color,
}

#[derive(Debug)]
pub enum MotorcycleRegistrationError {
    InvalidTimestamp,
    CustomerNotFound(i64),
    InvalidReference(MotorcycleRegistrationReference),
    Validation(MotorcycleValidationError),
    IdentityAlreadyExists,
    Database(rusqlite::Error),
}

impl fmt::Display for MotorcycleRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimestamp => {
                write!(formatter, "motorcycle timestamp is invalid")
            }
            Self::CustomerNotFound(id) => {
                write!(formatter, "customer {id} was not found")
            }
            Self::InvalidReference(reference) => {
                write!(formatter, "motorcycle reference {reference:?} is invalid")
            }
            Self::Validation(error) => {
                write!(formatter, "invalid motorcycle: {error:?}")
            }
            Self::IdentityAlreadyExists => {
                write!(formatter, "motorcycle identity already exists")
            }
            Self::Database(error) => {
                write!(formatter, "database operation failed: {error}")
            }
        }
    }
}

impl Error for MotorcycleRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MotorcycleRegistrationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<MotorcycleValidationError> for MotorcycleRegistrationError {
    fn from(error: MotorcycleValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<MotorcycleInsertError> for MotorcycleRegistrationError {
    fn from(error: MotorcycleInsertError) -> Self {
        match error {
            MotorcycleInsertError::IdentityAlreadyExists => Self::IdentityAlreadyExists,
            MotorcycleInsertError::Database(error) => Self::Database(error),
        }
    }
}

pub struct MotorcycleRegistrationService<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> MotorcycleRegistrationService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
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
        })
    }

    pub fn create_motorcycle(
        &mut self,
        input: CreateMotorcycleInput,
    ) -> Result<CustomerMotorcycleLookup, MotorcycleRegistrationError> {
        if input.created_at < 0 {
            return Err(MotorcycleRegistrationError::InvalidTimestamp);
        }

        let transaction = self.connection.transaction()?;

        let repository = MotorcycleRegistrationRepository::new(&transaction);

        if !repository.current_customer_exists(input.customer_id)? {
            return Err(MotorcycleRegistrationError::CustomerNotFound(
                input.customer_id,
            ));
        }

        if !repository.active_make_exists(input.make_id)? {
            return Err(MotorcycleRegistrationError::InvalidReference(
                MotorcycleRegistrationReference::Make,
            ));
        }

        if !repository.active_color_exists(input.color_id)? {
            return Err(MotorcycleRegistrationError::InvalidReference(
                MotorcycleRegistrationReference::Color,
            ));
        }

        let current_year = repository.current_local_year()?;

        let motorcycle = NewMotorcycle::new(
            NewMotorcycleInput {
                make_id: input.make_id,
                model: input.model,
                year: input.year,
                plate_number: input.plate_number,
                vin: input.vin,
                chassis_number: input.chassis_number,
                color_id: input.color_id,
                notes: input.notes,
            },
            current_year,
        )?;

        let motorcycle_id =
            repository.insert_motorcycle(&motorcycle, input.customer_id, input.created_at)?;

        let created = ServiceVisitLookupRepository::new(&transaction)
            .find_customer_motorcycle(input.customer_id, motorcycle_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;

        transaction.commit()?;

        Ok(created.into())
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

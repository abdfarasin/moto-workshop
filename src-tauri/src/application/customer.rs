use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::{
    domain::customer::{CustomerValidationError, NewCustomer},
    repositories::customer::{CustomerInsertError, CustomerRepository, CustomerRow},
};

#[derive(Debug, PartialEq, Eq)]
pub struct CreateCustomerInput {
    pub name: String,
    pub phone: String,
    pub notes: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerSummary {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug)]
pub enum CustomerApplicationError {
    InvalidTimestamp,
    Validation(CustomerValidationError),
    PhoneAlreadyExists,
    Database(rusqlite::Error),
}

impl fmt::Display for CustomerApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimestamp => write!(formatter, "customer timestamp is invalid"),
            Self::Validation(error) => write!(formatter, "invalid customer: {error:?}"),
            Self::PhoneAlreadyExists => write!(formatter, "customer phone already exists"),
            Self::Database(error) => write!(formatter, "database operation failed: {error}"),
        }
    }
}

impl Error for CustomerApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CustomerValidationError> for CustomerApplicationError {
    fn from(error: CustomerValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<rusqlite::Error> for CustomerApplicationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<CustomerInsertError> for CustomerApplicationError {
    fn from(error: CustomerInsertError) -> Self {
        match error {
            CustomerInsertError::PhoneAlreadyExists => Self::PhoneAlreadyExists,
            CustomerInsertError::Database(error) => Self::Database(error),
        }
    }
}

pub struct CustomerApplicationService<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> CustomerApplicationService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }

    pub fn create_customer(
        &mut self,
        input: CreateCustomerInput,
    ) -> Result<CustomerSummary, CustomerApplicationError> {
        if input.created_at < 0 {
            return Err(CustomerApplicationError::InvalidTimestamp);
        }
        let customer = NewCustomer::new(input.name, input.phone, input.notes)?;
        let transaction = self.connection.transaction()?;
        let persisted =
            CustomerRepository::new(&transaction).insert(&customer, input.created_at)?;
        transaction.commit()?;
        Ok(persisted.into())
    }
}

impl From<CustomerRow> for CustomerSummary {
    fn from(row: CustomerRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            phone: row.phone,
        }
    }
}

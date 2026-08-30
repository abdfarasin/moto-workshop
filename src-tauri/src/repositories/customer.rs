use rusqlite::{ffi, params, Connection};

use crate::domain::customer::NewCustomer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerRow {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug)]
pub enum CustomerInsertError {
    PhoneAlreadyExists,
    Database(rusqlite::Error),
}

pub struct CustomerRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> CustomerRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn insert(
        &self,
        customer: &NewCustomer,
        created_at: i64,
    ) -> Result<CustomerRow, CustomerInsertError> {
        self.connection
            .query_row(
                "INSERT INTO customers (name, phone, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 RETURNING id, name, phone",
                params![
                    customer.name(),
                    customer.phone(),
                    customer.notes(),
                    created_at,
                ],
                |row| {
                    Ok(CustomerRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        phone: row.get(2)?,
                    })
                },
            )
            .map_err(classify_insert_error)
    }
}

fn classify_insert_error(error: rusqlite::Error) -> CustomerInsertError {
    if matches!(
        &error,
        rusqlite::Error::SqliteFailure(code, _)
            if code.extended_code == ffi::SQLITE_CONSTRAINT_UNIQUE
    ) {
        CustomerInsertError::PhoneAlreadyExists
    } else {
        CustomerInsertError::Database(error)
    }
}

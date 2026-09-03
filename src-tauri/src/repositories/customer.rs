use rusqlite::{ffi, params, Connection};

use crate::domain::customer::NewCustomer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerRow {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerDirectoryRow {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub motorcycle_count: i64,
    pub last_visit_at: Option<i64>,
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
                "INSERT INTO customers (
                    name,
                    phone,
                    notes,
                    created_at,
                    updated_at
                 )
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

    pub fn search_directory(
        &self,
        query: &str,
        limit: i64,
    ) -> rusqlite::Result<Vec<CustomerDirectoryRow>> {
        let pattern = literal_substring_pattern(query);

        let mut statement = self.connection.prepare(
            "SELECT
                c.id,
                c.name,
                c.phone,

                (
                    SELECT COUNT(*)
                    FROM motorcycles m
                    WHERE m.customer_id = c.id
                      AND m.archived_at IS NULL
                ) AS motorcycle_count,

                (
                    SELECT MAX(v.opened_at)
                    FROM service_visits v
                    JOIN motorcycles m
                      ON m.id = v.motorcycle_id
                    WHERE m.customer_id = c.id
                ) AS last_visit_at

             FROM customers c

             WHERE c.archived_at IS NULL
               AND (
                    ?1 = ''
                    OR c.name LIKE ?2 ESCAPE '\\' COLLATE NOCASE
                    OR c.phone LIKE ?2 ESCAPE '\\' COLLATE NOCASE
               )

             ORDER BY c.updated_at DESC, c.id DESC
             LIMIT ?3",
        )?;

        let rows = statement.query_map(params![query, pattern, limit], |row| {
            Ok(CustomerDirectoryRow {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                motorcycle_count: row.get(3)?,
                last_visit_at: row.get(4)?,
            })
        })?;

        rows.collect()
    }
}

fn literal_substring_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");

    format!("%{escaped}%")
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

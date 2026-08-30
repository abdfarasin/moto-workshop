use rusqlite::{ffi, params, Connection};

use crate::domain::motorcycle::NewMotorcycle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleMakeRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotorcycleColorRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlateCodeRow {
    pub id: i64,
    pub code: String,
}

#[derive(Debug)]
pub enum MotorcycleInsertError {
    IdentityAlreadyExists,
    Database(rusqlite::Error),
}

pub struct MotorcycleRegistrationRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> MotorcycleRegistrationRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn list_active_makes(&self) -> rusqlite::Result<Vec<MotorcycleMakeRow>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name FROM motorcycle_makes
             WHERE active = 1
             ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(MotorcycleMakeRow {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    pub fn list_active_colors(&self) -> rusqlite::Result<Vec<MotorcycleColorRow>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name FROM motorcycle_colors
             WHERE active = 1
             ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(MotorcycleColorRow {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    pub fn list_active_plate_codes(&self) -> rusqlite::Result<Vec<PlateCodeRow>> {
        let mut statement = self.connection.prepare(
            "SELECT id, code FROM jordan_plate_codes
             WHERE active = 1
             ORDER BY code COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(PlateCodeRow {
                id: row.get(0)?,
                code: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    pub fn current_local_year(&self) -> rusqlite::Result<i32> {
        self.connection.query_row(
            "SELECT CAST(strftime('%Y', 'now', 'localtime') AS INTEGER)",
            [],
            |row| row.get(0),
        )
    }

    pub fn current_customer_exists(&self, customer_id: i64) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM customers WHERE id = ?1 AND archived_at IS NULL
             )",
            [customer_id],
            |row| row.get(0),
        )
    }

    pub fn active_make_exists(&self, make_id: i64) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM motorcycle_makes WHERE id = ?1 AND active = 1
             )",
            [make_id],
            |row| row.get(0),
        )
    }

    pub fn active_color_exists(&self, color_id: i64) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM motorcycle_colors WHERE id = ?1 AND active = 1
             )",
            [color_id],
            |row| row.get(0),
        )
    }

    pub fn active_plate_code_exists(&self, plate_code_id: i64) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM jordan_plate_codes WHERE id = ?1 AND active = 1
             )",
            [plate_code_id],
            |row| row.get(0),
        )
    }

    pub fn insert_motorcycle(
        &self,
        motorcycle: &NewMotorcycle,
        customer_id: i64,
        created_at: i64,
    ) -> Result<i64, MotorcycleInsertError> {
        let (plate_code_id, plate_number) = motorcycle
            .plate()
            .map(|plate| (plate.code_id(), i64::from(plate.number().value())))
            .unzip();
        self.connection
            .execute(
                "INSERT INTO motorcycles (
                    customer_id, make_id, model, year, plate_code_id, plate_number,
                    vin, chassis_number, color_id, notes, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
                params![
                    customer_id,
                    motorcycle.make_id(),
                    motorcycle.model(),
                    motorcycle.year(),
                    plate_code_id,
                    plate_number,
                    motorcycle.vin().map(|vin| vin.as_str()),
                    motorcycle
                        .chassis_number()
                        .map(|chassis_number| chassis_number.as_str()),
                    motorcycle.color_id(),
                    motorcycle.notes(),
                    created_at,
                ],
            )
            .map_err(classify_insert_error)?;
        Ok(self.connection.last_insert_rowid())
    }
}

fn classify_insert_error(error: rusqlite::Error) -> MotorcycleInsertError {
    if matches!(
        &error,
        rusqlite::Error::SqliteFailure(code, _)
            if code.extended_code == ffi::SQLITE_CONSTRAINT_UNIQUE
    ) {
        MotorcycleInsertError::IdentityAlreadyExists
    } else {
        MotorcycleInsertError::Database(error)
    }
}

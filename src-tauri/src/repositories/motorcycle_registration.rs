use rusqlite::Connection;

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
}

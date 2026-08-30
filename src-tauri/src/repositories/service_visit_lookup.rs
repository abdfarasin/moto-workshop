use rusqlite::{params, Connection, OptionalExtension, Row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerSummaryRow {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveServiceVisitStatusRow {
    Open,
    ReadyForPickup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerMotorcycleLookupRow {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub color_name: String,
    pub plate_code: Option<String>,
    pub plate_number: Option<i64>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub active_service_visit_id: Option<i64>,
    pub active_service_visit_status: Option<ActiveServiceVisitStatusRow>,
}

pub struct ServiceVisitLookupRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ServiceVisitLookupRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn search_customers(
        &self,
        query: &str,
        limit: i64,
    ) -> rusqlite::Result<Vec<CustomerSummaryRow>> {
        let pattern = literal_substring_pattern(query);
        let mut statement = self.connection.prepare(
            "SELECT id, name, phone
             FROM customers
             WHERE archived_at IS NULL
               AND (?1 = '' OR name LIKE ?2 ESCAPE '\\' COLLATE NOCASE
                    OR phone LIKE ?2 ESCAPE '\\' COLLATE NOCASE)
             ORDER BY updated_at DESC, id DESC
             LIMIT ?3",
        )?;
        let rows = statement.query_map(params![query, pattern, limit], |row| {
            Ok(CustomerSummaryRow {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn customer_exists(&self, customer_id: i64) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM customers WHERE id = ?1)",
            [customer_id],
            |row| row.get(0),
        )
    }

    pub fn list_customer_motorcycles(
        &self,
        customer_id: i64,
    ) -> rusqlite::Result<Vec<CustomerMotorcycleLookupRow>> {
        let mut statement = self.connection.prepare(
            "SELECT m.id, makes.name, m.model, m.year, colors.name,
                    plates.code, m.plate_number, m.vin, m.chassis_number,
                    active_visit.id, active_visit.status
             FROM motorcycles m
             JOIN motorcycle_makes makes ON makes.id = m.make_id
             JOIN motorcycle_colors colors ON colors.id = m.color_id
             LEFT JOIN jordan_plate_codes plates ON plates.id = m.plate_code_id
             LEFT JOIN service_visits active_visit
               ON active_visit.motorcycle_id = m.id
              AND active_visit.status IN ('OPEN', 'READY_FOR_PICKUP')
             WHERE m.customer_id = ?1 AND m.archived_at IS NULL
             ORDER BY makes.name COLLATE NOCASE, m.model COLLATE NOCASE, m.id",
        )?;
        let rows = statement.query_map([customer_id], map_motorcycle_row)?;
        rows.collect()
    }

    pub fn find_customer_motorcycle(
        &self,
        customer_id: i64,
        motorcycle_id: i64,
    ) -> rusqlite::Result<Option<CustomerMotorcycleLookupRow>> {
        self.connection
            .query_row(
                "SELECT m.id, makes.name, m.model, m.year, colors.name,
                        plates.code, m.plate_number, m.vin, m.chassis_number,
                        active_visit.id, active_visit.status
                 FROM motorcycles m
                 JOIN motorcycle_makes makes ON makes.id = m.make_id
                 JOIN motorcycle_colors colors ON colors.id = m.color_id
                 LEFT JOIN jordan_plate_codes plates ON plates.id = m.plate_code_id
                 LEFT JOIN service_visits active_visit
                   ON active_visit.motorcycle_id = m.id
                  AND active_visit.status IN ('OPEN', 'READY_FOR_PICKUP')
                 WHERE m.customer_id = ?1 AND m.id = ?2 AND m.archived_at IS NULL",
                (customer_id, motorcycle_id),
                map_motorcycle_row,
            )
            .optional()
    }
}

fn map_motorcycle_row(row: &Row<'_>) -> rusqlite::Result<CustomerMotorcycleLookupRow> {
    let status = match row.get::<_, Option<String>>(10)?.as_deref() {
        None => None,
        Some("OPEN") => Some(ActiveServiceVisitStatusRow::Open),
        Some("READY_FOR_PICKUP") => Some(ActiveServiceVisitStatusRow::ReadyForPickup),
        Some(_) => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(CustomerMotorcycleLookupRow {
        id: row.get(0)?,
        make_name: row.get(1)?,
        model: row.get(2)?,
        year: row.get(3)?,
        color_name: row.get(4)?,
        plate_code: row.get(5)?,
        plate_number: row.get(6)?,
        vin: row.get(7)?,
        chassis_number: row.get(8)?,
        active_service_visit_id: row.get(9)?,
        active_service_visit_status: status,
    })
}

fn literal_substring_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

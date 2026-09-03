use std::io::{Error as IoError, ErrorKind};

use rusqlite::{params, types::Type, Connection, OptionalExtension};

use crate::domain::service_visit::ServiceVisitStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MotorcycleDirectoryRow {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub color_name: String,
    pub plate_number: Option<String>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub owner_customer_id: i64,
    pub owner_name: String,
    pub owner_phone: String,
    pub latest_service_visit_at: Option<i64>,
    pub active_service_visit_id: Option<i64>,
    pub active_service_visit_status: Option<ServiceVisitStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MotorcycleServiceHistoryRow {
    pub id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub status: ServiceVisitStatus,
    pub total_fils: i64,
}

pub(crate) struct MotorcycleDirectoryRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> MotorcycleDirectoryRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn search(&self, query: &str, limit: i64) -> rusqlite::Result<Vec<MotorcycleDirectoryRow>> {
        let pattern = literal_substring_pattern(query);
        let mut statement = self.connection.prepare(
            "SELECT m.id, mk.name, m.model, m.year, mc.name, m.plate_number,
                    m.vin, m.chassis_number, c.id, c.name, c.phone,
                    (SELECT MAX(v.opened_at) FROM service_visits v WHERE v.motorcycle_id = m.id) AS latest_service_visit_at,
                    av.id, av.status
             FROM motorcycles m
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             JOIN motorcycle_colors mc ON mc.id = m.color_id
             JOIN customers c ON c.id = m.customer_id
             LEFT JOIN service_visits av
               ON av.motorcycle_id = m.id
              AND av.status IN ('OPEN', 'READY_FOR_PICKUP')
             WHERE m.archived_at IS NULL
               AND (?1 = '' OR (
                    COALESCE(m.plate_number, '') || ' ' || COALESCE(m.vin, '') || ' ' ||
                    COALESCE(m.chassis_number, '') || ' ' || mk.name || ' ' || m.model || ' ' ||
                    c.name || ' ' || c.phone
               ) LIKE ?2 ESCAPE '\\' COLLATE NOCASE)
             ORDER BY
                CASE WHEN av.id IS NULL THEN 1 ELSE 0 END,
                COALESCE(latest_service_visit_at, m.updated_at) DESC,
                m.id DESC
             LIMIT ?3",
        )?;
        let rows = statement
            .query_map(params![query, pattern, limit], map_motorcycle)?
            .collect();
        rows
    }

    pub fn find(&self, motorcycle_id: i64) -> rusqlite::Result<Option<MotorcycleDirectoryRow>> {
        self.connection
            .query_row(
                "SELECT m.id, mk.name, m.model, m.year, mc.name, m.plate_number,
                    m.vin, m.chassis_number, c.id, c.name, c.phone,
                    (SELECT MAX(v.opened_at) FROM service_visits v WHERE v.motorcycle_id = m.id),
                    av.id, av.status
             FROM motorcycles m
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             JOIN motorcycle_colors mc ON mc.id = m.color_id
             JOIN customers c ON c.id = m.customer_id
             LEFT JOIN service_visits av
               ON av.motorcycle_id = m.id
              AND av.status IN ('OPEN', 'READY_FOR_PICKUP')
             WHERE m.id = ?1 AND m.archived_at IS NULL",
                [motorcycle_id],
                map_motorcycle,
            )
            .optional()
    }

    pub fn list_service_history(
        &self,
        motorcycle_id: i64,
        limit: i64,
    ) -> rusqlite::Result<Vec<MotorcycleServiceHistoryRow>> {
        let mut statement = self.connection.prepare(
            "SELECT v.id, v.opened_at, v.odometer_km, v.customer_complaint, v.status,
                    v.labor_charge_fils + COALESCE((
                        SELECT SUM(p.line_total_fils)
                        FROM service_visit_parts p
                        WHERE p.service_visit_id = v.id AND p.status = 'ACTIVE'
                    ), 0)
             FROM service_visits v
             WHERE v.motorcycle_id = ?1
             ORDER BY v.opened_at DESC, v.id DESC
             LIMIT ?2",
        )?;
        let rows = statement
            .query_map(params![motorcycle_id, limit], |row| {
                Ok(MotorcycleServiceHistoryRow {
                    id: row.get(0)?,
                    opened_at: row.get(1)?,
                    odometer_km: row.get(2)?,
                    customer_complaint: row.get(3)?,
                    status: parse_status(row.get(4)?, 4)?,
                    total_fils: row.get(5)?,
                })
            })?
            .collect();
        rows
    }
}

fn map_motorcycle(row: &rusqlite::Row<'_>) -> rusqlite::Result<MotorcycleDirectoryRow> {
    let status = row.get::<_, Option<String>>(13)?;
    Ok(MotorcycleDirectoryRow {
        id: row.get(0)?,
        make_name: row.get(1)?,
        model: row.get(2)?,
        year: row.get(3)?,
        color_name: row.get(4)?,
        plate_number: row.get(5)?,
        vin: row.get(6)?,
        chassis_number: row.get(7)?,
        owner_customer_id: row.get(8)?,
        owner_name: row.get(9)?,
        owner_phone: row.get(10)?,
        latest_service_visit_at: row.get(11)?,
        active_service_visit_id: row.get(12)?,
        active_service_visit_status: status.map(|value| parse_status(value, 13)).transpose()?,
    })
}

fn literal_substring_pattern(query: &str) -> String {
    format!(
        "%{}%",
        query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

fn parse_status(value: String, column: usize) -> rusqlite::Result<ServiceVisitStatus> {
    match value.as_str() {
        "OPEN" => Ok(ServiceVisitStatus::Open),
        "READY_FOR_PICKUP" => Ok(ServiceVisitStatus::ReadyForPickup),
        "CLOSED" => Ok(ServiceVisitStatus::Closed),
        "CANCELLED" => Ok(ServiceVisitStatus::Cancelled),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            column,
            Type::Text,
            Box::new(IoError::new(
                ErrorKind::InvalidData,
                "unknown Service Visit status",
            )),
        )),
    }
}

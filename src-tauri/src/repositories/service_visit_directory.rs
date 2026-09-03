use std::io::{Error as IoError, ErrorKind};

use rusqlite::{params, types::Type, Connection};

use crate::domain::service_visit::ServiceVisitStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ServiceVisitDirectoryFilter {
    Active,
    All,
    Open,
    ReadyForPickup,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceVisitDirectoryRow {
    pub id: i64,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle_id: i64,
    pub make_name: String,
    pub model: String,
    pub plate_number: Option<String>,
    pub opened_at: i64,
    pub customer_complaint: String,
    pub status: ServiceVisitStatus,
    pub total_fils: i64,
}

pub(crate) struct ServiceVisitDirectoryRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ServiceVisitDirectoryRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn list(
        &self,
        query: &str,
        filter: ServiceVisitDirectoryFilter,
        limit: i64,
    ) -> rusqlite::Result<Vec<ServiceVisitDirectoryRow>> {
        let pattern = literal_substring_pattern(query);
        let filter_name = filter_name(filter);
        let mut statement = self.connection.prepare(
            "SELECT
                v.id,
                c.name,
                c.phone,
                m.id,
                mk.name,
                m.model,
                m.plate_number,
                v.opened_at,
                v.customer_complaint,
                v.status,
                v.labor_charge_fils + COALESCE((
                    SELECT SUM(p.line_total_fils)
                    FROM service_visit_parts p
                    WHERE p.service_visit_id = v.id
                      AND p.status = 'ACTIVE'
                ), 0) AS total_fils
             FROM service_visits v
             JOIN customers c ON c.id = v.owner_customer_id
             JOIN motorcycles m ON m.id = v.motorcycle_id
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             WHERE (
                    ?1 = 'ALL'
                    OR (?1 = 'ACTIVE' AND v.status IN ('OPEN', 'READY_FOR_PICKUP'))
                    OR v.status = ?1
               )
               AND (
                    ?2 = ''
                    OR c.name LIKE ?3 ESCAPE '\\' COLLATE NOCASE
                    OR c.phone LIKE ?3 ESCAPE '\\' COLLATE NOCASE
                    OR COALESCE(m.plate_number, '') LIKE ?3 ESCAPE '\\' COLLATE NOCASE
                    OR mk.name LIKE ?3 ESCAPE '\\' COLLATE NOCASE
                    OR m.model LIKE ?3 ESCAPE '\\' COLLATE NOCASE
                    OR (mk.name || ' ' || m.model) LIKE ?3 ESCAPE '\\' COLLATE NOCASE
               )
             ORDER BY
                CASE v.status
                    WHEN 'OPEN' THEN 0
                    WHEN 'READY_FOR_PICKUP' THEN 1
                    ELSE 2
                END,
                v.opened_at DESC,
                v.id DESC
             LIMIT ?4",
        )?;

        let rows = statement.query_map(params![filter_name, query, pattern, limit], |row| {
            Ok(ServiceVisitDirectoryRow {
                id: row.get(0)?,
                customer_name: row.get(1)?,
                customer_phone: row.get(2)?,
                motorcycle_id: row.get(3)?,
                make_name: row.get(4)?,
                model: row.get(5)?,
                plate_number: row.get(6)?,
                opened_at: row.get(7)?,
                customer_complaint: row.get(8)?,
                status: parse_status(row.get::<_, String>(9)?, 9)?,
                total_fils: row.get(10)?,
            })
        })?;

        rows.collect()
    }
}

fn filter_name(filter: ServiceVisitDirectoryFilter) -> &'static str {
    match filter {
        ServiceVisitDirectoryFilter::Active => "ACTIVE",
        ServiceVisitDirectoryFilter::All => "ALL",
        ServiceVisitDirectoryFilter::Open => "OPEN",
        ServiceVisitDirectoryFilter::ReadyForPickup => "READY_FOR_PICKUP",
        ServiceVisitDirectoryFilter::Closed => "CLOSED",
        ServiceVisitDirectoryFilter::Cancelled => "CANCELLED",
    }
}

fn literal_substring_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
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

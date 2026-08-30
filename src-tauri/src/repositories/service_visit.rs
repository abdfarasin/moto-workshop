use std::io::{Error as IoError, ErrorKind};

use rusqlite::{params, types::Type, Connection, OptionalExtension};

use crate::domain::{
    service_visit::{ServiceVisit, ServiceVisitStatus},
    service_visit_part::{ServiceVisitPart, ServiceVisitPartStatus},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceVisitRow {
    pub id: i64,
    pub motorcycle_id: i64,
    pub owner_customer_id: i64,
    pub status: ServiceVisitStatus,
    pub opened_at: i64,
    pub completed_at: Option<i64>,
    pub closed_at: Option<i64>,
    pub cancelled_at: Option<i64>,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub diagnosis: Option<String>,
    pub work_performed: Option<String>,
    pub labor_charge_fils: i64,
    pub cancellation_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnerRow {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MotorcycleRow {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub plate_code: Option<String>,
    pub plate_number: Option<i64>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub color_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceVisitWorkspaceRow {
    pub visit: ServiceVisitRow,
    pub owner: OwnerRow,
    pub motorcycle: MotorcycleRow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceVisitPartRow {
    pub id: i64,
    pub service_visit_id: i64,
    pub inventory_item_id: i64,
    pub item_name: String,
    pub unit_name: String,
    pub quantity: i64,
    pub quantity_scale: i64,
    pub unit_price_fils: i64,
    pub line_total_fils: i64,
    pub status: ServiceVisitPartStatus,
    pub voided_at: Option<i64>,
    pub void_reason: Option<String>,
    pub created_at: i64,
}

pub(crate) struct ServiceVisitWorkFields<'value> {
    pub diagnosis: Option<&'value str>,
    pub work_performed: Option<&'value str>,
    pub labor_charge_fils: i64,
    pub notes: Option<&'value str>,
    pub odometer_km: Option<i64>,
    pub updated_at: i64,
}

pub(crate) struct ServiceVisitLifecycleFields<'value> {
    pub status: ServiceVisitStatus,
    pub completed_at: Option<i64>,
    pub closed_at: Option<i64>,
    pub cancelled_at: Option<i64>,
    pub cancellation_reason: Option<&'value str>,
    pub updated_at: i64,
}

pub(crate) struct ServiceVisitRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ServiceVisitRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn find_motorcycle_owner(&self, motorcycle_id: i64) -> rusqlite::Result<Option<i64>> {
        self.connection
            .query_row(
                "SELECT customer_id FROM motorcycles WHERE id = ?1",
                [motorcycle_id],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn find_active_visit_id(&self, motorcycle_id: i64) -> rusqlite::Result<Option<i64>> {
        self.connection
            .query_row(
                "SELECT id FROM service_visits
                 WHERE motorcycle_id = ?1 AND status IN ('OPEN', 'READY_FOR_PICKUP')
                 LIMIT 1",
                [motorcycle_id],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn insert_service_visit(
        &self,
        visit: &ServiceVisit,
        created_at: i64,
    ) -> rusqlite::Result<i64> {
        self.connection.execute(
            "INSERT INTO service_visits (
                motorcycle_id, owner_customer_id, status, opened_at,
                completed_at, closed_at, cancelled_at, odometer_km,
                customer_complaint, diagnosis, work_performed, labor_charge_fils,
                cancellation_reason, notes, created_at, updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15
             )",
            params![
                visit.motorcycle_id(),
                visit.owner_customer_id(),
                visit_status_name(visit.status()),
                visit.opened_at(),
                visit.completed_at(),
                visit.closed_at(),
                visit.cancelled_at(),
                visit.odometer_km(),
                visit.customer_complaint(),
                visit.diagnosis(),
                visit.work_performed(),
                visit.labor_charge_fils(),
                visit.cancellation_reason(),
                visit.notes(),
                created_at,
            ],
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn find_workspace_header(
        &self,
        service_visit_id: i64,
    ) -> rusqlite::Result<Option<ServiceVisitWorkspaceRow>> {
        self.connection
            .query_row(
                "SELECT
                    v.id, v.motorcycle_id, v.owner_customer_id, v.status,
                    v.opened_at, v.completed_at, v.closed_at, v.cancelled_at,
                    v.odometer_km, v.customer_complaint, v.diagnosis,
                    v.work_performed, v.labor_charge_fils,
                    v.cancellation_reason, v.notes, v.created_at, v.updated_at,
                    c.id, c.name, c.phone,
                    m.id, mk.name, m.model, m.year, p.code, m.plate_number,
                    m.vin, m.chassis_number, mc.name
                 FROM service_visits v
                 JOIN customers c ON c.id = v.owner_customer_id
                 JOIN motorcycles m ON m.id = v.motorcycle_id
                 JOIN motorcycle_makes mk ON mk.id = m.make_id
                 JOIN motorcycle_colors mc ON mc.id = m.color_id
                 LEFT JOIN jordan_plate_codes p ON p.id = m.plate_code_id
                 WHERE v.id = ?1",
                [service_visit_id],
                |row| {
                    Ok(ServiceVisitWorkspaceRow {
                        visit: ServiceVisitRow {
                            id: row.get(0)?,
                            motorcycle_id: row.get(1)?,
                            owner_customer_id: row.get(2)?,
                            status: parse_visit_status(row.get::<_, String>(3)?, 3)?,
                            opened_at: row.get(4)?,
                            completed_at: row.get(5)?,
                            closed_at: row.get(6)?,
                            cancelled_at: row.get(7)?,
                            odometer_km: row.get(8)?,
                            customer_complaint: row.get(9)?,
                            diagnosis: row.get(10)?,
                            work_performed: row.get(11)?,
                            labor_charge_fils: row.get(12)?,
                            cancellation_reason: row.get(13)?,
                            notes: row.get(14)?,
                            created_at: row.get(15)?,
                            updated_at: row.get(16)?,
                        },
                        owner: OwnerRow {
                            id: row.get(17)?,
                            name: row.get(18)?,
                            phone: row.get(19)?,
                        },
                        motorcycle: MotorcycleRow {
                            id: row.get(20)?,
                            make_name: row.get(21)?,
                            model: row.get(22)?,
                            year: row.get(23)?,
                            plate_code: row.get(24)?,
                            plate_number: row.get(25)?,
                            vin: row.get(26)?,
                            chassis_number: row.get(27)?,
                            color_name: row.get(28)?,
                        },
                    })
                },
            )
            .optional()
    }

    pub fn list_parts(&self, service_visit_id: i64) -> rusqlite::Result<Vec<ServiceVisitPartRow>> {
        let mut statement = self.connection.prepare(
            "SELECT id, service_visit_id, inventory_item_id, item_name, unit_name,
                    quantity, quantity_scale, unit_price_fils, line_total_fils,
                    status, voided_at, void_reason, created_at
             FROM service_visit_parts
             WHERE service_visit_id = ?1
             ORDER BY created_at, id",
        )?;
        let parts = statement
            .query_map([service_visit_id], map_part)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(parts)
    }

    pub fn find_part(
        &self,
        service_visit_id: i64,
        part_id: i64,
    ) -> rusqlite::Result<Option<ServiceVisitPartRow>> {
        self.connection
            .query_row(
                "SELECT id, service_visit_id, inventory_item_id, item_name, unit_name,
                        quantity, quantity_scale, unit_price_fils, line_total_fils,
                        status, voided_at, void_reason, created_at
                 FROM service_visit_parts
                 WHERE service_visit_id = ?1 AND id = ?2",
                (service_visit_id, part_id),
                map_part,
            )
            .optional()
    }

    pub fn update_work(
        &self,
        service_visit_id: i64,
        fields: ServiceVisitWorkFields<'_>,
    ) -> rusqlite::Result<()> {
        let changed = self.connection.execute(
            "UPDATE service_visits
             SET diagnosis = ?1, work_performed = ?2, labor_charge_fils = ?3,
                 notes = ?4, odometer_km = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                fields.diagnosis,
                fields.work_performed,
                fields.labor_charge_fils,
                fields.notes,
                fields.odometer_km,
                fields.updated_at,
                service_visit_id,
            ],
        )?;
        if changed == 1 {
            Ok(())
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }

    pub fn update_lifecycle(
        &self,
        service_visit_id: i64,
        fields: ServiceVisitLifecycleFields<'_>,
    ) -> rusqlite::Result<()> {
        let changed = self.connection.execute(
            "UPDATE service_visits
             SET status = ?1, completed_at = ?2, closed_at = ?3,
                 cancelled_at = ?4, cancellation_reason = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                visit_status_name(fields.status),
                fields.completed_at,
                fields.closed_at,
                fields.cancelled_at,
                fields.cancellation_reason,
                fields.updated_at,
                service_visit_id,
            ],
        )?;
        if changed == 1 {
            Ok(())
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }

    pub fn insert_part(&self, part: &ServiceVisitPart) -> rusqlite::Result<i64> {
        self.connection.execute(
            "INSERT INTO service_visit_parts (
                service_visit_id, inventory_item_id, item_name, unit_name,
                quantity, quantity_scale, unit_price_fils, line_total_fils, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                part.service_visit_id(),
                part.inventory_item_id(),
                part.item_name(),
                part.unit_name(),
                part.quantity(),
                part.quantity_scale(),
                part.unit_price_fils(),
                part.line_total_fils(),
                part.created_at(),
            ],
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn void_part(
        &self,
        service_visit_id: i64,
        part_id: i64,
        voided_at: i64,
        void_reason: Option<&str>,
    ) -> rusqlite::Result<()> {
        let changed = self.connection.execute(
            "UPDATE service_visit_parts
             SET status = 'VOIDED', voided_at = ?1, void_reason = ?2
             WHERE service_visit_id = ?3 AND id = ?4",
            params![voided_at, void_reason, service_visit_id, part_id],
        )?;
        if changed == 1 {
            Ok(())
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    }
}

fn visit_status_name(status: ServiceVisitStatus) -> &'static str {
    match status {
        ServiceVisitStatus::Open => "OPEN",
        ServiceVisitStatus::ReadyForPickup => "READY_FOR_PICKUP",
        ServiceVisitStatus::Closed => "CLOSED",
        ServiceVisitStatus::Cancelled => "CANCELLED",
    }
}

fn map_part(row: &rusqlite::Row<'_>) -> rusqlite::Result<ServiceVisitPartRow> {
    Ok(ServiceVisitPartRow {
        id: row.get(0)?,
        service_visit_id: row.get(1)?,
        inventory_item_id: row.get(2)?,
        item_name: row.get(3)?,
        unit_name: row.get(4)?,
        quantity: row.get(5)?,
        quantity_scale: row.get(6)?,
        unit_price_fils: row.get(7)?,
        line_total_fils: row.get(8)?,
        status: parse_part_status(row.get::<_, String>(9)?, 9)?,
        voided_at: row.get(10)?,
        void_reason: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn parse_visit_status(value: String, column: usize) -> rusqlite::Result<ServiceVisitStatus> {
    match value.as_str() {
        "OPEN" => Ok(ServiceVisitStatus::Open),
        "READY_FOR_PICKUP" => Ok(ServiceVisitStatus::ReadyForPickup),
        "CLOSED" => Ok(ServiceVisitStatus::Closed),
        "CANCELLED" => Ok(ServiceVisitStatus::Cancelled),
        _ => Err(invalid_status(column, value)),
    }
}

fn parse_part_status(value: String, column: usize) -> rusqlite::Result<ServiceVisitPartStatus> {
    match value.as_str() {
        "ACTIVE" => Ok(ServiceVisitPartStatus::Active),
        "VOIDED" => Ok(ServiceVisitPartStatus::Voided),
        _ => Err(invalid_status(column, value)),
    }
}

fn invalid_status(column: usize, value: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        Type::Text,
        Box::new(IoError::new(
            ErrorKind::InvalidData,
            format!("unsupported persisted status {value}"),
        )),
    )
}

use std::io::{Error as IoError, ErrorKind};

use rusqlite::{params, types::Type, Connection, OptionalExtension};

use crate::domain::service_visit::ServiceVisitStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InvoiceStatusFilter {
    All,
    Draft,
    Issued,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvoiceDirectoryRow {
    pub id: i64,
    pub service_visit_id: i64,
    pub status: String,
    pub invoice_number: Option<String>,
    pub issued_at: Option<i64>,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle: String,
    pub plate_number: Option<String>,
    pub total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvoiceHeaderRow {
    pub id: i64,
    pub service_visit_id: i64,
    pub status: String,
    pub invoice_number: Option<String>,
    pub issued_at: Option<i64>,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle_make_name: String,
    pub motorcycle_model: String,
    pub motorcycle_plate_number: Option<String>,
    pub motorcycle_vin: Option<String>,
    pub motorcycle_chassis_number: Option<String>,
    pub labor_charge_fils: i64,
    pub parts_total_fils: i64,
    pub total_fils: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvoiceLineRow {
    pub service_visit_part_id: i64,
    pub item_name: String,
    pub unit_name: String,
    pub quantity: i64,
    pub quantity_scale: i64,
    pub unit_price_fils: i64,
    pub line_total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvoiceDetailsRow {
    pub header: InvoiceHeaderRow,
    pub lines: Vec<InvoiceLineRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvoiceIssueSourceRow {
    pub invoice_id: i64,
    pub service_visit_id: i64,
    pub invoice_status: String,
    pub service_visit_status: ServiceVisitStatus,
    pub completed_at: Option<i64>,
    pub customer_name: String,
    pub customer_phone: String,
    pub motorcycle_make_name: String,
    pub motorcycle_model: String,
    pub motorcycle_plate_number: Option<String>,
    pub motorcycle_vin: Option<String>,
    pub motorcycle_chassis_number: Option<String>,
    pub labor_charge_fils: i64,
    pub lines: Vec<InvoiceLineRow>,
}

pub(crate) struct InvoiceRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> InvoiceRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn list(
        &self,
        query: &str,
        filter: InvoiceStatusFilter,
        limit: i64,
    ) -> rusqlite::Result<Vec<InvoiceDirectoryRow>> {
        let status = match filter {
            InvoiceStatusFilter::All => None,
            InvoiceStatusFilter::Draft => Some("DRAFT"),
            InvoiceStatusFilter::Issued => Some("ISSUED"),
            InvoiceStatusFilter::Cancelled => Some("CANCELLED"),
        };
        let pattern = format!("%{query}%");
        let mut statement = self.connection.prepare(
            "SELECT i.id, i.service_visit_id, i.status, i.invoice_number, i.issued_at,
                    COALESCE(i.customer_name, c.name),
                    COALESCE(i.customer_phone, c.phone),
                    COALESCE(i.motorcycle_make_name, mk.name) || ' ' ||
                        COALESCE(i.motorcycle_model, m.model),
                    COALESCE(i.motorcycle_plate_number, m.plate_number),
                    COALESCE(i.total_fils,
                        v.labor_charge_fils + COALESCE((
                            SELECT SUM(p.line_total_fils)
                            FROM service_visit_parts p
                            WHERE p.service_visit_id = v.id AND p.status = 'ACTIVE'
                        ), 0))
             FROM invoices i
             JOIN service_visits v ON v.id = i.service_visit_id
             JOIN customers c ON c.id = v.owner_customer_id
             JOIN motorcycles m ON m.id = v.motorcycle_id
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             WHERE (?1 IS NULL OR i.status = ?1)
               AND (?2 = '' OR i.invoice_number LIKE ?3 COLLATE NOCASE
                    OR COALESCE(i.customer_name, c.name) LIKE ?3 COLLATE NOCASE
                    OR COALESCE(i.customer_phone, c.phone) LIKE ?3 COLLATE NOCASE
                    OR COALESCE(i.motorcycle_plate_number, m.plate_number) LIKE ?3 COLLATE NOCASE
                    OR CAST(i.service_visit_id AS TEXT) LIKE ?3)
             ORDER BY CASE i.status WHEN 'DRAFT' THEN 0 WHEN 'ISSUED' THEN 1 ELSE 2 END,
                      COALESCE(i.issued_at, i.created_at) DESC, i.id DESC
             LIMIT ?4",
        )?;
        let rows = statement
            .query_map(params![status, query, pattern, limit], |row| {
                Ok(InvoiceDirectoryRow {
                    id: row.get(0)?,
                    service_visit_id: row.get(1)?,
                    status: row.get(2)?,
                    invoice_number: row.get(3)?,
                    issued_at: row.get(4)?,
                    customer_name: row.get(5)?,
                    customer_phone: row.get(6)?,
                    motorcycle: row.get(7)?,
                    plate_number: row.get(8)?,
                    total_fils: row.get(9)?,
                })
            })?
            .collect();
        rows
    }

    pub fn find_details_by_id(
        &self,
        invoice_id: i64,
    ) -> rusqlite::Result<Option<InvoiceDetailsRow>> {
        self.find_details("i.id = ?1", invoice_id)
    }

    pub fn find_details_by_service_visit(
        &self,
        service_visit_id: i64,
    ) -> rusqlite::Result<Option<InvoiceDetailsRow>> {
        self.find_details("i.service_visit_id = ?1", service_visit_id)
    }

    fn find_details(
        &self,
        predicate: &str,
        value: i64,
    ) -> rusqlite::Result<Option<InvoiceDetailsRow>> {
        let sql = format!(
            "SELECT i.id, i.service_visit_id, i.status, i.invoice_number, i.issued_at,
                    COALESCE(i.customer_name, c.name), COALESCE(i.customer_phone, c.phone),
                    COALESCE(i.motorcycle_make_name, mk.name),
                    COALESCE(i.motorcycle_model, m.model),
                    COALESCE(i.motorcycle_plate_number, m.plate_number),
                    COALESCE(i.motorcycle_vin, m.vin),
                    COALESCE(i.motorcycle_chassis_number, m.chassis_number),
                    COALESCE(i.labor_charge_fils, v.labor_charge_fils),
                    COALESCE(i.parts_total_fils, (
                        SELECT COALESCE(SUM(p.line_total_fils), 0)
                        FROM service_visit_parts p
                        WHERE p.service_visit_id = v.id AND p.status = 'ACTIVE'
                    )),
                    COALESCE(i.total_fils, v.labor_charge_fils + (
                        SELECT COALESCE(SUM(p.line_total_fils), 0)
                        FROM service_visit_parts p
                        WHERE p.service_visit_id = v.id AND p.status = 'ACTIVE'
                    )), i.notes
             FROM invoices i
             JOIN service_visits v ON v.id = i.service_visit_id
             JOIN customers c ON c.id = v.owner_customer_id
             JOIN motorcycles m ON m.id = v.motorcycle_id
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             WHERE {predicate}"
        );
        let header = self
            .connection
            .query_row(&sql, [value], |row| {
                Ok(InvoiceHeaderRow {
                    id: row.get(0)?,
                    service_visit_id: row.get(1)?,
                    status: row.get(2)?,
                    invoice_number: row.get(3)?,
                    issued_at: row.get(4)?,
                    customer_name: row.get(5)?,
                    customer_phone: row.get(6)?,
                    motorcycle_make_name: row.get(7)?,
                    motorcycle_model: row.get(8)?,
                    motorcycle_plate_number: row.get(9)?,
                    motorcycle_vin: row.get(10)?,
                    motorcycle_chassis_number: row.get(11)?,
                    labor_charge_fils: row.get(12)?,
                    parts_total_fils: row.get(13)?,
                    total_fils: row.get(14)?,
                    notes: row.get(15)?,
                })
            })
            .optional()?;
        let Some(header) = header else {
            return Ok(None);
        };
        let lines = self.list_details_lines(header.id, header.service_visit_id, &header.status)?;
        Ok(Some(InvoiceDetailsRow { header, lines }))
    }

    fn list_details_lines(
        &self,
        invoice_id: i64,
        visit_id: i64,
        status: &str,
    ) -> rusqlite::Result<Vec<InvoiceLineRow>> {
        let (sql, value) = if status == "DRAFT" {
            ("SELECT id, item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils
              FROM service_visit_parts WHERE service_visit_id = ?1 AND status = 'ACTIVE'
              ORDER BY created_at, id", visit_id)
        } else {
            ("SELECT service_visit_part_id, item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils
              FROM invoice_lines WHERE invoice_id = ?1 ORDER BY id", invoice_id)
        };
        let mut statement = self.connection.prepare(sql)?;
        let lines = statement.query_map([value], map_line)?.collect();
        lines
    }

    pub fn find_issue_source(
        &self,
        service_visit_id: i64,
    ) -> rusqlite::Result<Option<InvoiceIssueSourceRow>> {
        let header = self
            .connection
            .query_row(
                "SELECT i.id, i.service_visit_id, i.status, v.status, v.completed_at,
                    c.name, c.phone, mk.name, m.model, m.plate_number, m.vin,
                    m.chassis_number, v.labor_charge_fils
             FROM invoices i
             JOIN service_visits v ON v.id = i.service_visit_id
             JOIN customers c ON c.id = v.owner_customer_id
             JOIN motorcycles m ON m.id = v.motorcycle_id
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             WHERE i.service_visit_id = ?1",
                [service_visit_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        parse_visit_status(row.get::<_, String>(3)?, 3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                        row.get(12)?,
                    ))
                },
            )
            .optional()?;
        let Some((
            invoice_id,
            visit_id,
            invoice_status,
            visit_status,
            completed_at,
            customer_name,
            customer_phone,
            make_name,
            model,
            plate,
            vin,
            chassis,
            labor,
        )) = header
        else {
            return Ok(None);
        };
        let lines = self.list_details_lines(invoice_id, visit_id, "DRAFT")?;
        Ok(Some(InvoiceIssueSourceRow {
            invoice_id,
            service_visit_id: visit_id,
            invoice_status,
            service_visit_status: visit_status,
            completed_at,
            customer_name,
            customer_phone,
            motorcycle_make_name: make_name,
            motorcycle_model: model,
            motorcycle_plate_number: plate,
            motorcycle_vin: vin,
            motorcycle_chassis_number: chassis,
            labor_charge_fils: labor,
            lines,
        }))
    }

    pub fn insert_snapshot_line(
        &self,
        invoice_id: i64,
        line: &InvoiceLineRow,
        created_at: i64,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO invoice_lines (invoice_id, service_visit_part_id, item_name, unit_name,
                quantity, quantity_scale, unit_price_fils, line_total_fils, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                invoice_id,
                line.service_visit_part_id,
                line.item_name,
                line.unit_name,
                line.quantity,
                line.quantity_scale,
                line.unit_price_fils,
                line.line_total_fils,
                created_at
            ],
        )?;
        Ok(())
    }

    pub fn mark_issued(
        &self,
        source: &InvoiceIssueSourceRow,
        number: &str,
        issued_at: i64,
        parts_total_fils: i64,
        total_fils: i64,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE invoices SET status = 'ISSUED', invoice_number = ?2, issued_at = ?3,
                customer_name = ?4, customer_phone = ?5, motorcycle_make_name = ?6,
                motorcycle_model = ?7, motorcycle_plate_number = ?8, motorcycle_vin = ?9,
                motorcycle_chassis_number = ?10, labor_charge_fils = ?11,
                parts_total_fils = ?12, total_fils = ?13, updated_at = ?3 WHERE id = ?1",
            params![
                source.invoice_id,
                number,
                issued_at,
                source.customer_name,
                source.customer_phone,
                source.motorcycle_make_name,
                source.motorcycle_model,
                source.motorcycle_plate_number,
                source.motorcycle_vin,
                source.motorcycle_chassis_number,
                source.labor_charge_fils,
                parts_total_fils,
                total_fils
            ],
        )?;
        Ok(())
    }
}

fn map_line(row: &rusqlite::Row<'_>) -> rusqlite::Result<InvoiceLineRow> {
    Ok(InvoiceLineRow {
        service_visit_part_id: row.get(0)?,
        item_name: row.get(1)?,
        unit_name: row.get(2)?,
        quantity: row.get(3)?,
        quantity_scale: row.get(4)?,
        unit_price_fils: row.get(5)?,
        line_total_fils: row.get(6)?,
    })
}

fn parse_visit_status(value: String, column: usize) -> rusqlite::Result<ServiceVisitStatus> {
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
                "invalid service visit status",
            )),
        )),
    }
}

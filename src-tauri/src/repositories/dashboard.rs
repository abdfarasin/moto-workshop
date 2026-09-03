use rusqlite::{params, Connection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardSummaryRow {
    pub active_service_visits: i64,
    pub ready_for_pickup_visits: i64,
    pub customer_count: i64,
    pub motorcycle_count: i64,
    pub low_stock_item_count: i64,
    pub negative_stock_item_count: i64,
    pub issued_invoice_count_today: i64,
    pub issued_invoice_value_today_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardServiceVisitRow {
    pub id: i64,
    pub customer_name: String,
    pub motorcycle: String,
    pub plate_number: Option<String>,
    pub opened_at: i64,
    pub status: String,
    pub complaint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardInvoiceRow {
    pub id: i64,
    pub invoice_number: String,
    pub issued_at: i64,
    pub customer_name: String,
    pub motorcycle: String,
    pub total_fils: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardInventoryAlertRow {
    pub id: i64,
    pub item_name: String,
    pub sku: Option<String>,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub current_quantity: i64,
    pub minimum_stock_quantity: i64,
}

pub(crate) struct DashboardRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> DashboardRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn load_summary(
        &self,
        day_start_ms: i64,
        day_end_ms: i64,
    ) -> rusqlite::Result<DashboardSummaryRow> {
        self.connection.query_row(
            "WITH stock AS (
                SELECT i.id, i.minimum_stock_quantity,
                       COALESCE(SUM(m.quantity_delta), 0) AS current_quantity
                FROM inventory_items i
                JOIN inventory_units u ON u.id = i.unit_id AND u.active = 1
                LEFT JOIN stock_movements m ON m.inventory_item_id = i.id
                WHERE i.archived_at IS NULL
                GROUP BY i.id, i.minimum_stock_quantity
             )
             SELECT
                (SELECT COUNT(*) FROM service_visits
                 WHERE status IN ('OPEN', 'READY_FOR_PICKUP')),
                (SELECT COUNT(*) FROM service_visits WHERE status = 'READY_FOR_PICKUP'),
                (SELECT COUNT(*) FROM customers WHERE archived_at IS NULL),
                (SELECT COUNT(*) FROM motorcycles WHERE archived_at IS NULL),
                (SELECT COUNT(*) FROM stock WHERE current_quantity <= minimum_stock_quantity),
                (SELECT COUNT(*) FROM stock WHERE current_quantity < 0),
                (SELECT COUNT(*) FROM invoices
                 WHERE status = 'ISSUED' AND issued_at >= ?1 AND issued_at < ?2),
                (SELECT COALESCE(SUM(total_fils), 0) FROM invoices
                 WHERE status = 'ISSUED' AND issued_at >= ?1 AND issued_at < ?2)",
            params![day_start_ms, day_end_ms],
            |row| {
                Ok(DashboardSummaryRow {
                    active_service_visits: row.get(0)?,
                    ready_for_pickup_visits: row.get(1)?,
                    customer_count: row.get(2)?,
                    motorcycle_count: row.get(3)?,
                    low_stock_item_count: row.get(4)?,
                    negative_stock_item_count: row.get(5)?,
                    issued_invoice_count_today: row.get(6)?,
                    issued_invoice_value_today_fils: row.get(7)?,
                })
            },
        )
    }

    pub fn list_recent_service_visits(
        &self,
        limit: i64,
    ) -> rusqlite::Result<Vec<DashboardServiceVisitRow>> {
        let mut statement = self.connection.prepare(
            "SELECT v.id, c.name, mk.name || ' ' || m.model, m.plate_number,
                    v.opened_at, v.status, v.customer_complaint
             FROM service_visits v
             JOIN customers c ON c.id = v.owner_customer_id
             JOIN motorcycles m ON m.id = v.motorcycle_id
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             ORDER BY v.opened_at DESC, v.id DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map([limit], |row| {
            Ok(DashboardServiceVisitRow {
                id: row.get(0)?,
                customer_name: row.get(1)?,
                motorcycle: row.get(2)?,
                plate_number: row.get(3)?,
                opened_at: row.get(4)?,
                status: row.get(5)?,
                complaint: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn list_recent_issued_invoices(
        &self,
        limit: i64,
    ) -> rusqlite::Result<Vec<DashboardInvoiceRow>> {
        let mut statement = self.connection.prepare(
            "SELECT id, invoice_number, issued_at, customer_name,
                    motorcycle_make_name || ' ' || motorcycle_model, total_fils
             FROM invoices
             WHERE status = 'ISSUED'
             ORDER BY issued_at DESC, id DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map([limit], |row| {
            Ok(DashboardInvoiceRow {
                id: row.get(0)?,
                invoice_number: row.get(1)?,
                issued_at: row.get(2)?,
                customer_name: row.get(3)?,
                motorcycle: row.get(4)?,
                total_fils: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn list_inventory_alerts(
        &self,
        limit: i64,
    ) -> rusqlite::Result<Vec<DashboardInventoryAlertRow>> {
        let mut statement = self.connection.prepare(
            "WITH stock AS (
                SELECT i.id, i.name, i.sku, u.name AS unit_name, u.quantity_scale,
                       i.minimum_stock_quantity,
                       COALESCE(SUM(m.quantity_delta), 0) AS current_quantity
                FROM inventory_items i
                JOIN inventory_units u ON u.id = i.unit_id AND u.active = 1
                LEFT JOIN stock_movements m ON m.inventory_item_id = i.id
                WHERE i.archived_at IS NULL
                GROUP BY i.id, i.name, i.sku, u.name, u.quantity_scale,
                         i.minimum_stock_quantity
             )
             SELECT id, name, sku, unit_name, quantity_scale, current_quantity,
                    minimum_stock_quantity
             FROM stock
             WHERE current_quantity <= minimum_stock_quantity
             ORDER BY CASE WHEN current_quantity < 0 THEN 0 ELSE 1 END,
                      (minimum_stock_quantity - current_quantity) DESC,
                      name COLLATE NOCASE, id
             LIMIT ?1",
        )?;
        let rows = statement.query_map([limit], |row| {
            Ok(DashboardInventoryAlertRow {
                id: row.get(0)?,
                item_name: row.get(1)?,
                sku: row.get(2)?,
                unit_name: row.get(3)?,
                quantity_scale: row.get(4)?,
                current_quantity: row.get(5)?,
                minimum_stock_quantity: row.get(6)?,
            })
        })?;
        rows.collect()
    }
}

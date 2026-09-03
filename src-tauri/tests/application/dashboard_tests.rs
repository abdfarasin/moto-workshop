use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::{
        dashboard::{DashboardApplicationError, DashboardApplicationService, DashboardDayRange},
        invoice::{InvoiceApplicationService, IssueInvoiceInput},
    },
    db::{migrate_database, open_database},
};

#[test]
fn empty_dashboard_returns_zero_metrics_and_empty_bounded_lists() {
    // # Arrange
    let mut fixture = empty_fixture();

    // # Act
    let dashboard = DashboardApplicationService::new(&mut fixture.connection)
        .load(DashboardDayRange {
            start_ms: 10_000,
            end_ms: 20_000,
        })
        .expect("empty dashboard should load");

    // # Assert
    assert_eq!(dashboard.summary.active_service_visits, 0);
    assert_eq!(dashboard.summary.ready_for_pickup_visits, 0);
    assert_eq!(dashboard.summary.customer_count, 0);
    assert_eq!(dashboard.summary.motorcycle_count, 0);
    assert_eq!(dashboard.summary.low_stock_item_count, 0);
    assert_eq!(dashboard.summary.negative_stock_item_count, 0);
    assert_eq!(dashboard.summary.issued_invoice_count_today, 0);
    assert_eq!(dashboard.summary.issued_invoice_value_today_fils, 0);
    assert!(dashboard.recent_service_visits.is_empty());
    assert!(dashboard.recent_invoices.is_empty());
    assert!(dashboard.inventory_alerts.is_empty());
}

#[test]
fn dashboard_uses_real_status_archive_stock_snapshot_and_day_boundary_semantics() {
    // # Arrange
    let mut fixture = empty_fixture();
    let customer_id = insert_customer(&fixture.connection, "Ahmad Ali", "+962791234567", None);
    insert_customer(
        &fixture.connection,
        "Archived",
        "+962799999999",
        Some(9_000),
    );
    let mut visits = Vec::new();
    for sequence in 1..=8 {
        let motorcycle_id = insert_motorcycle(&fixture.connection, customer_id, sequence, None);
        let status = match sequence {
            1 => "OPEN",
            2 => "READY_FOR_PICKUP",
            8 => "CANCELLED",
            _ => "CLOSED",
        };
        visits.push(insert_visit(
            &fixture.connection,
            motorcycle_id,
            customer_id,
            sequence,
            status,
        ));
    }
    insert_motorcycle(&fixture.connection, customer_id, 99, Some(8_000));

    issue(&mut fixture.connection, visits[1], 10_000);
    issue(&mut fixture.connection, visits[2], 19_999);
    issue(&mut fixture.connection, visits[3], 20_000);
    issue(&mut fixture.connection, visits[4], 20_001);
    issue(&mut fixture.connection, visits[5], 20_002);
    issue(&mut fixture.connection, visits[6], 20_003);

    insert_inventory(&fixture.connection, "Negative", "NEG", 0, -5, None);
    insert_inventory(&fixture.connection, "Low", "LOW", 5, 2, None);
    insert_inventory(&fixture.connection, "Healthy", "OK", 5, 10, None);
    insert_inventory(
        &fixture.connection,
        "Archived low",
        "OLD",
        5,
        -50,
        Some(9_000),
    );

    // # Act
    let dashboard = DashboardApplicationService::new(&mut fixture.connection)
        .load(DashboardDayRange {
            start_ms: 10_000,
            end_ms: 20_000,
        })
        .expect("dashboard should load");

    // # Assert
    assert_eq!(dashboard.summary.active_service_visits, 2);
    assert_eq!(dashboard.summary.ready_for_pickup_visits, 1);
    assert_eq!(dashboard.summary.customer_count, 1);
    assert_eq!(dashboard.summary.motorcycle_count, 8);
    assert_eq!(dashboard.summary.low_stock_item_count, 2);
    assert_eq!(dashboard.summary.negative_stock_item_count, 1);
    assert_eq!(dashboard.summary.issued_invoice_count_today, 2);
    assert_eq!(dashboard.summary.issued_invoice_value_today_fils, 5_000);
    assert_eq!(dashboard.recent_service_visits.len(), 5);
    assert_eq!(dashboard.recent_service_visits[0].id, visits[7]);
    assert_eq!(dashboard.recent_service_visits[4].id, visits[3]);
    assert_eq!(dashboard.recent_invoices.len(), 5);
    assert_eq!(dashboard.recent_invoices[0].issued_at, 20_003);
    assert_eq!(dashboard.recent_invoices[4].issued_at, 19_999);
    assert_eq!(dashboard.inventory_alerts.len(), 2);
    assert_eq!(dashboard.inventory_alerts[0].item_name, "Negative");
    assert_eq!(dashboard.inventory_alerts[1].item_name, "Low");
}

#[test]
fn dashboard_rejects_invalid_or_implausible_local_day_ranges() {
    // # Arrange
    let mut fixture = empty_fixture();

    // # Act
    let reversed =
        DashboardApplicationService::new(&mut fixture.connection).load(DashboardDayRange {
            start_ms: 20_000,
            end_ms: 10_000,
        });
    let too_long =
        DashboardApplicationService::new(&mut fixture.connection).load(DashboardDayRange {
            start_ms: 0,
            end_ms: 27 * 60 * 60 * 1_000,
        });

    // # Assert
    assert!(matches!(
        reversed,
        Err(DashboardApplicationError::InvalidDayRange)
    ));
    assert!(matches!(
        too_long,
        Err(DashboardApplicationError::InvalidDayRange)
    ));
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn empty_fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("dashboard-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

fn insert_customer(
    connection: &Connection,
    name: &str,
    phone: &str,
    archived_at: Option<i64>,
) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at, archived_at)
        VALUES (?1, ?2, 1, 1, ?3)",
            params![name, phone, archived_at],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_motorcycle(
    connection: &Connection,
    customer_id: i64,
    sequence: i64,
    archived_at: Option<i64>,
) -> i64 {
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name='Honda'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name='Black'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO motorcycles (customer_id, make_id, model, plate_number, color_id,
        created_at, updated_at, archived_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, 1, ?6)",
            params![
                customer_id,
                make_id,
                format!("CB{sequence}"),
                format!("29-{sequence}"),
                color_id,
                archived_at
            ],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_visit(
    connection: &Connection,
    motorcycle_id: i64,
    customer_id: i64,
    sequence: i64,
    status: &str,
) -> i64 {
    let opened_at = sequence * 100;
    let completed_at = matches!(status, "READY_FOR_PICKUP" | "CLOSED").then_some(opened_at + 10);
    let closed_at = (status == "CLOSED").then_some(opened_at + 20);
    let cancelled_at = (status == "CANCELLED").then_some(opened_at + 20);
    let work = completed_at.map(|_| "Completed");
    let reason = cancelled_at.map(|_| "Declined");
    connection
        .execute(
            "INSERT INTO service_visits (motorcycle_id, owner_customer_id, status,
        opened_at, completed_at, closed_at, cancelled_at, customer_complaint, work_performed,
        labor_charge_fils, cancellation_reason, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?4, ?4)",
            params![
                motorcycle_id,
                customer_id,
                status,
                opened_at,
                completed_at,
                closed_at,
                cancelled_at,
                format!("Complaint {sequence}"),
                work,
                sequence * 1_000,
                reason
            ],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn issue(connection: &mut Connection, visit_id: i64, issued_at: i64) {
    InvoiceApplicationService::new(connection)
        .issue(IssueInvoiceInput {
            service_visit_id: visit_id,
            issued_at,
        })
        .unwrap();
}

fn insert_inventory(
    connection: &Connection,
    name: &str,
    sku: &str,
    minimum: i64,
    quantity: i64,
    archived_at: Option<i64>,
) -> i64 {
    let unit_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name='Piece'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO inventory_items (name, sku, unit_id, default_selling_price_fils,
        minimum_stock_quantity, created_at, updated_at, archived_at)
        VALUES (?1, ?2, ?3, 1000, ?4, 1, 1, ?5)",
            params![name, sku, unit_id, minimum, archived_at],
        )
        .unwrap();
    let item_id = connection.last_insert_rowid();
    if quantity != 0 {
        let movement_type = if quantity > 0 {
            "ADJUSTMENT_IN"
        } else {
            "ADJUSTMENT_OUT"
        };
        connection.execute("INSERT INTO stock_movements (inventory_item_id, movement_type, quantity_delta, created_at)
            VALUES (?1, ?2, ?3, 2)", params![item_id, movement_type, quantity]).unwrap();
    }
    item_id
}

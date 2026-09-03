use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::invoice::{
        InvoiceApplicationError, InvoiceApplicationService, InvoiceStatus, IssueInvoiceInput,
        ListInvoicesInput,
    },
    db::{migrate_database, open_database},
};

#[test]
fn issues_only_active_lines_and_preserves_the_historical_snapshot() {
    // # Arrange
    let mut fixture = fixture();
    fixture
        .connection
        .execute(
            "UPDATE service_visit_parts SET status = 'VOIDED', voided_at = 1800,
            void_reason = 'Not used' WHERE id = ?1",
            [fixture.voided_part_id],
        )
        .unwrap();
    fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'READY_FOR_PICKUP', completed_at = 2000,
            work_performed = 'Replaced filter', labor_charge_fils = 12500, updated_at = 2000
         WHERE id = ?1",
            [fixture.visit_id],
        )
        .unwrap();

    // # Act
    let issued = InvoiceApplicationService::new(&mut fixture.connection)
        .issue(IssueInvoiceInput {
            service_visit_id: fixture.visit_id,
            issued_at: 2100,
        })
        .expect("ready work should issue");
    fixture
        .connection
        .execute(
            "UPDATE customers SET name = 'Changed Name' WHERE id = ?1",
            [fixture.owner_id],
        )
        .unwrap();
    fixture
        .connection
        .execute(
            "UPDATE service_visits SET labor_charge_fils = 99000, updated_at = 2150
             WHERE id = ?1",
            [fixture.visit_id],
        )
        .unwrap();
    let item_id: i64 = fixture
        .connection
        .query_row(
            "SELECT id FROM inventory_items WHERE sku = 'FILTER'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    fixture
        .connection
        .execute(
            "INSERT INTO service_visit_parts (service_visit_id, inventory_item_id,
            item_name, unit_name, quantity, quantity_scale, unit_price_fils,
            line_total_fils, created_at)
         VALUES (?1, ?2, 'Oil Filter', 'Piece', 1, 1, 4500, 4500, 2160)",
            params![fixture.visit_id, item_id],
        )
        .unwrap();
    let reloaded = InvoiceApplicationService::new(&mut fixture.connection)
        .load(issued.id)
        .unwrap();
    let duplicate = InvoiceApplicationService::new(&mut fixture.connection)
        .issue(IssueInvoiceInput {
            service_visit_id: fixture.visit_id,
            issued_at: 2200,
        })
        .expect_err("issued invoice cannot be issued again");

    // # Assert
    assert_eq!(issued.status, InvoiceStatus::Issued);
    assert_eq!(issued.invoice_number.as_deref(), Some("INV-000001"));
    assert_eq!(issued.customer_name, "Ahmad Ali");
    assert_eq!(issued.labor_charge_fils, 12_500);
    assert_eq!(issued.parts_total_fils, 9_000);
    assert_eq!(issued.total_fils, 21_500);
    assert_eq!(issued.lines.len(), 1);
    assert_eq!(issued.lines[0].item_name, "Oil Filter");
    assert_eq!(reloaded, issued);
    assert!(matches!(
        duplicate,
        InvoiceApplicationError::InvoiceAlreadyIssued(1)
    ));
}

#[test]
fn directory_filters_searches_and_caps_inside_the_repository_query() {
    // # Arrange
    let mut fixture = fixture();

    // # Act
    let matches = InvoiceApplicationService::new(&mut fixture.connection)
        .list(ListInvoicesInput {
            query: "29-12345".into(),
            status_filter: None,
            limit: Some(500),
        })
        .unwrap();
    let misses = InvoiceApplicationService::new(&mut fixture.connection)
        .list(ListInvoicesInput {
            query: "nobody".into(),
            status_filter: None,
            limit: None,
        })
        .unwrap();

    // # Assert
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].service_visit_id, fixture.visit_id);
    assert_eq!(matches[0].total_fils, 18_500);
    assert!(misses.is_empty());
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
    owner_id: i64,
    visit_id: i64,
    voided_part_id: i64,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("invoice-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
        VALUES ('Ahmad Ali', '+962791234567', 1000, 1000)",
            [],
        )
        .unwrap();
    let owner_id = connection.last_insert_rowid();
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection.execute("INSERT INTO motorcycles (customer_id, make_id, model, year, plate_number,
        color_id, created_at, updated_at) VALUES (?1, ?2, 'CB150R', 2022, '29-12345', ?3, 1000, 1000)",
        params![owner_id, make_id, color_id]).unwrap();
    let motorcycle_id = connection.last_insert_rowid();
    connection
        .execute(
            "INSERT INTO service_visits (motorcycle_id, owner_customer_id, status,
        opened_at, customer_complaint, labor_charge_fils, created_at, updated_at)
        VALUES (?1, ?2, 'OPEN', 1000, 'Oil leak', 500, 1000, 1000)",
            params![motorcycle_id, owner_id],
        )
        .unwrap();
    let visit_id = connection.last_insert_rowid();
    let unit_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name = 'Piece'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO inventory_items (name, sku, unit_id, default_selling_price_fils,
        created_at, updated_at) VALUES ('Oil Filter', 'FILTER', ?1, 4500, 1000, 1000)",
            [unit_id],
        )
        .unwrap();
    let item_id = connection.last_insert_rowid();
    for created_at in [1500_i64, 1600_i64] {
        connection.execute("INSERT INTO service_visit_parts (service_visit_id, inventory_item_id,
            item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils, created_at)
            VALUES (?1, ?2, 'Oil Filter', 'Piece', 2, 1, 4500, 9000, ?3)",
            params![visit_id, item_id, created_at]).unwrap();
    }
    let voided_part_id = connection.last_insert_rowid();
    Fixture {
        _temp_dir: temp_dir,
        connection,
        owner_id,
        visit_id,
        voided_part_id,
    }
}

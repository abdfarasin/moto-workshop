use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

use moto_workshop_lib::{
    commands::{
        invoice::{
            handle_issue_invoice, handle_list_invoices, handle_load_invoice_details,
            IssueInvoiceCommandInput, ListInvoicesCommandInput,
        },
        service_visit_workspace::CommandErrorCategory,
    },
    db::open_database,
    runtime::database::RuntimeDatabase,
};

#[test]
fn invoice_commands_expose_camel_case_snapshots_and_stable_errors() {
    // # Arrange
    let directory = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(directory.path()).unwrap();
    let visit_id = insert_ready_visit(&database);

    // # Act
    let issued = handle_issue_invoice(
        &database,
        IssueInvoiceCommandInput {
            service_visit_id: visit_id,
            issued_at: 2_100,
        },
    )
    .unwrap();
    let listed = handle_list_invoices(
        &database,
        ListInvoicesCommandInput {
            query: "Ahmad".into(),
            status_filter: None,
            limit: Some(50),
        },
    )
    .unwrap();
    let json = serde_json::to_value(&issued).unwrap();
    let missing = handle_load_invoice_details(&database, 999).unwrap_err();

    // # Assert
    assert_eq!(json["status"], "ISSUED");
    assert_eq!(json["invoiceNumber"], "INV-000001");
    assert_eq!(json["customerName"], "Ahmad Ali");
    assert_eq!(json["totalFils"], 12_500);
    assert_eq!(listed.len(), 1);
    assert_eq!(missing.category, CommandErrorCategory::InvoiceNotFound);
    assert!(!missing.message.to_ascii_lowercase().contains("select"));
}

#[test]
fn invoice_write_input_rejects_caller_controlled_totals_and_snapshots() {
    let unsafe_input = json!({
        "serviceVisitId": 1, "issuedAt": 2,
        "totalFils": 1, "customerName": "Forged"
    });
    assert!(serde_json::from_value::<IssueInvoiceCommandInput>(unsafe_input).is_err());
}

fn insert_ready_visit(database: &RuntimeDatabase) -> i64 {
    let connection = open_database(database.database_path()).unwrap();
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
        VALUES ('Ahmad Ali', '+962791234567', 1000, 1000)",
            [],
        )
        .unwrap();
    let customer_id = connection.last_insert_rowid();
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
    connection
        .execute(
            "INSERT INTO motorcycles (customer_id, make_id, model, plate_number,
        color_id, created_at, updated_at) VALUES (?1, ?2, 'CB150R', '29-12345', ?3, 1000, 1000)",
            params![customer_id, make_id, color_id],
        )
        .unwrap();
    let motorcycle_id = connection.last_insert_rowid();
    connection
        .execute(
            "INSERT INTO service_visits (motorcycle_id, owner_customer_id, status,
        opened_at, completed_at, customer_complaint, work_performed, labor_charge_fils,
        created_at, updated_at) VALUES (?1, ?2, 'READY_FOR_PICKUP', 1000, 2000,
        'Oil leak', 'Replaced filter', 12500, 1000, 2000)",
            params![motorcycle_id, customer_id],
        )
        .unwrap();
    connection.last_insert_rowid()
}

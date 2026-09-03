use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::customer_details::{CustomerDetailsApplicationService, LoadCustomerDetailsInput},
    db::{migrate_database, open_database},
};

#[test]
fn loads_customer_with_real_motorcycles_and_service_history() {
    // # Arrange
    let mut fixture = fixture();

    let customer_id = insert_customer(&fixture.connection, "Ahmad Ali", "+962791234567", 1_000);

    let motorcycle_id = insert_motorcycle(&fixture.connection, customer_id, "47-122132", 1_100);

    let visit_id = insert_service_visit(&fixture.connection, motorcycle_id, customer_id, 5_000);

    // # Act
    let details = CustomerDetailsApplicationService::new(&mut fixture.connection)
        .load(LoadCustomerDetailsInput { customer_id })
        .unwrap()
        .expect("customer should exist");

    // # Assert
    assert_eq!(details.id, customer_id);
    assert_eq!(details.name, "Ahmad Ali");
    assert_eq!(details.phone, "+962791234567");

    assert_eq!(details.motorcycles.len(), 1);

    let motorcycle = &details.motorcycles[0];

    assert_eq!(motorcycle.id, motorcycle_id);
    assert_eq!(motorcycle.make_name, "Honda");
    assert_eq!(motorcycle.model, "CB150R");
    assert_eq!(motorcycle.plate_number.as_deref(), Some("47-122132"),);
    assert_eq!(motorcycle.color_name, "Black");

    assert_eq!(details.service_history.len(), 1);

    let visit = &details.service_history[0];

    assert_eq!(visit.id, visit_id);
    assert_eq!(visit.motorcycle_id, motorcycle_id);
    assert_eq!(visit.opened_at, 5_000);
    assert_eq!(visit.customer_complaint, "Routine inspection",);
    assert_eq!(visit.total_fils, 0);
}

#[test]
fn customer_details_returns_none_for_unknown_customer() {
    // # Arrange
    let mut fixture = fixture();

    // # Act
    let details = CustomerDetailsApplicationService::new(&mut fixture.connection)
        .load(LoadCustomerDetailsInput {
            customer_id: 999_999,
        })
        .unwrap();

    // # Assert
    assert_eq!(details, None);
}

fn insert_customer(connection: &Connection, name: &str, phone: &str, timestamp: i64) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (
                name,
                phone,
                created_at,
                updated_at
             )
             VALUES (?1, ?2, ?3, ?3)",
            params![name, phone, timestamp],
        )
        .unwrap();

    connection.last_insert_rowid()
}

fn insert_motorcycle(
    connection: &Connection,
    customer_id: i64,
    plate_number: &str,
    timestamp: i64,
) -> i64 {
    let make_id: i64 = connection
        .query_row(
            "SELECT id
             FROM motorcycle_makes
             WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    let color_id: i64 = connection
        .query_row(
            "SELECT id
             FROM motorcycle_colors
             WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id,
                make_id,
                model,
                plate_number,
                color_id,
                created_at,
                updated_at
             )
             VALUES (
                ?1,
                ?2,
                'CB150R',
                ?3,
                ?4,
                ?5,
                ?5
             )",
            params![customer_id, make_id, plate_number, color_id, timestamp,],
        )
        .unwrap();

    connection.last_insert_rowid()
}

fn insert_service_visit(
    connection: &Connection,
    motorcycle_id: i64,
    customer_id: i64,
    opened_at: i64,
) -> i64 {
    connection
        .execute(
            "INSERT INTO service_visits (
                motorcycle_id,
                owner_customer_id,
                status,
                opened_at,
                customer_complaint,
                labor_charge_fils,
                created_at,
                updated_at
             )
             VALUES (
                ?1,
                ?2,
                'OPEN',
                ?3,
                'Routine inspection',
                0,
                ?3,
                ?3
             )",
            params![motorcycle_id, customer_id, opened_at,],
        )
        .unwrap();

    connection.last_insert_rowid()
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();

    let mut connection = open_database(temp_dir.path().join("customer-details-test.db")).unwrap();

    migrate_database(&mut connection).unwrap();

    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

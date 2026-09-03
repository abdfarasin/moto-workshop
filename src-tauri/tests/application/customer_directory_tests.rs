use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::customer::{CustomerApplicationService, SearchCustomerDirectoryInput},
    db::{migrate_database, open_database},
};

#[test]
fn searches_real_customers_with_motorcycle_count_and_last_visit() {
    // # Arrange
    let mut fixture = fixture();

    let ahmad_id = insert_customer(&fixture.connection, "Ahmad Ali", "+962791234567", 1_000);

    let maya_id = insert_customer(&fixture.connection, "Maya Saleh", "+962791234568", 2_000);

    let ahmad_motorcycle_id = insert_motorcycle(&fixture.connection, ahmad_id, "47-122132", 1_100);

    insert_motorcycle(&fixture.connection, ahmad_id, "29-55555", 1_200);

    insert_service_visit(&fixture.connection, ahmad_motorcycle_id, ahmad_id, 5_000);

    // # Act
    let customers = CustomerApplicationService::new(&mut fixture.connection)
        .search_directory(SearchCustomerDirectoryInput {
            query: "Ahmad".into(),
            limit: None,
        })
        .unwrap();

    // # Assert
    assert_eq!(customers.len(), 1);

    let customer = &customers[0];

    assert_eq!(customer.id, ahmad_id);
    assert_eq!(customer.name, "Ahmad Ali");
    assert_eq!(customer.phone, "+962791234567");
    assert_eq!(customer.motorcycle_count, 2);
    assert_eq!(customer.last_visit_at, Some(5_000));

    assert_ne!(customer.id, maya_id);
}

#[test]
fn customer_directory_searches_by_phone_and_returns_customers_without_history() {
    // # Arrange
    let mut fixture = fixture();

    let customer_id = insert_customer(&fixture.connection, "Maya Saleh", "+962791234568", 2_000);

    // # Act
    let customers = CustomerApplicationService::new(&mut fixture.connection)
        .search_directory(SearchCustomerDirectoryInput {
            query: "1234568".into(),
            limit: None,
        })
        .unwrap();

    // # Assert
    assert_eq!(customers.len(), 1);

    assert_eq!(customers[0].id, customer_id);
    assert_eq!(customers[0].motorcycle_count, 0);
    assert_eq!(customers[0].last_visit_at, None);
}

#[test]
fn customer_directory_respects_the_requested_result_limit() {
    // # Arrange
    let mut fixture = fixture();

    insert_customer(&fixture.connection, "Customer One", "+962791234561", 1_000);

    insert_customer(&fixture.connection, "Customer Two", "+962791234562", 2_000);

    insert_customer(
        &fixture.connection,
        "Customer Three",
        "+962791234563",
        3_000,
    );

    // # Act
    let customers = CustomerApplicationService::new(&mut fixture.connection)
        .search_directory(SearchCustomerDirectoryInput {
            query: String::new(),
            limit: Some(2),
        })
        .unwrap();

    // # Assert
    assert_eq!(customers.len(), 2);
    assert_eq!(customers[0].name, "Customer Three");
    assert_eq!(customers[1].name, "Customer Two");
}

fn insert_customer(connection: &Connection, name: &str, phone: &str, timestamp: i64) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (
                name,
                phone,
                created_at,
                updated_at
             ) VALUES (?1, ?2, ?3, ?3)",
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
             ) VALUES (
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
) {
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
             ) VALUES (
                ?1,
                ?2,
                'OPEN',
                ?3,
                'Routine inspection',
                0,
                ?3,
                ?3
             )",
            params![motorcycle_id, customer_id, opened_at],
        )
        .unwrap();
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();

    let mut connection = open_database(temp_dir.path().join("customer-directory-test.db")).unwrap();

    migrate_database(&mut connection).unwrap();

    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

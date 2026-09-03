use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

use moto_workshop_lib::{
    commands::service_visit_directory::{
        handle_list_service_visits, ListServiceVisitsCommandInput,
        ServiceVisitDirectoryStatusFilterDto,
    },
    db::open_database,
    runtime::database::RuntimeDatabase,
};

#[test]
fn directory_command_maps_exact_camel_case_input_and_output() {
    // # Arrange
    let temp_dir = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(temp_dir.path()).unwrap();
    {
        let connection = open_database(database.database_path()).unwrap();
        let customer_id = insert_customer(&connection);
        let motorcycle_id = insert_motorcycle(&connection, customer_id);
        connection
            .execute(
                "INSERT INTO service_visits (
                motorcycle_id, owner_customer_id, status, opened_at,
                customer_complaint, created_at, updated_at
             ) VALUES (?1, ?2, 'OPEN', 1000, 'Oil leak', 1000, 1000)",
                params![motorcycle_id, customer_id],
            )
            .unwrap();
    }

    // # Act
    let visits = handle_list_service_visits(
        &database,
        ListServiceVisitsCommandInput {
            query: "Ahmad".into(),
            status_filter: Some(ServiceVisitDirectoryStatusFilterDto::Active),
            limit: Some(25),
        },
    )
    .unwrap();
    let serialized = serde_json::to_value(&visits).unwrap();

    // # Assert
    assert_eq!(serialized[0]["customerName"], "Ahmad Ali");
    assert_eq!(serialized[0]["customerPhone"], "+962791234567");
    assert_eq!(serialized[0]["makeName"], "Honda");
    assert_eq!(serialized[0]["plateNumber"], "29-12345");
    assert_eq!(serialized[0]["status"], "OPEN");
    assert_eq!(serialized[0]["totalFils"], 0);

    let parsed: ListServiceVisitsCommandInput = serde_json::from_value(json!({
        "query": "",
        "statusFilter": "READY_FOR_PICKUP",
        "limit": 50
    }))
    .unwrap();
    assert_eq!(
        parsed.status_filter,
        Some(ServiceVisitDirectoryStatusFilterDto::ReadyForPickup)
    );
    assert!(
        serde_json::from_value::<ListServiceVisitsCommandInput>(json!({
            "query": "",
            "statusFilter": "ACTIVE",
            "limit": 50,
            "databasePath": "forged.db"
        }))
        .is_err()
    );
}

fn insert_customer(connection: &rusqlite::Connection) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
         VALUES ('Ahmad Ali', '+962791234567', 1, 1)",
            [],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_motorcycle(connection: &rusqlite::Connection, customer_id: i64) -> i64 {
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
            "INSERT INTO motorcycles (
            customer_id, make_id, model, plate_number, color_id, created_at, updated_at
         ) VALUES (?1, ?2, 'CB150R', '29-12345', ?3, 1, 1)",
            params![customer_id, make_id, color_id],
        )
        .unwrap();
    connection.last_insert_rowid()
}

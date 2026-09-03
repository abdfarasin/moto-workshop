use moto_workshop_lib::{
    commands::motorcycle_directory::{
        handle_load_motorcycle_details, handle_search_motorcycle_directory,
        LoadMotorcycleDetailsCommandInput, SearchMotorcycleDirectoryCommandInput,
    },
    db::open_database,
    runtime::database::RuntimeDatabase,
};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

#[test]
fn commands_map_exact_camel_case_contract_and_not_found_error() {
    // # Arrange
    let directory = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(directory.path()).unwrap();
    let motorcycle_id = {
        let connection = open_database(database.database_path()).unwrap();
        connection.execute("INSERT INTO customers (name, phone, created_at, updated_at) VALUES ('Ahmad Ali', '+962791234567', 1, 1)", []).unwrap();
        let customer_id = connection.last_insert_rowid();
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
        connection.execute("INSERT INTO motorcycles (customer_id, make_id, model, plate_number, color_id, created_at, updated_at) VALUES (?1, ?2, 'CB150R', '29-12345', ?3, 1, 1)",params![customer_id,make_id,color_id]).unwrap();
        connection.last_insert_rowid()
    };

    // # Act
    let listed = handle_search_motorcycle_directory(
        &database,
        SearchMotorcycleDirectoryCommandInput {
            query: "29-12345".into(),
            limit: Some(50),
        },
    )
    .unwrap();
    let details = handle_load_motorcycle_details(
        &database,
        LoadMotorcycleDetailsCommandInput { motorcycle_id },
    )
    .unwrap();
    let missing = handle_load_motorcycle_details(
        &database,
        LoadMotorcycleDetailsCommandInput {
            motorcycle_id: 999_999,
        },
    )
    .unwrap_err();

    // # Assert
    let listed_json = serde_json::to_value(listed).unwrap();
    let details_json = serde_json::to_value(details).unwrap();
    assert_eq!(listed_json[0]["ownerName"], "Ahmad Ali");
    assert_eq!(details_json["plateNumber"], "29-12345");
    assert_eq!(
        serde_json::to_value(missing).unwrap()["category"],
        "motorcycleNotFound"
    );
    assert!(
        serde_json::from_value::<SearchMotorcycleDirectoryCommandInput>(
            json!({"query":"","limit":50,"databasePath":"forged.db"})
        )
        .is_err()
    );
}

use rusqlite::{params, Connection};
use serde_json::json;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    commands::{
        service_visit_lookup::{
            handle_list_customer_motorcycles, handle_search_customers,
            ListCustomerMotorcyclesCommandInput, SearchCustomersCommandInput,
        },
        service_visit_workspace::CommandErrorCategory,
    },
    db::open_database,
    runtime::database::RuntimeDatabase,
};

#[test]
fn lookup_handlers_return_exact_camel_case_dtos_and_active_status() {
    // # Arrange
    let fixture = fixture();

    // # Act
    let customers = handle_search_customers(
        &fixture.database,
        SearchCustomersCommandInput {
            query: "Ahmad".into(),
            limit: Some(25),
        },
    )
    .unwrap();
    let motorcycles = handle_list_customer_motorcycles(
        &fixture.database,
        ListCustomerMotorcyclesCommandInput {
            customer_id: fixture.customer_id,
        },
    )
    .unwrap();

    // # Assert
    assert_eq!(
        serde_json::to_value(&customers).unwrap(),
        json!([{
            "id": fixture.customer_id,
            "name": "Ahmad Ali",
            "phone": "+962791234567"
        }])
    );
    assert_eq!(
        serde_json::to_value(&motorcycles).unwrap(),
        json!([{
            "id": fixture.motorcycle_id,
            "makeName": "Honda",
            "model": "CB150R",
            "year": 2022,
            "colorName": "Black",
            "plateNumber": "29-12345",
            "vin": null,
            "chassisNumber": null,
            "activeServiceVisitId": fixture.visit_id,
            "activeServiceVisitStatus": "OPEN"
        }])
    );
}

#[test]
fn lookup_inputs_reject_unknown_fields_and_missing_customer_has_stable_category() {
    // # Arrange
    let fixture = fixture();
    let forged_search = json!({ "query": "Ahmad", "limit": 25, "sql": "DROP TABLE customers" });
    let forged_list = json!({ "customerId": fixture.customer_id, "activeStatus": "CLOSED" });

    // # Act
    let search_input = serde_json::from_value::<SearchCustomersCommandInput>(forged_search);
    let list_input = serde_json::from_value::<ListCustomerMotorcyclesCommandInput>(forged_list);
    let missing = handle_list_customer_motorcycles(
        &fixture.database,
        ListCustomerMotorcyclesCommandInput {
            customer_id: 999_999,
        },
    )
    .expect_err("missing Customer should map to a stable command error");

    // # Assert
    assert!(search_input.is_err());
    assert!(list_input.is_err());
    assert_eq!(missing.category, CommandErrorCategory::CustomerNotFound);
    assert_eq!(missing.message, "The Customer was not found.");
    assert_eq!(
        serde_json::to_value(missing).unwrap()["category"],
        "customerNotFound"
    );
}

struct Fixture {
    _temp_dir: TempDir,
    database: RuntimeDatabase,
    _seed_connection: Connection,
    customer_id: i64,
    motorcycle_id: i64,
    visit_id: i64,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(temp_dir.path()).unwrap();
    let seed_connection = open_database(database.database_path()).unwrap();
    seed_connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES ('Ahmad Ali', '+962791234567', 1000, 1000)",
            [],
        )
        .unwrap();
    let customer_id = seed_connection.last_insert_rowid();
    let make_id: i64 = seed_connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = seed_connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    seed_connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, year, plate_number,
                color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'CB150R', 2022, '29-12345', ?3, 1000, 1000)",
            params![customer_id, make_id, color_id],
        )
        .unwrap();
    let motorcycle_id = seed_connection.last_insert_rowid();
    seed_connection
        .execute(
            "INSERT INTO service_visits (
                motorcycle_id, owner_customer_id, status, opened_at,
                customer_complaint, created_at, updated_at
             ) VALUES (?1, ?2, 'OPEN', 1000, 'Oil leak', 1000, 1000)",
            (motorcycle_id, customer_id),
        )
        .unwrap();
    let visit_id = seed_connection.last_insert_rowid();
    Fixture {
        _temp_dir: temp_dir,
        database,
        _seed_connection: seed_connection,
        customer_id,
        motorcycle_id,
        visit_id,
    }
}

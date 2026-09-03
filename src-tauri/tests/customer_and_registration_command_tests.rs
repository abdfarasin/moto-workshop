use serde_json::json;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    commands::{
        customer::{handle_create_customer, CreateCustomerCommandInput},
        motorcycle_registration::{
            handle_create_motorcycle, handle_load_motorcycle_registration_reference_data,
            CreateMotorcycleCommandInput,
        },
        service_visit_workspace::CommandErrorCategory,
    },
    db::open_database,
    runtime::database::RuntimeDatabase,
};

#[test]
fn create_customer_handler_uses_exact_camel_case_dto_and_sanitizes_duplicates() {
    // # Arrange
    let fixture = fixture();
    let safe = json!({
        "name": "  Ahmad Ali  ",
        "phone": "0791234567",
        "notes": "  Prefers WhatsApp  ",
        "createdAt": 1234
    });
    let forged = json!({
        "name": "Ahmad Ali",
        "phone": "0791234567",
        "notes": null,
        "createdAt": 1234,
        "updatedAt": 9999,
        "archivedAt": 9999,
        "normalizedPhone": "+962700000000"
    });

    // # Act
    let input = serde_json::from_value::<CreateCustomerCommandInput>(safe).unwrap();
    let created = handle_create_customer(&fixture.database, input).unwrap();
    let forged = serde_json::from_value::<CreateCustomerCommandInput>(forged);
    let duplicate = handle_create_customer(
        &fixture.database,
        CreateCustomerCommandInput {
            name: "Duplicate".into(),
            phone: "00962791234567".into(),
            notes: None,
            created_at: 2_000,
        },
    )
    .expect_err("alternate canonical representation should be a friendly duplicate");

    // # Assert
    assert_eq!(
        serde_json::to_value(created).unwrap(),
        json!({
            "id": 1,
            "name": "Ahmad Ali",
            "phone": "+962791234567"
        })
    );
    assert!(forged.is_err());
    assert_eq!(
        duplicate.category,
        CommandErrorCategory::CustomerPhoneAlreadyExists
    );
    assert_eq!(
        duplicate.message,
        "A Customer with this phone number already exists."
    );
    let serialized = serde_json::to_value(duplicate).unwrap();
    assert_eq!(serialized["category"], "customerPhoneAlreadyExists");
    let message = serialized["message"].as_str().unwrap().to_ascii_lowercase();
    assert!(!message.contains("unique"));
    assert!(!message.contains("constraint"));
    assert!(!message.contains("sqlite"));
}

#[test]
fn registration_reference_handler_returns_exact_active_camel_case_catalogs() {
    // # Arrange
    let fixture = fixture();
    let connection = open_database(fixture.database.database_path()).unwrap();
    connection
        .execute("UPDATE motorcycle_makes SET active = 0", [])
        .unwrap();
    connection
        .execute("UPDATE motorcycle_colors SET active = 0", [])
        .unwrap();
    connection
        .execute(
            "INSERT INTO motorcycle_makes (name, active) VALUES ('Test Make', 1)",
            [],
        )
        .unwrap();
    let make_id = connection.last_insert_rowid();
    connection
        .execute(
            "INSERT INTO motorcycle_colors (name, active) VALUES ('Test Color', 1)",
            [],
        )
        .unwrap();
    let color_id = connection.last_insert_rowid();
    // # Act
    let result = handle_load_motorcycle_registration_reference_data(&fixture.database).unwrap();

    // # Assert
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        json!({
            "makes": [{ "id": make_id, "name": "Test Make" }],
            "colors": [{ "id": color_id, "name": "Test Color" }]
        })
    );
}

#[test]
fn create_motorcycle_handler_uses_safe_camel_case_input_and_sanitized_errors() {
    // # Arrange
    let fixture = fixture();
    let connection = open_database(fixture.database.database_path()).unwrap();
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
    let safe = json!({
        "customerId": customer_id,
        "makeId": make_id,
        "model": "  CB150R  ",
        "year": null,
        "plateNumber": "29-12345",
        "vin": null,
        "chassisNumber": null,
        "colorId": color_id,
        "notes": null,
        "createdAt": 2000
    });
    let forged = json!({
        "customerId": customer_id,
        "makeId": make_id,
        "model": "CB150R",
        "year": null,
        "plateNumber": "29-12345",
        "vin": null,
        "chassisNumber": null,
        "colorId": color_id,
        "notes": null,
        "createdAt": 2000,
        "currentYear": 2099,
        "updatedAt": 2099,
        "archivedAt": 2099
    });

    // # Act
    let input = serde_json::from_value::<CreateMotorcycleCommandInput>(safe).unwrap();
    let created = handle_create_motorcycle(&fixture.database, input.clone()).unwrap();
    let duplicate = handle_create_motorcycle(
        &fixture.database,
        CreateMotorcycleCommandInput {
            model: "Duplicate".into(),
            created_at: 2_100,
            ..input
        },
    )
    .expect_err("duplicate plate should be a stable friendly error");
    let missing_customer = handle_create_motorcycle(
        &fixture.database,
        CreateMotorcycleCommandInput {
            customer_id: 999_999,
            plate_number: "29-12346".into(),
            model: "Missing owner".into(),
            created_at: 2_200,
            ..serde_json::from_value::<CreateMotorcycleCommandInput>(json!({
                "customerId": customer_id,
                "makeId": make_id,
                "model": "CB150R",
                "year": null,
                "plateNumber": "29-12345",
                "vin": null,
                "chassisNumber": null,
                "colorId": color_id,
                "notes": null,
                "createdAt": 2000
            }))
            .unwrap()
        },
    )
    .expect_err("missing Customer should remain typed");
    let invalid_reference = handle_create_motorcycle(
        &fixture.database,
        CreateMotorcycleCommandInput {
            customer_id,
            make_id: 999_999,
            model: "Invalid reference".into(),
            year: None,
            plate_number: "30-12345".into(),
            vin: Some("2HGCM82633A004352".into()),
            chassis_number: None,
            color_id,
            notes: None,
            created_at: 2_300,
        },
    )
    .expect_err("invalid reference should remain a validation error");
    let forged = serde_json::from_value::<CreateMotorcycleCommandInput>(forged);

    // # Assert
    assert_eq!(
        serde_json::to_value(created).unwrap(),
        json!({
            "id": 1,
            "makeName": "Honda",
            "model": "CB150R",
            "year": null,
            "colorName": "Black",
            "plateNumber": "29-12345",
            "vin": null,
            "chassisNumber": null,
            "activeServiceVisitId": null,
            "activeServiceVisitStatus": null
        })
    );
    assert!(forged.is_err());
    assert_eq!(
        duplicate.category,
        CommandErrorCategory::MotorcycleIdentityAlreadyExists
    );
    assert_eq!(
        duplicate.message,
        "A Motorcycle with this identity already exists."
    );
    let duplicate_message = duplicate.message.to_ascii_lowercase();
    assert!(!duplicate_message.contains("unique"));
    assert!(!duplicate_message.contains("constraint"));
    assert!(!duplicate_message.contains("sqlite"));
    assert_eq!(
        missing_customer.category,
        CommandErrorCategory::CustomerNotFound
    );
    assert_eq!(
        invalid_reference.category,
        CommandErrorCategory::ValidationError
    );
    assert_eq!(
        serde_json::to_value(duplicate).unwrap()["category"],
        "motorcycleIdentityAlreadyExists"
    );
}

struct Fixture {
    _temp_dir: TempDir,
    database: RuntimeDatabase,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(temp_dir.path()).unwrap();
    Fixture {
        _temp_dir: temp_dir,
        database,
    }
}

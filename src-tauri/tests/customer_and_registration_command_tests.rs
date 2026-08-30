use serde_json::json;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    commands::{
        customer::{handle_create_customer, CreateCustomerCommandInput},
        motorcycle_registration::handle_load_motorcycle_registration_reference_data,
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
    connection
        .execute(
            "INSERT INTO jordan_plate_codes (code, active) VALUES ('29', 1), ('Hidden', 0)",
            [],
        )
        .unwrap();
    let plate_id: i64 = connection
        .query_row(
            "SELECT id FROM jordan_plate_codes WHERE code = '29'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    // # Act
    let result = handle_load_motorcycle_registration_reference_data(&fixture.database).unwrap();

    // # Assert
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        json!({
            "makes": [{ "id": make_id, "name": "Test Make" }],
            "colors": [{ "id": color_id, "name": "Test Color" }],
            "plateCodes": [{ "id": plate_id, "code": "29" }]
        })
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

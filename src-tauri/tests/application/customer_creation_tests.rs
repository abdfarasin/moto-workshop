use rusqlite::Connection;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::customer::{
        CreateCustomerInput, CustomerApplicationError, CustomerApplicationService,
    },
    db::{migrate_database, open_database},
    domain::customer::CustomerValidationError,
};

#[test]
fn creates_customer_with_domain_normalization_exact_timestamps_and_persisted_summary() {
    // # Arrange
    let mut fixture = fixture();

    // # Act
    let local = CustomerApplicationService::new(&mut fixture.connection)
        .create_customer(CreateCustomerInput {
            name: "  Ahmad Ali  ".into(),
            phone: " 0791234567 ".into(),
            notes: Some("   ".into()),
            created_at: 1_234,
        })
        .unwrap();
    let international = CustomerApplicationService::new(&mut fixture.connection)
        .create_customer(CreateCustomerInput {
            name: "Maya Saleh".into(),
            phone: "00962791234568".into(),
            notes: Some("  Prefers WhatsApp  ".into()),
            created_at: 2_345,
        })
        .unwrap();

    // # Assert
    assert!(local.id > 0);
    assert_eq!(local.name, "Ahmad Ali");
    assert_eq!(local.phone, "+962791234567");
    assert_eq!(international.phone, "+962791234568");
    let persisted_local: (String, String, Option<String>, i64, i64) = fixture
        .connection
        .query_row(
            "SELECT name, phone, notes, created_at, updated_at FROM customers WHERE id = ?1",
            [local.id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(
        persisted_local,
        (
            "Ahmad Ali".into(),
            "+962791234567".into(),
            None,
            1_234,
            1_234
        )
    );
    let persisted_notes: Option<String> = fixture
        .connection
        .query_row(
            "SELECT notes FROM customers WHERE id = ?1",
            [international.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(persisted_notes.as_deref(), Some("Prefers WhatsApp"));
}

#[test]
fn customer_creation_returns_domain_validation_and_timestamp_errors_without_writes() {
    // # Arrange
    let mut fixture = fixture();

    // # Act
    let blank_name = create_error(&mut fixture.connection, "   ", "0791234567", None, 100);
    let invalid_name = create_error(
        &mut fixture.connection,
        "Ahmad\nAli",
        "0791234567",
        None,
        100,
    );
    let invalid_phone = create_error(&mut fixture.connection, "Ahmad", "not-a-phone", None, 100);
    let long_notes = create_error(
        &mut fixture.connection,
        "Ahmad",
        "0791234567",
        Some("x".repeat(2_001)),
        100,
    );
    let invalid_timestamp = create_error(&mut fixture.connection, "Ahmad", "0791234567", None, -1);

    // # Assert
    assert!(matches!(
        blank_name,
        CustomerApplicationError::Validation(CustomerValidationError::BlankName)
    ));
    assert!(matches!(
        invalid_name,
        CustomerApplicationError::Validation(CustomerValidationError::NameContainsControlCharacter)
    ));
    assert!(matches!(
        invalid_phone,
        CustomerApplicationError::Validation(CustomerValidationError::InvalidPhone)
    ));
    assert!(matches!(
        long_notes,
        CustomerApplicationError::Validation(CustomerValidationError::NotesTooLong)
    ));
    assert!(matches!(
        invalid_timestamp,
        CustomerApplicationError::InvalidTimestamp
    ));
    let count: i64 = fixture
        .connection
        .query_row("SELECT COUNT(*) FROM customers", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn alternate_phone_representations_collide_as_one_typed_duplicate() {
    // # Arrange
    let mut fixture = fixture();
    CustomerApplicationService::new(&mut fixture.connection)
        .create_customer(CreateCustomerInput {
            name: "Ahmad Ali".into(),
            phone: "0791234567".into(),
            notes: None,
            created_at: 100,
        })
        .unwrap();

    // # Act
    let canonical = create_error(
        &mut fixture.connection,
        "Ahmad Duplicate",
        "+962791234567",
        None,
        200,
    );
    let international = create_error(
        &mut fixture.connection,
        "Ahmad Duplicate Two",
        "00962791234567",
        None,
        300,
    );

    // # Assert
    assert!(matches!(
        canonical,
        CustomerApplicationError::PhoneAlreadyExists
    ));
    assert!(matches!(
        international,
        CustomerApplicationError::PhoneAlreadyExists
    ));
    let count: i64 = fixture
        .connection
        .query_row("SELECT COUNT(*) FROM customers", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

fn create_error(
    connection: &mut Connection,
    name: &str,
    phone: &str,
    notes: Option<String>,
    created_at: i64,
) -> CustomerApplicationError {
    CustomerApplicationService::new(connection)
        .create_customer(CreateCustomerInput {
            name: name.into(),
            phone: phone.into(),
            notes,
            created_at,
        })
        .expect_err("input should be rejected")
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("customer-create-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

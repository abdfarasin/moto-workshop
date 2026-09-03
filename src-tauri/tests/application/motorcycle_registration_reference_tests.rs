use rusqlite::Connection;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::motorcycle_registration::{
        CreateMotorcycleInput, MotorcycleRegistrationError, MotorcycleRegistrationReference,
        MotorcycleRegistrationService,
    },
    db::{migrate_database, open_database},
    domain::motorcycle::{
        ChassisNumberValidationError, MotorcycleValidationError, PlateNumberValidationError,
        VinValidationError,
    },
};

#[test]
fn loads_only_active_make_and_color_catalogs_in_deterministic_order_without_mutation() {
    // # Arrange
    let mut fixture = fixture();
    fixture
        .connection
        .execute(
            "UPDATE motorcycle_makes SET active = 0 WHERE name = 'Honda'",
            [],
        )
        .unwrap();
    fixture
        .connection
        .execute(
            "UPDATE motorcycle_colors SET active = 0 WHERE name = 'Black'",
            [],
        )
        .unwrap();
    let changes_before = total_changes(&fixture.connection);

    // # Act
    let catalogs = MotorcycleRegistrationService::new(&mut fixture.connection)
        .load_reference_data()
        .unwrap();

    // # Assert
    assert!(!catalogs.makes.is_empty());
    assert!(!catalogs.colors.is_empty());
    assert!(catalogs
        .makes
        .iter()
        .all(|make| make.name != "Honda" && make.id > 0));
    assert!(catalogs
        .colors
        .iter()
        .all(|color| color.name != "Black" && color.id > 0));
    assert_case_insensitive_order(
        &catalogs
            .makes
            .iter()
            .map(|make| make.name.as_str())
            .collect::<Vec<_>>(),
    );
    assert_case_insensitive_order(
        &catalogs
            .colors
            .iter()
            .map(|color| color.name.as_str())
            .collect::<Vec<_>>(),
    );
    assert_eq!(total_changes(&fixture.connection), changes_before);
}

#[test]
fn creates_motorcycle_from_domain_normalized_values_and_returns_string_plate() {
    // # Arrange
    let mut fixture = creation_fixture();
    let current_year = backend_current_year(&fixture.connection);

    // # Act
    let created = MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(CreateMotorcycleInput {
            customer_id: fixture.customer_id,
            make_id: fixture.make_id,
            model: "  CB150R  ".into(),
            year: Some(current_year + 1),
            plate_number: "  29-00001  ".into(),
            vin: Some("1hgcm82633a004352".into()),
            chassis_number: Some(" frame-abc/1 ".into()),
            color_id: fixture.color_id,
            notes: Some("  Customer notes  ".into()),
            created_at: 2_000,
        })
        .unwrap();

    // # Assert
    assert_eq!(created.model, "CB150R");
    assert_eq!(created.year, Some(i64::from(current_year + 1)));
    assert_eq!(created.plate_number.as_deref(), Some("29-00001"));
    assert_eq!(created.vin.as_deref(), Some("1HGCM82633A004352"));
    assert_eq!(created.chassis_number.as_deref(), Some("FRAME-ABC/1"));
    assert_eq!(created.make_name, "Honda");
    assert_eq!(created.color_name, "Black");
    let persisted: (Option<String>, i64, i64, Option<i64>) = fixture
        .connection
        .query_row(
            "SELECT notes, created_at, updated_at, archived_at FROM motorcycles WHERE id = ?1",
            [created.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        persisted,
        (Some("Customer notes".into()), 2_000, 2_000, None)
    );
}

#[test]
fn creation_rejects_missing_or_inactive_customer_make_and_color_without_partial_rows() {
    // # Arrange
    let mut fixture = creation_fixture();
    let archived_customer_id = insert_customer(&fixture.connection, "+962791111112");
    fixture
        .connection
        .execute(
            "UPDATE customers SET archived_at = 1500 WHERE id = ?1",
            [archived_customer_id],
        )
        .unwrap();
    let inactive_make_id = insert_make(&fixture.connection, "Inactive Make", 0);
    let inactive_color_id = insert_color(&fixture.connection, "Inactive Color", 0);

    // # Act
    let missing_customer = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        customer_id: 999_999,
        ..valid_input(fixture, "1", 2_000)
    });
    let archived_customer = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        customer_id: archived_customer_id,
        ..valid_input(fixture, "2", 2_000)
    });
    let missing_make = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        make_id: 999_999,
        ..valid_input(fixture, "3", 2_000)
    });
    let inactive_make = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        make_id: inactive_make_id,
        ..valid_input(fixture, "4", 2_000)
    });
    let missing_color = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        color_id: 999_999,
        ..valid_input(fixture, "5", 2_000)
    });
    let inactive_color = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        color_id: inactive_color_id,
        ..valid_input(fixture, "6", 2_000)
    });

    // # Assert
    assert!(matches!(
        missing_customer,
        MotorcycleRegistrationError::CustomerNotFound(999_999)
    ));
    assert!(
        matches!(archived_customer, MotorcycleRegistrationError::CustomerNotFound(id) if id == archived_customer_id)
    );
    for error in [missing_make, inactive_make] {
        assert!(matches!(
            error,
            MotorcycleRegistrationError::InvalidReference(MotorcycleRegistrationReference::Make)
        ));
    }
    for error in [missing_color, inactive_color] {
        assert!(matches!(
            error,
            MotorcycleRegistrationError::InvalidReference(MotorcycleRegistrationReference::Color)
        ));
    }
    assert_eq!(motorcycle_count(&fixture.connection), 0);
}

#[test]
fn creation_delegates_plate_vin_chassis_year_and_timestamp_validation() {
    // # Arrange
    let mut fixture = creation_fixture();
    let current_year = backend_current_year(&fixture.connection);

    // # Act
    let blank_plate =
        create_fixture_error(&mut fixture, |fixture| valid_input(fixture, "   ", 2_000));
    let letter_plate =
        create_fixture_error(&mut fixture, |fixture| valid_input(fixture, "ABC", 2_000));
    let malformed_plate = create_fixture_error(&mut fixture, |fixture| {
        valid_input(fixture, "12--34", 2_000)
    });
    let invalid_vin = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        vin: Some("INVALID".into()),
        ..valid_input(fixture, "7", 2_000)
    });
    let invalid_chassis = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        chassis_number: Some("bad chassis".into()),
        ..valid_input(fixture, "8", 2_000)
    });
    let future_year = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        year: Some(current_year + 2),
        ..valid_input(fixture, "9", 2_000)
    });
    let invalid_timestamp =
        create_fixture_error(&mut fixture, |fixture| valid_input(fixture, "10", -1));

    // # Assert
    assert_domain_error(
        blank_plate,
        MotorcycleValidationError::InvalidPlateNumber(PlateNumberValidationError::Blank),
    );
    assert_domain_error(
        letter_plate,
        MotorcycleValidationError::InvalidPlateNumber(PlateNumberValidationError::InvalidCharacter),
    );
    assert_domain_error(
        malformed_plate,
        MotorcycleValidationError::InvalidPlateNumber(PlateNumberValidationError::InvalidFormat),
    );
    assert_domain_error(
        invalid_vin,
        MotorcycleValidationError::InvalidVin(VinValidationError::InvalidLength),
    );
    assert_domain_error(
        invalid_chassis,
        MotorcycleValidationError::InvalidChassisNumber(
            ChassisNumberValidationError::InvalidCharacter,
        ),
    );
    assert_domain_error(future_year, MotorcycleValidationError::InvalidYear);
    assert!(matches!(
        invalid_timestamp,
        MotorcycleRegistrationError::InvalidTimestamp
    ));
    assert_eq!(motorcycle_count(&fixture.connection), 0);
}

#[test]
fn duplicate_plate_vin_and_chassis_are_reported_as_identity_collisions() {
    // # Arrange
    let mut fixture = creation_fixture();
    let seed = CreateMotorcycleInput {
        vin: Some("1HGCM82633A004352".into()),
        chassis_number: Some("FRAME-42".into()),
        ..valid_input(&fixture, "42", 2_000)
    };
    MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(seed)
        .unwrap();

    // # Act
    let duplicate_plate =
        create_fixture_error(&mut fixture, |fixture| valid_input(fixture, "42", 2_100));
    let duplicate_vin = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        vin: Some("1hgcm82633a004352".into()),
        ..valid_input(fixture, "43", 2_200)
    });
    let duplicate_chassis = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        chassis_number: Some(" frame-42 ".into()),
        ..valid_input(fixture, "44", 2_300)
    });

    // # Assert
    for error in [duplicate_plate, duplicate_vin, duplicate_chassis] {
        assert!(matches!(
            error,
            MotorcycleRegistrationError::IdentityAlreadyExists
        ));
    }
    assert_eq!(motorcycle_count(&fixture.connection), 1);
}

fn assert_case_insensitive_order(values: &[&str]) {
    assert!(values
        .windows(2)
        .all(|pair| pair[0].to_ascii_lowercase() <= pair[1].to_ascii_lowercase()));
}

fn total_changes(connection: &Connection) -> i64 {
    connection
        .query_row("SELECT total_changes()", [], |row| row.get(0))
        .unwrap()
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("reference-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

struct CreationFixture {
    _temp_dir: TempDir,
    connection: Connection,
    customer_id: i64,
    make_id: i64,
    color_id: i64,
}

fn creation_fixture() -> CreationFixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("create-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    let customer_id = insert_customer(&connection, "+962791111111");
    let make_id = catalog_id(&connection, "motorcycle_makes", "Honda");
    let color_id = catalog_id(&connection, "motorcycle_colors", "Black");
    CreationFixture {
        _temp_dir: temp_dir,
        connection,
        customer_id,
        make_id,
        color_id,
    }
}

fn valid_input(
    fixture: &CreationFixture,
    plate_number: &str,
    created_at: i64,
) -> CreateMotorcycleInput {
    CreateMotorcycleInput {
        customer_id: fixture.customer_id,
        make_id: fixture.make_id,
        model: "Test Model".into(),
        year: None,
        plate_number: plate_number.into(),
        vin: None,
        chassis_number: None,
        color_id: fixture.color_id,
        notes: None,
        created_at,
    }
}

fn create_fixture_error(
    fixture: &mut CreationFixture,
    input: impl FnOnce(&CreationFixture) -> CreateMotorcycleInput,
) -> MotorcycleRegistrationError {
    let input = input(fixture);
    MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(input)
        .expect_err("creation should fail")
}

fn assert_domain_error(error: MotorcycleRegistrationError, expected: MotorcycleValidationError) {
    assert!(matches!(error, MotorcycleRegistrationError::Validation(actual) if actual == expected));
}

fn backend_current_year(connection: &Connection) -> i32 {
    connection
        .query_row(
            "SELECT CAST(strftime('%Y', 'now', 'localtime') AS INTEGER)",
            [],
            |row| row.get(0),
        )
        .unwrap()
}

fn motorcycle_count(connection: &Connection) -> i64 {
    connection
        .query_row("SELECT COUNT(*) FROM motorcycles", [], |row| row.get(0))
        .unwrap()
}

fn insert_customer(connection: &Connection, phone: &str) -> i64 {
    connection.execute(
        "INSERT INTO customers (name, phone, created_at, updated_at) VALUES ('Motorcycle Owner', ?1, 1000, 1000)",
        [phone],
    ).unwrap();
    connection.last_insert_rowid()
}

fn catalog_id(connection: &Connection, table: &str, name: &str) -> i64 {
    connection
        .query_row(
            &format!("SELECT id FROM {table} WHERE name = ?1"),
            [name],
            |row| row.get(0),
        )
        .unwrap()
}

fn insert_make(connection: &Connection, name: &str, active: i64) -> i64 {
    connection
        .execute(
            "INSERT INTO motorcycle_makes (name, active) VALUES (?1, ?2)",
            (name, active),
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_color(connection: &Connection, name: &str, active: i64) -> i64 {
    connection
        .execute(
            "INSERT INTO motorcycle_colors (name, active) VALUES (?1, ?2)",
            (name, active),
        )
        .unwrap();
    connection.last_insert_rowid()
}

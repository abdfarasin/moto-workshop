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
fn loads_only_active_registration_catalogs_in_deterministic_order_without_mutation() {
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
    fixture
        .connection
        .execute(
            "INSERT INTO jordan_plate_codes (code, active) VALUES
             ('Z-Plate', 1), ('a-plate', 1), ('Inactive', 0)",
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
    assert!(catalogs.makes.iter().all(|make| make.name != "Honda"));
    assert!(catalogs.colors.iter().all(|color| color.name != "Black"));
    assert_eq!(
        catalogs
            .plate_codes
            .iter()
            .map(|plate| plate.code.as_str())
            .collect::<Vec<_>>(),
        vec!["a-plate", "Z-Plate"]
    );
    assert_case_insensitive_name_order(
        &catalogs
            .makes
            .iter()
            .map(|make| make.name.as_str())
            .collect::<Vec<_>>(),
    );
    assert_case_insensitive_name_order(
        &catalogs
            .colors
            .iter()
            .map(|color| color.name.as_str())
            .collect::<Vec<_>>(),
    );
    assert!(catalogs.makes.iter().all(|make| make.id > 0));
    assert!(catalogs.colors.iter().all(|color| color.id > 0));
    assert!(catalogs.plate_codes.iter().all(|plate| plate.id > 0));
    assert_eq!(total_changes(&fixture.connection), changes_before);
}

#[test]
fn creates_plate_vin_chassis_and_combined_motorcycles_from_domain_normalized_values() {
    // # Arrange
    let mut fixture = creation_fixture();
    let current_year = backend_current_year(&fixture.connection);

    // # Act
    let plate_only = MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(CreateMotorcycleInput {
            customer_id: fixture.customer_id,
            make_id: fixture.make_id,
            model: "  CB150R  ".into(),
            year: Some(current_year + 1),
            plate_code_id: Some(fixture.plate_code_id),
            plate_number: Some("00001".into()),
            vin: None,
            chassis_number: None,
            color_id: fixture.color_id,
            notes: Some("   ".into()),
            created_at: 2_000,
        })
        .unwrap();
    let vin_only_input = valid_input(&fixture, IdentityInput::Vin("1hgcm82633a004352"), 2_100);
    let vin_only = MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(vin_only_input)
        .unwrap();
    let chassis_only_input = valid_input(&fixture, IdentityInput::Chassis(" frame-abc/1 "), 2_200);
    let chassis_only = MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(chassis_only_input)
        .unwrap();
    let combined = MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(CreateMotorcycleInput {
            customer_id: fixture.customer_id,
            make_id: fixture.make_id,
            model: "Combined".into(),
            year: None,
            plate_code_id: Some(fixture.second_plate_code_id),
            plate_number: Some("99999".into()),
            vin: Some("JH2RC4468MK123456".into()),
            chassis_number: Some(" combined.2 ".into()),
            color_id: fixture.color_id,
            notes: Some("  Customer notes  ".into()),
            created_at: 2_300,
        })
        .unwrap();

    // # Assert
    assert_eq!(plate_only.model, "CB150R");
    assert_eq!(plate_only.year, Some(i64::from(current_year + 1)));
    assert_eq!(plate_only.plate_code.as_deref(), Some("29"));
    assert_eq!(plate_only.plate_number, Some(1));
    assert_eq!(plate_only.vin, None);
    assert_eq!(plate_only.chassis_number, None);
    assert_eq!(plate_only.make_name, "Honda");
    assert_eq!(plate_only.color_name, "Black");
    assert_eq!(plate_only.active_service_visit_id, None);
    assert_eq!(plate_only.active_service_visit_status, None);
    assert_eq!(vin_only.vin.as_deref(), Some("1HGCM82633A004352"));
    assert_eq!(chassis_only.chassis_number.as_deref(), Some("FRAME-ABC/1"));
    assert_eq!(combined.chassis_number.as_deref(), Some("COMBINED.2"));
    assert_eq!(combined.plate_number, Some(99_999));
    let persisted: (Option<String>, i64, i64, Option<i64>) = fixture
        .connection
        .query_row(
            "SELECT notes, created_at, updated_at, archived_at
             FROM motorcycles WHERE id = ?1",
            [plate_only.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(persisted, (None, 2_000, 2_000, None));
    let combined_notes: Option<String> = fixture
        .connection
        .query_row(
            "SELECT notes FROM motorcycles WHERE id = ?1",
            [combined.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(combined_notes.as_deref(), Some("Customer notes"));
}

#[test]
fn creation_rejects_missing_or_inactive_customer_and_references_without_partial_rows() {
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
    let inactive_plate_id = insert_plate_code(&fixture.connection, "Inactive", 0);

    // # Act
    let missing_customer = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        customer_id: 999_999,
        ..valid_input(fixture, IdentityInput::Vin("1HGCM82633A004352"), 2_000)
    });
    let archived_customer = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        customer_id: archived_customer_id,
        ..valid_input(fixture, IdentityInput::Vin("2HGCM82633A004352"), 2_000)
    });
    let missing_make = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        make_id: 999_999,
        ..valid_input(fixture, IdentityInput::Vin("3HGCM82633A004352"), 2_000)
    });
    let inactive_make = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        make_id: inactive_make_id,
        ..valid_input(fixture, IdentityInput::Vin("4HGCM82633A004352"), 2_000)
    });
    let missing_color = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        color_id: 999_999,
        ..valid_input(fixture, IdentityInput::Vin("5HGCM82633A004352"), 2_000)
    });
    let inactive_color = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        color_id: inactive_color_id,
        ..valid_input(fixture, IdentityInput::Vin("6HGCM82633A004352"), 2_000)
    });
    let missing_plate = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        plate_code_id: Some(999_999),
        plate_number: Some("10".into()),
        vin: None,
        ..valid_input(fixture, IdentityInput::Vin("7HGCM82633A004352"), 2_000)
    });
    let inactive_plate = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        plate_code_id: Some(inactive_plate_id),
        plate_number: Some("10".into()),
        vin: None,
        ..valid_input(fixture, IdentityInput::Vin("8HGCM82633A004352"), 2_000)
    });

    // # Assert
    assert!(matches!(
        missing_customer,
        MotorcycleRegistrationError::CustomerNotFound(999_999)
    ));
    assert!(matches!(
        archived_customer,
        MotorcycleRegistrationError::CustomerNotFound(id) if id == archived_customer_id
    ));
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
    for error in [missing_plate, inactive_plate] {
        assert!(matches!(
            error,
            MotorcycleRegistrationError::InvalidReference(
                MotorcycleRegistrationReference::PlateCode
            )
        ));
    }
    assert_eq!(motorcycle_count(&fixture.connection), 0);
}

#[test]
fn creation_delegates_identity_year_and_plate_boundaries_to_the_domain() {
    // # Arrange
    let mut fixture = creation_fixture();
    let current_year = backend_current_year(&fixture.connection);

    // # Act
    let incomplete_plate = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        plate_code_id: Some(fixture.plate_code_id),
        plate_number: None,
        ..valid_input(fixture, IdentityInput::Vin("1HGCM82633A004352"), 2_000)
    });
    let nonnumeric_plate =
        create_fixture_error(&mut fixture, |fixture| plate_input(fixture, "ABC", 2_000));
    let low_plate = create_fixture_error(&mut fixture, |fixture| plate_input(fixture, "0", 2_000));
    let high_plate = create_fixture_error(&mut fixture, |fixture| {
        plate_input(fixture, "100000", 2_000)
    });
    let missing_identity = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        plate_code_id: None,
        plate_number: None,
        vin: None,
        chassis_number: None,
        ..valid_input(fixture, IdentityInput::Vin("1HGCM82633A004352"), 2_000)
    });
    let invalid_vin = create_fixture_error(&mut fixture, |fixture| {
        valid_input(fixture, IdentityInput::Vin("INVALID"), 2_000)
    });
    let invalid_chassis = create_fixture_error(&mut fixture, |fixture| {
        valid_input(fixture, IdentityInput::Chassis("bad chassis"), 2_000)
    });
    let future_year = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        year: Some(current_year + 2),
        ..valid_input(fixture, IdentityInput::Vin("2HGCM82633A004352"), 2_000)
    });
    let invalid_timestamp = create_fixture_error(&mut fixture, |fixture| {
        valid_input(fixture, IdentityInput::Vin("3HGCM82633A004352"), -1)
    });

    // # Assert
    assert_domain_error(incomplete_plate, MotorcycleValidationError::IncompletePlate);
    assert_domain_error(
        nonnumeric_plate,
        MotorcycleValidationError::InvalidPlateNumber(PlateNumberValidationError::NonNumeric),
    );
    assert_domain_error(
        low_plate,
        MotorcycleValidationError::InvalidPlateNumber(PlateNumberValidationError::OutOfRange),
    );
    assert_domain_error(
        high_plate,
        MotorcycleValidationError::InvalidPlateNumber(PlateNumberValidationError::OutOfRange),
    );
    assert_domain_error(missing_identity, MotorcycleValidationError::MissingIdentity);
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
fn duplicate_identities_are_typed_while_same_number_under_another_plate_code_is_valid() {
    // # Arrange
    let mut fixture = creation_fixture();
    let seed_input = CreateMotorcycleInput {
        plate_code_id: Some(fixture.plate_code_id),
        plate_number: Some("42".into()),
        vin: Some("1HGCM82633A004352".into()),
        chassis_number: Some("FRAME-42".into()),
        ..valid_input(&fixture, IdentityInput::Vin("1HGCM82633A004352"), 2_000)
    };
    MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(seed_input)
        .unwrap();

    // # Act
    let duplicate_vin = create_fixture_error(&mut fixture, |fixture| {
        valid_input(fixture, IdentityInput::Vin("1hgcm82633a004352"), 2_100)
    });
    let duplicate_chassis = create_fixture_error(&mut fixture, |fixture| {
        valid_input(fixture, IdentityInput::Chassis(" frame-42 "), 2_200)
    });
    let duplicate_plate = create_fixture_error(&mut fixture, |fixture| CreateMotorcycleInput {
        plate_code_id: Some(fixture.plate_code_id),
        plate_number: Some("42".into()),
        ..valid_input(fixture, IdentityInput::Vin("2HGCM82633A004352"), 2_300)
    });
    let different_code_input = CreateMotorcycleInput {
        plate_code_id: Some(fixture.second_plate_code_id),
        plate_number: Some("42".into()),
        ..valid_input(&fixture, IdentityInput::Vin("3HGCM82633A004352"), 2_400)
    };
    let different_code = MotorcycleRegistrationService::new(&mut fixture.connection)
        .create_motorcycle(different_code_input)
        .unwrap();

    // # Assert
    assert!(matches!(
        duplicate_vin,
        MotorcycleRegistrationError::IdentityAlreadyExists
    ));
    assert!(matches!(
        duplicate_chassis,
        MotorcycleRegistrationError::IdentityAlreadyExists
    ));
    assert!(matches!(
        duplicate_plate,
        MotorcycleRegistrationError::IdentityAlreadyExists
    ));
    assert_eq!(different_code.plate_code.as_deref(), Some("30"));
    assert_eq!(different_code.plate_number, Some(42));
    assert_eq!(motorcycle_count(&fixture.connection), 2);
}

fn assert_case_insensitive_name_order(values: &[&str]) {
    assert!(values
        .windows(2)
        .all(|pair| { pair[0].to_ascii_lowercase() <= pair[1].to_ascii_lowercase() }));
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
    let mut connection =
        open_database(temp_dir.path().join("registration-reference-test.db")).unwrap();
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
    plate_code_id: i64,
    second_plate_code_id: i64,
}

fn creation_fixture() -> CreationFixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("motorcycle-create-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    let customer_id = insert_customer(&connection, "+962791111111");
    let make_id = catalog_id(&connection, "motorcycle_makes", "Honda");
    let color_id = catalog_id(&connection, "motorcycle_colors", "Black");
    let plate_code_id = insert_plate_code(&connection, "29", 1);
    let second_plate_code_id = insert_plate_code(&connection, "30", 1);
    CreationFixture {
        _temp_dir: temp_dir,
        connection,
        customer_id,
        make_id,
        color_id,
        plate_code_id,
        second_plate_code_id,
    }
}

enum IdentityInput<'value> {
    Vin(&'value str),
    Chassis(&'value str),
}

fn valid_input(
    fixture: &CreationFixture,
    identity: IdentityInput<'_>,
    created_at: i64,
) -> CreateMotorcycleInput {
    let (vin, chassis_number) = match identity {
        IdentityInput::Vin(value) => (Some(value.into()), None),
        IdentityInput::Chassis(value) => (None, Some(value.into())),
    };
    CreateMotorcycleInput {
        customer_id: fixture.customer_id,
        make_id: fixture.make_id,
        model: "Test Model".into(),
        year: None,
        plate_code_id: None,
        plate_number: None,
        vin,
        chassis_number,
        color_id: fixture.color_id,
        notes: None,
        created_at,
    }
}

fn plate_input(
    fixture: &CreationFixture,
    plate_number: &str,
    created_at: i64,
) -> CreateMotorcycleInput {
    CreateMotorcycleInput {
        plate_code_id: Some(fixture.plate_code_id),
        plate_number: Some(plate_number.into()),
        vin: None,
        chassis_number: None,
        ..valid_input(fixture, IdentityInput::Vin("1HGCM82633A004352"), created_at)
    }
}

fn create_error(
    connection: &mut Connection,
    input: CreateMotorcycleInput,
) -> MotorcycleRegistrationError {
    MotorcycleRegistrationService::new(connection)
        .create_motorcycle(input)
        .expect_err("Motorcycle creation should fail")
}

fn create_fixture_error(
    fixture: &mut CreationFixture,
    input: impl FnOnce(&CreationFixture) -> CreateMotorcycleInput,
) -> MotorcycleRegistrationError {
    let input = input(fixture);
    create_error(&mut fixture.connection, input)
}

fn assert_domain_error(error: MotorcycleRegistrationError, expected: MotorcycleValidationError) {
    assert!(matches!(
        error,
        MotorcycleRegistrationError::Validation(actual) if actual == expected
    ));
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
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES ('Motorcycle Owner', ?1, 1000, 1000)",
            [phone],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn catalog_id(connection: &Connection, table: &str, name: &str) -> i64 {
    let sql = format!("SELECT id FROM {table} WHERE name = ?1");
    connection
        .query_row(&sql, [name], |row| row.get(0))
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

fn insert_plate_code(connection: &Connection, code: &str, active: i64) -> i64 {
    connection
        .execute(
            "INSERT INTO jordan_plate_codes (code, active) VALUES (?1, ?2)",
            (code, active),
        )
        .unwrap();
    connection.last_insert_rowid()
}

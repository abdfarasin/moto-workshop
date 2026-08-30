use rusqlite::Connection;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::motorcycle_registration::MotorcycleRegistrationService,
    db::{migrate_database, open_database},
};

#[test]
fn loads_only_active_registration_catalogs_in_deterministic_order_without_mutation() {
    // # Arrange
    let fixture = fixture();
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
    let catalogs = MotorcycleRegistrationService::new(&fixture.connection)
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

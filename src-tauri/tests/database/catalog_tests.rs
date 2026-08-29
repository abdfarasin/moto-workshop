use rusqlite::Connection;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::db::{migrate_database, open_database};

#[test]
fn motorcycle_make_catalog_contains_the_initial_seed_data() {
    // # Arrange
    let (_temp_dir, connection) = migrated_database();
    let expected = vec![
        "Aprilia",
        "Bajaj",
        "Benelli",
        "Beta",
        "BMW",
        "BSA",
        "CFMOTO",
        "Ducati",
        "GasGas",
        "Harley-Davidson",
        "Hero",
        "Honda",
        "Husqvarna",
        "Indian",
        "Kawasaki",
        "Keeway",
        "KTM",
        "Kymco",
        "Lifan",
        "Loncin",
        "Moto Guzzi",
        "MV Agusta",
        "Piaggio",
        "QJMotor",
        "Royal Enfield",
        "Sherco",
        "Suzuki",
        "SYM",
        "Triumph",
        "TVS",
        "Vespa",
        "Voge",
        "Yamaha",
        "Zontes",
    ];

    // # Act
    let actual = catalog_names(&connection, "motorcycle_makes");

    // # Assert
    assert_eq!(actual, expected);
}

#[test]
fn motorcycle_color_catalog_contains_the_initial_seed_data() {
    // # Arrange
    let (_temp_dir, connection) = migrated_database();
    let expected = vec![
        "Black",
        "White",
        "Gray",
        "Silver",
        "Red",
        "Blue",
        "Green",
        "Yellow",
        "Orange",
        "Brown",
        "Beige",
        "Gold",
        "Purple",
        "Pink",
        "Bronze",
        "Maroon",
        "Multicolor",
    ];

    // # Act
    let actual = catalog_names(&connection, "motorcycle_colors");

    // # Assert
    assert_eq!(actual, expected);
}

#[test]
fn make_and_color_names_are_unique_case_insensitively() {
    // # Arrange
    let (_temp_dir, connection) = migrated_database();

    // # Act
    let duplicate_make =
        connection.execute("INSERT INTO motorcycle_makes (name) VALUES ('yamaha')", []);
    let duplicate_color =
        connection.execute("INSERT INTO motorcycle_colors (name) VALUES ('black')", []);

    // # Assert
    assert!(duplicate_make.is_err());
    assert!(duplicate_color.is_err());
}

#[test]
fn catalog_active_flags_accept_only_zero_or_one() {
    // # Arrange
    let (_temp_dir, connection) = migrated_database();

    // # Act
    let invalid_make = connection.execute(
        "INSERT INTO motorcycle_makes (name, active) VALUES ('Test Make', 2)",
        [],
    );
    let invalid_color = connection.execute(
        "INSERT INTO motorcycle_colors (name, active) VALUES ('Test Color', -1)",
        [],
    );
    let invalid_plate_code = connection.execute(
        "INSERT INTO jordan_plate_codes (code, active) VALUES ('A', 7)",
        [],
    );

    // # Assert
    assert!(invalid_make.is_err());
    assert!(invalid_color.is_err());
    assert!(invalid_plate_code.is_err());
}

#[test]
fn catalog_names_are_nonblank_trimmed_and_bounded() {
    // # Arrange
    let (_temp_dir, connection) = migrated_database();

    let invalid_statements = [
        "INSERT INTO motorcycle_makes (name) VALUES ('   ')",
        "INSERT INTO motorcycle_makes (name) VALUES (' Untrimmed')",
        "INSERT INTO motorcycle_colors (name) VALUES ('')",
        "INSERT INTO jordan_plate_codes (code) VALUES ('   ')",
    ];

    for statement in invalid_statements {
        // # Act
        let result = connection.execute(statement, []);

        // # Assert
        assert!(result.is_err(), "statement should fail: {statement}");
    }

    let make_too_long = "M".repeat(81);
    let color_too_long = "C".repeat(41);
    let code_too_long = "P".repeat(21);

    assert!(connection
        .execute(
            "INSERT INTO motorcycle_makes (name) VALUES (?1)",
            [&make_too_long]
        )
        .is_err());
    assert!(connection
        .execute(
            "INSERT INTO motorcycle_colors (name) VALUES (?1)",
            [&color_too_long]
        )
        .is_err());
    assert!(connection
        .execute(
            "INSERT INTO jordan_plate_codes (code) VALUES (?1)",
            [&code_too_long]
        )
        .is_err());
}

fn migrated_database() -> (TempDir, Connection) {
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    migrate_database(&mut connection).expect("database should migrate");
    (temp_dir, connection)
}

fn catalog_names(connection: &Connection, table_name: &str) -> Vec<String> {
    let statement = format!("SELECT name FROM {table_name} ORDER BY id");
    let mut query = connection
        .prepare(&statement)
        .expect("catalog query should prepare");
    query
        .query_map([], |row| row.get(0))
        .expect("catalog names should be queryable")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("catalog names should be readable")
}

use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::db::{migrate_database, open_database};

const VIN_ONE: &str = "1HGCM82633A004352";

#[test]
fn valid_string_plate_persists_and_ids_are_generated() {
    // # Arrange
    let fixture = fixture();
    let first = fixture.valid_motorcycle("47-122132");
    let second = fixture.valid_motorcycle("123");

    // # Act
    insert_motorcycle(&fixture.connection, &first).unwrap();
    let first_id = fixture.connection.last_insert_rowid();
    insert_motorcycle(&fixture.connection, &second).unwrap();
    let second_id = fixture.connection.last_insert_rowid();

    // # Assert
    let persisted: String = fixture
        .connection
        .query_row(
            "SELECT plate_number FROM motorcycles WHERE id = ?1",
            [first_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(persisted, "47-122132");
    assert_eq!((first_id, second_id), (1, 2));
}

#[test]
fn duplicate_plate_is_rejected() {
    // # Arrange
    let fixture = fixture();
    let motorcycle = fixture.valid_motorcycle("29-12345");
    insert_motorcycle(&fixture.connection, &motorcycle).unwrap();

    // # Act
    let result = insert_motorcycle(&fixture.connection, &motorcycle);

    // # Assert
    assert!(result.is_err());
}

#[test]
fn malformed_and_null_new_plates_are_rejected() {
    // # Arrange
    let cases: [Option<&str>; 7] = [
        None,
        Some("ABC123"),
        Some("12 A 34"),
        Some("-123"),
        Some("123-"),
        Some("12--34"),
        Some(" 123 "),
    ];

    for plate_number in cases {
        let fixture = fixture();
        let motorcycle = fixture.valid_motorcycle(plate_number.unwrap_or("unused"));

        // # Act
        let result = insert_motorcycle_with_plate(&fixture.connection, &motorcycle, plate_number);

        // # Assert
        assert!(
            result.is_err(),
            "plate should be rejected: {plate_number:?}"
        );
    }
}

#[test]
fn plate_has_no_artificial_length_or_numeric_range_limit() {
    // # Arrange
    let fixture = fixture();
    let motorcycle = fixture.valid_motorcycle("1234567890-1234567890-1234567890");

    // # Act
    let result = insert_motorcycle(&fixture.connection, &motorcycle);

    // # Assert
    assert!(result.is_ok());
}

#[test]
fn vin_and_chassis_are_optional_when_valid_plate_exists() {
    // # Arrange
    let fixture = fixture();
    let motorcycle = fixture.valid_motorcycle("4712213");

    // # Act
    let result = insert_motorcycle(&fixture.connection, &motorcycle);

    // # Assert
    assert!(result.is_ok());
}

#[test]
fn vin_and_chassis_remain_unique_when_present() {
    // # Arrange
    let fixture = fixture();
    let mut first = fixture.valid_motorcycle("1");
    first.vin = Some(VIN_ONE.to_string());
    first.chassis_number = Some("FRAME/12345".to_string());
    insert_motorcycle(&fixture.connection, &first).unwrap();
    let mut duplicate_vin = fixture.valid_motorcycle("2");
    duplicate_vin.vin = first.vin.clone();
    let mut duplicate_chassis = fixture.valid_motorcycle("3");
    duplicate_chassis.chassis_number = first.chassis_number.clone();

    // # Act
    let vin_result = insert_motorcycle(&fixture.connection, &duplicate_vin);
    let chassis_result = insert_motorcycle(&fixture.connection, &duplicate_chassis);

    // # Assert
    assert!(vin_result.is_err());
    assert!(chassis_result.is_err());
}

#[test]
fn malformed_vin_and_chassis_are_rejected_by_database_constraints() {
    // # Arrange
    let fixture = fixture();
    let mut invalid_vin = fixture.valid_motorcycle("4");
    invalid_vin.vin = Some("1hgcm82633a004352".to_string());
    let mut invalid_chassis = fixture.valid_motorcycle("5");
    invalid_chassis.chassis_number = Some("ABC 123".to_string());

    // # Act
    let vin_result = insert_motorcycle(&fixture.connection, &invalid_vin);
    let chassis_result = insert_motorcycle(&fixture.connection, &invalid_chassis);

    // # Assert
    assert!(vin_result.is_err());
    assert!(chassis_result.is_err());
}

#[test]
fn nonexistent_customer_make_and_color_are_rejected() {
    // # Arrange
    for field in ["customer", "make", "color"] {
        let fixture = fixture();
        let mut motorcycle = fixture.valid_motorcycle("6");
        match field {
            "customer" => motorcycle.customer_id = 999_001,
            "make" => motorcycle.make_id = 999_002,
            "color" => motorcycle.color_id = 999_003,
            _ => unreachable!(),
        }

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(result.is_err(), "nonexistent {field} should be rejected");
    }
}

#[test]
fn model_year_and_notes_constraints_remain_enforced() {
    // # Arrange
    let fixture = fixture();
    let mut blank_model = fixture.valid_motorcycle("7");
    blank_model.model = "   ".to_string();
    let mut old_year = fixture.valid_motorcycle("8");
    old_year.year = Some(1884);
    let mut long_notes = fixture.valid_motorcycle("9");
    long_notes.notes = Some("N".repeat(2001));

    // # Act
    let model_result = insert_motorcycle(&fixture.connection, &blank_model);
    let year_result = insert_motorcycle(&fixture.connection, &old_year);
    let notes_result = insert_motorcycle(&fixture.connection, &long_notes);

    // # Assert
    assert!(model_result.is_err());
    assert!(year_result.is_err());
    assert!(notes_result.is_err());
}

#[test]
fn customer_make_and_color_relationships_are_protected() {
    // # Arrange
    let fixture = fixture();
    insert_motorcycle(&fixture.connection, &fixture.valid_motorcycle("10")).unwrap();

    // # Act / # Assert
    for (table, id) in [
        ("customers", fixture.customer_id),
        ("motorcycle_makes", fixture.make_id),
        ("motorcycle_colors", fixture.color_id),
    ] {
        let result = fixture
            .connection
            .execute(&format!("DELETE FROM {table} WHERE id = ?1"), [id]);
        assert!(
            result.is_err(),
            "referenced {table} row should be protected"
        );
    }
}

#[test]
fn customer_and_make_relationship_columns_are_indexed() {
    // # Arrange
    let fixture = fixture();

    for column in ["customer_id", "make_id"] {
        // # Act
        let indexed = motorcycle_column_is_indexed(&fixture.connection, column);

        // # Assert
        assert!(indexed, "{column} should be indexed");
    }
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
    customer_id: i64,
    make_id: i64,
    color_id: i64,
}

impl Fixture {
    fn valid_motorcycle(&self, plate_number: &str) -> MotorcycleRecord {
        MotorcycleRecord {
            customer_id: self.customer_id,
            make_id: self.make_id,
            model: "MT-07".to_string(),
            year: Some(2026),
            plate_number: plate_number.to_string(),
            vin: None,
            chassis_number: None,
            color_id: self.color_id,
            notes: None,
            archived_at: None,
        }
    }
}

#[derive(Clone)]
struct MotorcycleRecord {
    customer_id: i64,
    make_id: i64,
    model: String,
    year: Option<i64>,
    plate_number: String,
    vin: Option<String>,
    chassis_number: Option<String>,
    color_id: i64,
    notes: Option<String>,
    archived_at: Option<i64>,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    let customer_id = insert_customer(&connection, "Ahmad", "+962791111111");
    let make_id = catalog_id(&connection, "motorcycle_makes", "Honda");
    let color_id = catalog_id(&connection, "motorcycle_colors", "Black");
    Fixture {
        _temp_dir: temp_dir,
        connection,
        customer_id,
        make_id,
        color_id,
    }
}

fn insert_customer(connection: &Connection, name: &str, phone: &str) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at) VALUES (?1, ?2, 1000, 1000)",
            (name, phone),
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn catalog_id(connection: &Connection, table: &str, value: &str) -> i64 {
    connection
        .query_row(
            &format!("SELECT id FROM {table} WHERE name = ?1"),
            [value],
            |row| row.get(0),
        )
        .unwrap()
}

fn insert_motorcycle(
    connection: &Connection,
    motorcycle: &MotorcycleRecord,
) -> rusqlite::Result<usize> {
    insert_motorcycle_with_plate(connection, motorcycle, Some(&motorcycle.plate_number))
}

fn insert_motorcycle_with_plate(
    connection: &Connection,
    motorcycle: &MotorcycleRecord,
    plate_number: Option<&str>,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO motorcycles (
            customer_id, make_id, model, year, plate_number, vin, chassis_number,
            color_id, notes, created_at, updated_at, archived_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1000, 1000, ?10)",
        params![
            motorcycle.customer_id,
            motorcycle.make_id,
            motorcycle.model,
            motorcycle.year,
            plate_number,
            motorcycle.vin,
            motorcycle.chassis_number,
            motorcycle.color_id,
            motorcycle.notes,
            motorcycle.archived_at,
        ],
    )
}

fn motorcycle_column_is_indexed(connection: &Connection, column: &str) -> bool {
    connection
        .query_row(
            "SELECT EXISTS (
                SELECT 1 FROM pragma_index_list('motorcycles') AS index_list
                JOIN pragma_index_info(index_list.name) AS index_info ON TRUE
                WHERE index_info.name = ?1
             )",
            [column],
            |row| row.get(0),
        )
        .unwrap()
}

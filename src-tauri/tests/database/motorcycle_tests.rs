use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::db::{migrate_database, open_database};

const VIN_ONE: &str = "1HGCM82633A004352";
const VIN_TWO: &str = "ABCDEFGHJKLMNPRST";

#[test]
fn motorcycle_ids_are_generated_automatically() {
    // # Arrange
    let fixture = fixture();
    let first = fixture.valid_plate_motorcycle();
    let mut second = first.clone();
    second.plate_number = Some(12346);

    // # Act
    insert_motorcycle(&fixture.connection, &first).expect("first motorcycle should be inserted");
    let first_id = fixture.connection.last_insert_rowid();
    insert_motorcycle(&fixture.connection, &second).expect("second motorcycle should be inserted");
    let second_id = fixture.connection.last_insert_rowid();

    // # Assert
    assert_eq!(first_id, 1);
    assert_eq!(second_id, 2);
}

#[test]
fn motorcycle_accepts_existing_customer_make_color_and_plate_code() {
    // # Arrange
    let fixture = fixture();
    let motorcycle = fixture.valid_plate_motorcycle();

    // # Act
    let result = insert_motorcycle(&fixture.connection, &motorcycle);

    // # Assert
    assert!(result.is_ok());
}

#[test]
fn motorcycle_rejects_nonexistent_foreign_keys() {
    // # Arrange
    let fixture = fixture();
    let cases = ["customer", "make", "color", "plate code"];

    for field in cases {
        let mut motorcycle = fixture.valid_plate_motorcycle();
        match field {
            "customer" => motorcycle.customer_id = 999_001,
            "make" => motorcycle.make_id = 999_002,
            "color" => motorcycle.color_id = 999_003,
            "plate code" => motorcycle.plate_code_id = Some(999_004),
            _ => unreachable!(),
        }

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(result.is_err(), "nonexistent {field} should be rejected");
    }
}

#[test]
fn motorcycle_accepts_plate_only_vin_only_and_combined_identity() {
    // # Arrange
    let fixture = fixture();
    let plate_only = fixture.valid_plate_motorcycle();

    let mut vin_only = fixture.valid_plate_motorcycle();
    vin_only.plate_code_id = None;
    vin_only.plate_number = None;
    vin_only.vin = Some(VIN_ONE.to_string());

    let mut combined = fixture.valid_plate_motorcycle();
    combined.plate_number = Some(12346);
    combined.vin = Some(VIN_TWO.to_string());

    // # Act
    let plate_result = insert_motorcycle(&fixture.connection, &plate_only);
    let vin_result = insert_motorcycle(&fixture.connection, &vin_only);
    let combined_result = insert_motorcycle(&fixture.connection, &combined);

    // # Assert
    assert!(plate_result.is_ok());
    assert!(vin_result.is_ok());
    assert!(combined_result.is_ok());
}

#[test]
fn motorcycle_database_accepts_every_supported_identity_combination() {
    // # Arrange
    let cases = [
        ("plate only", true, false, false, true),
        ("VIN only", false, true, false, true),
        ("chassis only", false, false, true, true),
        ("plate and VIN", true, true, false, true),
        ("plate and chassis", true, false, true, true),
        ("VIN and chassis", false, true, true, true),
        ("all three", true, true, true, true),
        ("none", false, false, false, false),
    ];

    for (case, has_plate, has_vin, has_chassis, should_succeed) in cases {
        let fixture = fixture();
        let mut motorcycle = fixture.valid_plate_motorcycle();
        if !has_plate {
            motorcycle.plate_code_id = None;
            motorcycle.plate_number = None;
        }
        motorcycle.vin = has_vin.then(|| VIN_ONE.to_string());
        motorcycle.chassis_number = has_chassis.then(|| "FRAME/ABC-123.4".to_string());

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert_eq!(result.is_ok(), should_succeed, "identity case: {case}");
    }
}

#[test]
fn motorcycle_database_enforces_canonical_chassis_number_rules() {
    // # Arrange
    let fixture = fixture();
    let invalid_chassis_numbers = [
        "".to_string(),
        "abc123".to_string(),
        "ABC 123".to_string(),
        "ABC_123".to_string(),
        "ABC@123".to_string(),
        "هيكل123".to_string(),
        "A".repeat(65),
    ];

    for chassis_number in invalid_chassis_numbers {
        let mut motorcycle = fixture.valid_plate_motorcycle();
        motorcycle.chassis_number = Some(chassis_number.clone());

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(
            result.is_err(),
            "chassis should be rejected: {chassis_number:?}"
        );
    }

    let mut maximum = fixture.valid_plate_motorcycle();
    maximum.chassis_number = Some("A".repeat(64));

    // # Act
    let maximum_result = insert_motorcycle(&fixture.connection, &maximum);

    // # Assert
    assert!(maximum_result.is_ok());
}

#[test]
fn motorcycle_chassis_number_is_unique_and_multiple_nulls_are_allowed() {
    // # Arrange
    let fixture = fixture();
    let mut first = fixture.valid_plate_motorcycle();
    first.chassis_number = Some("FRAME/12345".to_string());
    insert_motorcycle(&fixture.connection, &first).expect("first chassis should be inserted");

    let mut duplicate = first.clone();
    duplicate.plate_number = Some(12346);

    let mut first_null = fixture.valid_plate_motorcycle();
    first_null.plate_number = Some(12347);
    let mut second_null = first_null.clone();
    second_null.plate_number = Some(12348);

    // # Act
    let duplicate_result = insert_motorcycle(&fixture.connection, &duplicate);
    let first_null_result = insert_motorcycle(&fixture.connection, &first_null);
    let second_null_result = insert_motorcycle(&fixture.connection, &second_null);

    // # Assert
    assert!(duplicate_result.is_err());
    assert!(first_null_result.is_ok());
    assert!(second_null_result.is_ok());
}

#[test]
fn motorcycle_rejects_missing_or_half_present_plate_identity() {
    // # Arrange
    let fixture = fixture();
    let cases = [
        (None, None),
        (Some(fixture.plate_code_id), None),
        (None, Some(12345)),
    ];

    for (plate_code_id, plate_number) in cases {
        let mut motorcycle = fixture.valid_plate_motorcycle();
        motorcycle.plate_code_id = plate_code_id;
        motorcycle.plate_number = plate_number;
        motorcycle.vin = None;

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(
            result.is_err(),
            "identity should reject code {plate_code_id:?} and number {plate_number:?}"
        );
    }
}

#[test]
fn motorcycle_plate_number_enforces_numeric_boundaries() {
    // # Arrange
    let fixture = fixture();

    for plate_number in [1, 99_999] {
        let mut motorcycle = fixture.valid_plate_motorcycle();
        motorcycle.plate_number = Some(plate_number);

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(result.is_ok(), "plate {plate_number} should be accepted");
    }

    for plate_number in [0, 100_000] {
        let mut motorcycle = fixture.valid_plate_motorcycle();
        motorcycle.plate_number = Some(plate_number);

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(result.is_err(), "plate {plate_number} should be rejected");
    }
}

#[test]
fn motorcycle_rejects_text_plate_number_at_database_boundary() {
    // # Arrange
    let fixture = fixture();

    // # Act
    let result = fixture.connection.execute(
        "
        INSERT INTO motorcycles (
            customer_id, make_id, model, plate_code_id, plate_number,
            color_id, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            fixture.customer_id,
            fixture.make_id,
            "MT-07",
            fixture.plate_code_id,
            "abc",
            fixture.color_id,
            1_000_i64,
            1_000_i64,
        ],
    );

    // # Assert
    assert!(result.is_err());
}

#[test]
fn plate_identity_is_unique_per_code_but_number_may_repeat_across_codes() {
    // # Arrange
    let fixture = fixture();
    let first = fixture.valid_plate_motorcycle();
    insert_motorcycle(&fixture.connection, &first).expect("first plate should be inserted");

    let duplicate = first.clone();

    let second_code_id = insert_plate_code(&fixture.connection, "B");
    let mut same_number_different_code = first.clone();
    same_number_different_code.plate_code_id = Some(second_code_id);

    // # Act
    let duplicate_result = insert_motorcycle(&fixture.connection, &duplicate);
    let different_code_result = insert_motorcycle(&fixture.connection, &same_number_different_code);

    // # Assert
    assert!(duplicate_result.is_err());
    assert!(different_code_result.is_ok());
}

#[test]
fn vin_is_unique_when_present_and_null_is_allowed_with_distinct_plates() {
    // # Arrange
    let fixture = fixture();
    let mut first_vin = fixture.valid_plate_motorcycle();
    first_vin.plate_code_id = None;
    first_vin.plate_number = None;
    first_vin.vin = Some(VIN_ONE.to_string());
    insert_motorcycle(&fixture.connection, &first_vin).expect("first VIN should be inserted");

    let duplicate_vin = first_vin.clone();

    let first_null_vin = fixture.valid_plate_motorcycle();
    let mut second_null_vin = first_null_vin.clone();
    second_null_vin.plate_number = Some(12346);

    // # Act
    let duplicate_result = insert_motorcycle(&fixture.connection, &duplicate_vin);
    let first_null_result = insert_motorcycle(&fixture.connection, &first_null_vin);
    let second_null_result = insert_motorcycle(&fixture.connection, &second_null_vin);

    // # Assert
    assert!(duplicate_result.is_err());
    assert!(first_null_result.is_ok());
    assert!(second_null_result.is_ok());
}

#[test]
fn malformed_vin_is_rejected_by_database_constraints() {
    // # Arrange
    let fixture = fixture();
    let malformed = [
        "1HGCM82633A00435",
        "1HGCM82633A004352X",
        "1hgcm82633a004352",
        "1HGCM82633I004352",
        "1HGCM82633O004352",
        "1HGCM82633Q004352",
        "1HGCM82633A00435-",
        "1HGCM82633A00 352",
        "1HGCM82633A00435é",
    ];

    for vin in malformed {
        let mut motorcycle = fixture.valid_plate_motorcycle();
        motorcycle.plate_code_id = None;
        motorcycle.plate_number = None;
        motorcycle.vin = Some(vin.to_string());

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(result.is_err(), "VIN should be rejected: {vin:?}");
    }
}

#[test]
fn motorcycle_model_is_trimmed_nonblank_and_at_most_eighty_characters_in_database() {
    // # Arrange
    let fixture = fixture();
    let invalid_models = ["   ".to_string(), " MT-07".to_string(), "M".repeat(81)];

    for model in invalid_models {
        let mut motorcycle = fixture.valid_plate_motorcycle();
        motorcycle.model = model;

        // # Act
        let result = insert_motorcycle(&fixture.connection, &motorcycle);

        // # Assert
        assert!(result.is_err());
    }
}

#[test]
fn motorcycle_year_lower_bound_is_enforced_in_database() {
    // # Arrange
    let fixture = fixture();
    let mut motorcycle = fixture.valid_plate_motorcycle();
    motorcycle.year = Some(1884);

    // # Act
    let result = insert_motorcycle(&fixture.connection, &motorcycle);

    // # Assert
    assert!(result.is_err());
}

#[test]
fn motorcycle_notes_are_bounded_in_database() {
    // # Arrange
    let fixture = fixture();
    let mut motorcycle = fixture.valid_plate_motorcycle();
    motorcycle.notes = Some("N".repeat(2001));

    // # Act
    let result = insert_motorcycle(&fixture.connection, &motorcycle);

    // # Assert
    assert!(result.is_err());
}

#[test]
fn one_customer_may_own_multiple_motorcycles() {
    // # Arrange
    let fixture = fixture();
    let first = fixture.valid_plate_motorcycle();
    let mut second = first.clone();
    second.plate_number = Some(12346);

    // # Act
    insert_motorcycle(&fixture.connection, &first).expect("first motorcycle should be inserted");
    insert_motorcycle(&fixture.connection, &second).expect("second motorcycle should be inserted");

    // # Assert
    let count: i64 = fixture
        .connection
        .query_row(
            "SELECT COUNT(*) FROM motorcycles WHERE customer_id = ?1",
            [fixture.customer_id],
            |row| row.get(0),
        )
        .expect("motorcycle count should be queryable");
    assert_eq!(count, 2);
}

#[test]
fn different_customers_may_own_motorcycles() {
    // # Arrange
    let fixture = fixture();
    let second_customer_id = insert_customer(&fixture.connection, "Omar", "+962792222222");
    let first = fixture.valid_plate_motorcycle();
    let mut second = first.clone();
    second.customer_id = second_customer_id;
    second.plate_number = Some(12346);

    // # Act
    insert_motorcycle(&fixture.connection, &first).expect("first motorcycle should be inserted");
    insert_motorcycle(&fixture.connection, &second).expect("second motorcycle should be inserted");

    // # Assert
    let owner_count: i64 = fixture
        .connection
        .query_row(
            "SELECT COUNT(DISTINCT customer_id) FROM motorcycles",
            [],
            |row| row.get(0),
        )
        .expect("owner count should be queryable");
    assert_eq!(owner_count, 2);
}

#[test]
fn referenced_customer_make_color_and_plate_code_cannot_be_deleted() {
    // # Arrange
    let fixture = fixture();
    let motorcycle = fixture.valid_plate_motorcycle();
    insert_motorcycle(&fixture.connection, &motorcycle).expect("motorcycle should be inserted");

    let deletes = [
        ("customers", fixture.customer_id),
        ("motorcycle_makes", fixture.make_id),
        ("motorcycle_colors", fixture.color_id),
        ("jordan_plate_codes", fixture.plate_code_id),
    ];

    for (table, id) in deletes {
        // # Act
        let result = fixture
            .connection
            .execute(&format!("DELETE FROM {table} WHERE id = ?1"), [id]);

        // # Assert
        assert!(
            result.is_err(),
            "referenced {table} row should be protected"
        );
    }

    let count: i64 = fixture
        .connection
        .query_row("SELECT COUNT(*) FROM motorcycles", [], |row| row.get(0))
        .expect("motorcycle count should be queryable");
    assert_eq!(count, 1);
}

#[test]
fn archived_motorcycle_retains_all_catalog_and_customer_relationships() {
    // # Arrange
    let fixture = fixture();
    let mut motorcycle = fixture.valid_plate_motorcycle();
    motorcycle.archived_at = Some(3_000);

    // # Act
    insert_motorcycle(&fixture.connection, &motorcycle)
        .expect("archived motorcycle should be inserted");

    // # Assert
    let (archived_at, customer, make, color, code): (Option<i64>, String, String, String, String) =
        fixture
            .connection
            .query_row(
                "
            SELECT m.archived_at, c.name, mk.name, co.name, pc.code
            FROM motorcycles m
            JOIN customers c ON c.id = m.customer_id
            JOIN motorcycle_makes mk ON mk.id = m.make_id
            JOIN motorcycle_colors co ON co.id = m.color_id
            JOIN jordan_plate_codes pc ON pc.id = m.plate_code_id
            ",
                [],
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
            .expect("archived relationships should be queryable");

    assert_eq!(archived_at, Some(3_000));
    assert_eq!(customer, "Ahmad");
    assert_eq!(make, "Honda");
    assert_eq!(color, "Black");
    assert_eq!(code, "A");
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
    plate_code_id: i64,
}

impl Fixture {
    fn valid_plate_motorcycle(&self) -> MotorcycleRecord {
        MotorcycleRecord {
            customer_id: self.customer_id,
            make_id: self.make_id,
            model: "MT-07".to_string(),
            year: Some(2026),
            plate_code_id: Some(self.plate_code_id),
            plate_number: Some(12345),
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
    plate_code_id: Option<i64>,
    plate_number: Option<i64>,
    vin: Option<String>,
    chassis_number: Option<String>,
    color_id: i64,
    notes: Option<String>,
    archived_at: Option<i64>,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    migrate_database(&mut connection).expect("database should migrate");

    let customer_id = insert_customer(&connection, "Ahmad", "+962791111111");
    let make_id = catalog_id(&connection, "motorcycle_makes", "name", "Honda");
    let color_id = catalog_id(&connection, "motorcycle_colors", "name", "Black");
    let plate_code_id = insert_plate_code(&connection, "A");

    Fixture {
        _temp_dir: temp_dir,
        connection,
        customer_id,
        make_id,
        color_id,
        plate_code_id,
    }
}

fn insert_customer(connection: &Connection, name: &str, phone: &str) -> i64 {
    connection
        .execute(
            "
            INSERT INTO customers (name, phone, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ",
            (name, phone, 1_000_i64, 1_000_i64),
        )
        .expect("customer should be inserted");
    connection.last_insert_rowid()
}

fn insert_plate_code(connection: &Connection, code: &str) -> i64 {
    connection
        .execute("INSERT INTO jordan_plate_codes (code) VALUES (?1)", [code])
        .expect("plate code should be inserted");
    connection.last_insert_rowid()
}

fn catalog_id(connection: &Connection, table: &str, column: &str, value: &str) -> i64 {
    connection
        .query_row(
            &format!("SELECT id FROM {table} WHERE {column} = ?1"),
            [value],
            |row| row.get(0),
        )
        .expect("catalog row should exist")
}

fn insert_motorcycle(
    connection: &Connection,
    motorcycle: &MotorcycleRecord,
) -> rusqlite::Result<usize> {
    connection.execute(
        "
        INSERT INTO motorcycles (
            customer_id,
            make_id,
            model,
            year,
            plate_code_id,
            plate_number,
            vin,
            chassis_number,
            color_id,
            notes,
            created_at,
            updated_at,
            archived_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ",
        params![
            motorcycle.customer_id,
            motorcycle.make_id,
            motorcycle.model,
            motorcycle.year,
            motorcycle.plate_code_id,
            motorcycle.plate_number,
            motorcycle.vin,
            motorcycle.chassis_number,
            motorcycle.color_id,
            motorcycle.notes,
            1_000_i64,
            1_000_i64,
            motorcycle.archived_at,
        ],
    )
}

fn motorcycle_column_is_indexed(connection: &Connection, column: &str) -> bool {
    connection
        .query_row(
            "
            SELECT EXISTS (
                SELECT 1
                FROM pragma_index_list('motorcycles') AS index_list
                JOIN pragma_index_info(index_list.name) AS index_info ON TRUE
                WHERE index_info.name = ?1
            )
            ",
            [column],
            |row| row.get(0),
        )
        .expect("motorcycle indexes should be queryable")
}

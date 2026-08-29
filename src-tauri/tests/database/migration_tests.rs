use tempfile::tempdir;

use moto_workshop_lib::db::{migrate_database, open_database, MigrationError};

const INSERT_VALIDATION_TRIGGER: &str = "validate_customers_before_insert_v3";
const UPDATE_VALIDATION_TRIGGER: &str = "validate_customers_before_update_v3";

#[test]
fn fresh_database_migrates_to_latest_schema_version() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    let mut connection = open_database(&database_path).expect("database should open");

    assert_eq!(current_schema_version(&connection), 0);

    // # Act
    migrate_database(&mut connection).expect("database migrations should succeed");

    // # Assert
    assert_eq!(current_schema_version(&connection), 3);
    assert!(table_exists(&connection, "customers"));
    assert!(table_exists(&connection, "motorcycle_makes"));
    assert!(table_exists(&connection, "motorcycle_colors"));
    assert!(table_exists(&connection, "jordan_plate_codes"));
    assert!(table_exists(&connection, "motorcycles"));
}

#[test]
fn migrating_an_already_migrated_database_is_safe() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    let mut connection = open_database(&database_path).expect("database should open");

    migrate_database(&mut connection).expect("first migration should succeed");

    // # Act
    migrate_database(&mut connection).expect("second migration should also succeed");

    // # Assert
    assert_eq!(current_schema_version(&connection), 3);
    assert!(table_exists(&connection, "customers"));
    assert!(table_exists(&connection, "motorcycle_makes"));
    assert!(table_exists(&connection, "motorcycle_colors"));
    assert!(table_exists(&connection, "jordan_plate_codes"));
    assert!(table_exists(&connection, "motorcycles"));
}

#[test]
fn migration_one_stamps_version_one_and_failed_migration_two_rolls_back() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");

    stop_after_migration_one(&mut connection);

    // # Act
    let schema_version = current_schema_version(&connection);

    // # Assert
    assert_eq!(schema_version, 1);
    assert!(table_exists(&connection, "customers"));
    assert!(!table_exists(&connection, "motorcycle_makes"));
    assert!(!table_exists(&connection, "motorcycle_colors"));
    assert!(!table_exists(&connection, "jordan_plate_codes"));
    assert!(!table_exists(&connection, "motorcycles"));
}

#[test]
fn version_one_database_upgrades_to_latest_version() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");

    stop_after_migration_one(&mut connection);

    connection
        .execute(
            "
            INSERT INTO customers (name, phone, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ",
            ("Ahmad", "+962791234567", 1_000_i64, 1_000_i64),
        )
        .expect("version-one customer should be inserted");

    // # Act
    migrate_database(&mut connection).expect("version-one database should upgrade");

    // # Assert
    assert_eq!(current_schema_version(&connection), 3);
    assert!(table_exists(&connection, "motorcycle_makes"));
    assert!(table_exists(&connection, "motorcycle_colors"));
    assert!(table_exists(&connection, "jordan_plate_codes"));
    assert!(table_exists(&connection, "motorcycles"));

    let customer_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM customers", [], |row| row.get(0))
        .expect("preserved customer count should be queryable");
    assert_eq!(customer_count, 1);
}

#[test]
fn version_two_database_upgrades_to_version_three_and_preserves_customer_and_motorcycle() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    stop_after_migration_two(&mut connection);

    connection
        .execute(
            "INSERT INTO customers (name, phone, notes, created_at, updated_at)
             VALUES ('Ahmad', '+962791234567', 'Existing notes', 1000, 1000)",
            [],
        )
        .expect("version-two customer should be inserted");
    let customer_id = connection.last_insert_rowid();
    connection
        .execute("INSERT INTO jordan_plate_codes (code) VALUES ('A')", [])
        .expect("plate code should be inserted");
    let plate_code_id = connection.last_insert_rowid();
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .expect("seeded make should exist");
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .expect("seeded color should exist");
    connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, plate_code_id, plate_number,
                color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'MT-07', ?3, 12345, ?4, 1000, 1000)",
            (customer_id, make_id, plate_code_id, color_id),
        )
        .expect("version-two motorcycle should be inserted");
    let motorcycle_id = connection.last_insert_rowid();

    // # Act
    migrate_database(&mut connection).expect("version-two database should upgrade");

    // # Assert
    assert_eq!(current_schema_version(&connection), 3);
    let customer: (String, String, Option<String>) = connection
        .query_row(
            "SELECT name, phone, notes FROM customers WHERE id = ?1",
            [customer_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("existing customer should survive");
    assert_eq!(
        customer,
        (
            "Ahmad".to_string(),
            "+962791234567".to_string(),
            Some("Existing notes".to_string())
        )
    );
    let preserved_motorcycle: (i64, i64) = connection
        .query_row(
            "SELECT id, customer_id FROM motorcycles WHERE id = ?1",
            [motorcycle_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("existing motorcycle should survive");
    assert_eq!(preserved_motorcycle, (motorcycle_id, customer_id));
}

#[test]
fn migration_two_stamps_version_two_before_migration_three_runs() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");

    // # Act
    stop_after_migration_two(&mut connection);

    // # Assert
    assert_eq!(current_schema_version(&connection), 2);
    assert!(table_exists(&connection, "customers"));
    assert!(table_exists(&connection, "motorcycles"));
    assert!(!trigger_exists(&connection, INSERT_VALIDATION_TRIGGER));
    assert!(!trigger_exists(&connection, UPDATE_VALIDATION_TRIGGER));
}

#[test]
fn version_three_migration_is_a_no_op_when_rerun() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    migrate_database(&mut connection).expect("initial migration should succeed");
    let schema_before = schema_objects(&connection);

    // # Act
    migrate_database(&mut connection).expect("version-three rerun should succeed");

    // # Assert
    assert_eq!(current_schema_version(&connection), 3);
    assert_eq!(schema_objects(&connection), schema_before);
}

#[test]
fn databases_newer_than_supported_are_rejected_without_modification() {
    for future_version in [4_i64, 999_i64] {
        // # Arrange
        let temp_dir = tempdir().expect("temporary directory should be created");
        let database_path = temp_dir.path().join(format!("future-{future_version}.db"));
        let mut connection = open_database(&database_path).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE sentinel (value TEXT NOT NULL);
                 INSERT INTO sentinel (value) VALUES ('unchanged');",
            )
            .expect("sentinel data should be created");
        connection
            .pragma_update(None, "user_version", future_version)
            .expect("future version should be stamped");
        let schema_before = schema_objects(&connection);

        // # Act
        let result = migrate_database(&mut connection);

        // # Assert
        assert!(matches!(
            result,
            Err(MigrationError::UnsupportedSchemaVersion {
                found,
                max_supported: 3
            }) if found == future_version
        ));
        assert_eq!(current_schema_version(&connection), future_version);
        assert_eq!(schema_objects(&connection), schema_before);
        let sentinel: String = connection
            .query_row("SELECT value FROM sentinel", [], |row| row.get(0))
            .expect("sentinel data should remain queryable");
        assert_eq!(sentinel, "unchanged");
        assert!(!table_exists(&connection, "customers"));
    }
}

#[test]
fn failed_migration_three_rolls_back_its_partial_objects_and_version_stamp() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    stop_after_migration_two(&mut connection);
    create_validation_trigger_blocker(&connection, UPDATE_VALIDATION_TRIGGER);

    // # Act
    let result = migrate_database(&mut connection);

    // # Assert
    assert!(result.is_err());
    assert_eq!(current_schema_version(&connection), 2);
    assert!(!trigger_exists(&connection, INSERT_VALIDATION_TRIGGER));
    assert!(trigger_exists(&connection, UPDATE_VALIDATION_TRIGGER));
}

#[test]
fn migration_three_rejects_invalid_legacy_customers_without_modifying_related_data() {
    // # Arrange
    let cases = [
        (
            "padded name",
            " Ahmad ".to_string(),
            "+962791234567".to_string(),
            None,
        ),
        (
            "malformed phone",
            "Ahmad".to_string(),
            "banana".to_string(),
            None,
        ),
        (
            "oversized notes",
            "Ahmad".to_string(),
            "+962791234567".to_string(),
            Some("N".repeat(2_001)),
        ),
    ];

    for (case, name, phone, notes) in cases {
        let temp_dir = tempdir().expect("temporary directory should be created");
        let database_path = temp_dir.path().join(format!("{case}.db"));
        let mut connection = open_database(&database_path).expect("database should open");
        stop_after_migration_two(&mut connection);
        connection
            .execute(
                "INSERT INTO customers (name, phone, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 1000, 1000)",
                rusqlite::params![name, phone, notes],
            )
            .expect("legacy customer should be legal in schema version two");
        let customer_id = connection.last_insert_rowid();
        connection
            .execute("INSERT INTO jordan_plate_codes (code) VALUES ('A')", [])
            .expect("plate code should be inserted");
        let plate_code_id = connection.last_insert_rowid();
        let make_id: i64 = connection
            .query_row(
                "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
                [],
                |row| row.get(0),
            )
            .expect("seeded make should exist");
        let color_id: i64 = connection
            .query_row(
                "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
                [],
                |row| row.get(0),
            )
            .expect("seeded color should exist");
        connection
            .execute(
                "INSERT INTO motorcycles (
                    customer_id, make_id, model, plate_code_id, plate_number,
                    color_id, created_at, updated_at
                 ) VALUES (?1, ?2, 'MT-07', ?3, 12345, ?4, 1000, 1000)",
                (customer_id, make_id, plate_code_id, color_id),
            )
            .expect("related motorcycle should be inserted");
        let motorcycle_id = connection.last_insert_rowid();

        // # Act
        let result = migrate_database(&mut connection);

        // # Assert
        assert!(
            matches!(
                result,
                Err(MigrationError::InvalidExistingCustomer {
                    customer_id: found_customer_id
                }) if found_customer_id == customer_id
            ),
            "invalid legacy case should identify its Customer: {case}"
        );
        assert_eq!(current_schema_version(&connection), 2, "case: {case}");
        let preserved_customer: (String, String, Option<String>) = connection
            .query_row(
                "SELECT name, phone, notes FROM customers WHERE id = ?1",
                [customer_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("invalid legacy customer should remain unchanged");
        assert_eq!(preserved_customer, (name, phone, notes), "case: {case}");
        let preserved_motorcycle_customer_id: i64 = connection
            .query_row(
                "SELECT customer_id FROM motorcycles WHERE id = ?1",
                [motorcycle_id],
                |row| row.get(0),
            )
            .expect("related motorcycle should remain unchanged");
        assert_eq!(
            preserved_motorcycle_customer_id, customer_id,
            "case: {case}"
        );
        assert!(!trigger_exists(&connection, INSERT_VALIDATION_TRIGGER));
        assert!(!trigger_exists(&connection, UPDATE_VALIDATION_TRIGGER));
    }
}

fn stop_after_migration_one(connection: &mut rusqlite::Connection) {
    connection
        .execute_batch("CREATE VIEW motorcycles AS SELECT 1 AS migration_blocker;")
        .expect("migration-two blocker should be created");

    let result = migrate_database(connection);
    assert!(
        result.is_err(),
        "the migration-two blocker should stop the migration runner"
    );

    connection
        .execute_batch("DROP VIEW motorcycles;")
        .expect("migration-two blocker should be removed");
}

fn stop_after_migration_two(connection: &mut rusqlite::Connection) {
    stop_after_migration_one(connection);
    create_validation_trigger_blocker(connection, INSERT_VALIDATION_TRIGGER);

    let result = migrate_database(connection);
    assert!(
        result.is_err(),
        "the migration-three blocker should stop the migration runner"
    );
    assert_eq!(current_schema_version(connection), 2);

    connection
        .execute_batch(&format!("DROP TRIGGER {INSERT_VALIDATION_TRIGGER};"))
        .expect("migration-three blocker should be removed");
}

fn create_validation_trigger_blocker(connection: &rusqlite::Connection, trigger_name: &str) {
    connection
        .execute_batch(&format!(
            "CREATE TRIGGER {trigger_name}
             BEFORE INSERT ON customers
             BEGIN
                 SELECT 1;
             END;"
        ))
        .expect("migration-three blocker should be created");
}

fn current_schema_version(connection: &rusqlite::Connection) -> i64 {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable")
}

fn table_exists(connection: &rusqlite::Connection, table_name: &str) -> bool {
    connection
        .query_row(
            "
            SELECT EXISTS (
                SELECT 1
                FROM sqlite_master
                WHERE type = 'table'
                  AND name = ?1
            )
            ",
            [table_name],
            |row| row.get(0),
        )
        .expect("table existence should be queryable")
}

fn trigger_exists(connection: &rusqlite::Connection, trigger_name: &str) -> bool {
    connection
        .query_row(
            "SELECT EXISTS (
                SELECT 1 FROM sqlite_master WHERE type = 'trigger' AND name = ?1
             )",
            [trigger_name],
            |row| row.get(0),
        )
        .expect("trigger existence should be queryable")
}

fn schema_objects(connection: &rusqlite::Connection) -> Vec<(String, String, String)> {
    let mut statement = connection
        .prepare(
            "SELECT type, name, COALESCE(sql, '')
             FROM sqlite_master
             WHERE name NOT LIKE 'sqlite_%'
             ORDER BY type, name",
        )
        .expect("schema should be queryable");

    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("schema rows should be readable")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("schema rows should be collected")
}

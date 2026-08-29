use tempfile::tempdir;

use moto_workshop_lib::db::{migrate_database, open_database};

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
    assert_eq!(current_schema_version(&connection), 2);
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
    assert_eq!(current_schema_version(&connection), 2);
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
fn version_one_database_upgrades_to_version_two() {
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
    assert_eq!(current_schema_version(&connection), 2);
    assert!(table_exists(&connection, "motorcycle_makes"));
    assert!(table_exists(&connection, "motorcycle_colors"));
    assert!(table_exists(&connection, "jordan_plate_codes"));
    assert!(table_exists(&connection, "motorcycles"));

    let customer_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM customers", [], |row| row.get(0))
        .expect("preserved customer count should be queryable");
    assert_eq!(customer_count, 1);
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

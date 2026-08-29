use tempfile::tempdir;

use moto_workshop_lib::db::{migrate_database, open_database};

#[test]
fn fresh_database_migrates_to_latest_schema_version() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    let mut connection = open_database(&database_path).expect("database should open");

    let version_before: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable");

    assert_eq!(version_before, 0);

    // # Act
    migrate_database(&mut connection).expect("database migrations should succeed");

    // # Assert
    let version_after: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable");

    assert_eq!(version_after, 1);

    let customers_table_exists: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM sqlite_master
            WHERE type = 'table'
              AND name = 'customers'
            ",
            [],
            |row| row.get(0),
        )
        .expect("customers table existence should be queryable");

    assert_eq!(customers_table_exists, 1);
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
    let schema_version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .expect("schema version should be readable");

    assert_eq!(schema_version, 1);

    let customers_table_count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM sqlite_master
            WHERE type = 'table'
              AND name = 'customers'
            ",
            [],
            |row| row.get(0),
        )
        .expect("customers table existence should be queryable");

    assert_eq!(customers_table_count, 1);
}

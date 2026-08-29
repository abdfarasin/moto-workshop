use tempfile::tempdir;

use moto_workshop_lib::db::open_database;

#[test]
fn opened_database_enables_foreign_keys() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    // # Act
    let connection = open_database(&database_path).expect("database should open");

    // # Assert
    let foreign_keys: i64 = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .expect("foreign key setting should be readable");

    assert_eq!(foreign_keys, 1);
}

#[test]
fn opened_database_uses_wal_journal_mode() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    // # Act
    let connection = open_database(&database_path).expect("database should open");

    // # Assert
    let journal_mode: String = connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .expect("journal mode should be readable");

    assert_eq!(journal_mode.to_lowercase(), "wal");
}

#[test]
fn opened_database_uses_full_synchronous_mode() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    // # Act
    let connection = open_database(&database_path).expect("database should open");

    // # Assert
    let synchronous: i64 = connection
        .pragma_query_value(None, "synchronous", |row| row.get(0))
        .expect("synchronous setting should be readable");

    assert_eq!(synchronous, 2);
}

#[test]
fn opened_database_uses_five_second_busy_timeout() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    // # Act
    let connection = open_database(&database_path).expect("database should open");

    // # Assert
    let busy_timeout_ms: i64 = connection
        .pragma_query_value(None, "busy_timeout", |row| row.get(0))
        .expect("busy timeout should be readable");

    assert_eq!(busy_timeout_ms, 5000);
}

use tempfile::tempdir;

use moto_workshop_lib::db::{migrate_database, open_database};

#[test]
fn customer_ids_are_generated_automatically() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    let mut connection = open_database(&database_path).expect("database should open");

    migrate_database(&mut connection).expect("database should migrate");

    // # Act
    connection
        .execute(
            "
            INSERT INTO customers (
                name,
                phone,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4)
            ",
            ("Ahmad", "+962791111111", 1_000_i64, 1_000_i64),
        )
        .expect("first customer should be inserted");

    let first_id = connection.last_insert_rowid();

    connection
        .execute(
            "
            INSERT INTO customers (
                name,
                phone,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4)
            ",
            ("Omar", "+962792222222", 2_000_i64, 2_000_i64),
        )
        .expect("second customer should be inserted");

    let second_id = connection.last_insert_rowid();

    // # Assert
    assert_eq!(first_id, 1);
    assert_eq!(second_id, 2);
}

#[test]
fn customer_phone_numbers_must_be_unique() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");

    let mut connection = open_database(&database_path).expect("database should open");

    migrate_database(&mut connection).expect("database should migrate");

    connection
        .execute(
            "
            INSERT INTO customers (
                name,
                phone,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4)
            ",
            ("Ahmad", "+962791111111", 1_000_i64, 1_000_i64),
        )
        .expect("first customer should be inserted");

    // # Act
    let result = connection.execute(
        "
        INSERT INTO customers (
            name,
            phone,
            created_at,
            updated_at
        )
        VALUES (?1, ?2, ?3, ?4)
        ",
        ("Omar", "+962791111111", 2_000_i64, 2_000_i64),
    );

    // # Assert
    assert!(result.is_err());
}

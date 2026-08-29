use rusqlite::Connection;
use tempfile::{tempdir, TempDir};

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

#[test]
fn customer_database_rejects_invalid_names_on_insert() {
    // # Arrange
    let (_temp_dir, connection) = migrated_connection();
    let invalid_names = [
        "".to_string(),
        "   ".to_string(),
        " Ahmad".to_string(),
        "Ahmad ".to_string(),
        "أ".repeat(101),
    ];

    for name in invalid_names {
        // # Act
        let result = insert_customer(&connection, &name, "+962791234567", None);

        // # Assert
        assert!(result.is_err(), "name should be rejected: {name:?}");
    }
}

#[test]
fn customer_database_accepts_supported_names_and_exact_boundaries() {
    // # Arrange
    let (_temp_dir, connection) = migrated_connection();
    let maximum_name = "أ".repeat(100);
    let maximum_notes = "ن".repeat(2_000);
    let valid_customers = [
        ("Ahmad Saleh", "+962791111111", None),
        ("أحمد صالح", "+962792222222", Some(maximum_notes.as_str())),
        (maximum_name.as_str(), "+962793333333", Some("")),
    ];

    for (name, phone, notes) in valid_customers {
        // # Act
        let result = insert_customer(&connection, name, phone, notes);

        // # Assert
        assert!(
            result.is_ok(),
            "valid customer should be accepted: {name:?}"
        );
    }

    // # Assert
    let null_notes: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM customers WHERE notes IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("NULL notes count should be queryable");
    assert_eq!(null_notes, 1);
}

#[test]
fn customer_database_rejects_noncanonical_phones_on_insert() {
    // # Arrange
    let (_temp_dir, connection) = migrated_connection();
    let invalid_phones = [
        "",
        "0791234567",
        "00962791234567",
        "banana",
        "+962banana",
        "+962 791234567",
        "+96279-1234567",
        "+96279123456",
        "+9627912345678",
        "+962٧٩١٢٣٤٥٦٧",
    ];

    for phone in invalid_phones {
        // # Act
        let result = insert_customer(&connection, "Ahmad", phone, None);

        // # Assert
        assert!(result.is_err(), "phone should be rejected: {phone:?}");
    }
}

#[test]
fn customer_database_rejects_notes_over_two_thousand_characters() {
    // # Arrange
    let (_temp_dir, connection) = migrated_connection();
    let oversized_notes = "ن".repeat(2_001);

    // # Act
    let result = insert_customer(
        &connection,
        "Ahmad",
        "+962791234567",
        Some(&oversized_notes),
    );

    // # Assert
    assert!(result.is_err());
}

#[test]
fn customer_database_rejects_invalid_updates_without_changing_the_row() {
    // # Arrange
    let (_temp_dir, connection) = migrated_connection();
    insert_customer(&connection, "Ahmad", "+962791234567", None)
        .expect("valid customer should be inserted");
    let customer_id = connection.last_insert_rowid();
    let oversized_notes = "N".repeat(2_001);
    let invalid_updates = [
        ("name", " Ahmad"),
        ("phone", "0791234567"),
        ("notes", oversized_notes.as_str()),
    ];

    for (column, value) in invalid_updates {
        // # Act
        let result = connection.execute(
            &format!("UPDATE customers SET {column} = ?1 WHERE id = ?2"),
            (value, customer_id),
        );

        // # Assert
        assert!(
            result.is_err(),
            "invalid {column} update should be rejected"
        );
    }

    // # Assert
    let customer: (String, String, Option<String>) = connection
        .query_row(
            "SELECT name, phone, notes FROM customers WHERE id = ?1",
            [customer_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("customer should remain queryable");
    assert_eq!(
        customer,
        ("Ahmad".to_string(), "+962791234567".to_string(), None)
    );
}

#[test]
fn customer_database_name_trimming_uses_sqlite_ascii_space_semantics() {
    // # Arrange
    let (_temp_dir, connection) = migrated_connection();

    // # Act
    let result = insert_customer(&connection, "\tAhmad\t", "+962791234567", None);

    // # Assert
    assert!(
        result.is_ok(),
        "SQLite trim() removes ASCII spaces, not tabs; the Rust domain enforces stronger rules"
    );
}

fn migrated_connection() -> (TempDir, Connection) {
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    migrate_database(&mut connection).expect("database should migrate");
    (temp_dir, connection)
}

fn insert_customer(
    connection: &Connection,
    name: &str,
    phone: &str,
    notes: Option<&str>,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO customers (name, phone, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, 1000, 1000)",
        (name, phone, notes),
    )
}

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
    assert_eq!(current_schema_version(&connection), 5);
    assert!(table_exists(&connection, "customers"));
    assert!(table_exists(&connection, "motorcycle_makes"));
    assert!(table_exists(&connection, "motorcycle_colors"));
    assert!(table_exists(&connection, "jordan_plate_codes"));
    assert!(table_exists(&connection, "motorcycles"));
    assert!(table_exists(&connection, "service_visits"));
    assert!(table_exists(&connection, "invoices"));
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
    assert_eq!(current_schema_version(&connection), 5);
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
    assert_eq!(current_schema_version(&connection), 5);
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
fn version_two_database_upgrades_to_latest_and_preserves_customer_and_motorcycle() {
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
    assert_eq!(current_schema_version(&connection), 5);
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
fn migration_three_stamps_version_three_before_migration_four_runs() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");

    // # Act
    stop_after_migration_three(&mut connection);

    // # Assert
    assert_eq!(current_schema_version(&connection), 3);
    assert!(trigger_exists(&connection, INSERT_VALIDATION_TRIGGER));
    assert!(trigger_exists(&connection, UPDATE_VALIDATION_TRIGGER));
    assert!(!column_exists(&connection, "motorcycles", "chassis_number"));
}

#[test]
fn migration_four_stamps_version_four_before_migration_five_runs() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");

    // # Act
    stop_after_migration_four(&mut connection);

    // # Assert
    assert_eq!(current_schema_version(&connection), 4);
    assert!(column_exists(&connection, "motorcycles", "chassis_number"));
    assert!(!table_exists(&connection, "service_visits"));
    assert!(!table_exists(&connection, "invoices"));
}

#[test]
fn version_three_database_upgrades_to_four_preserving_motorcycles_and_autoincrement() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    stop_after_migration_three(&mut connection);

    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES ('Ahmad', '+962791234567', 1000, 1000)",
            [],
        )
        .expect("customer should be inserted");
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
        .execute_batch(&format!(
            "INSERT INTO motorcycles (
                id, customer_id, make_id, model, year, plate_code_id, plate_number,
                vin, color_id, notes, created_at, updated_at, archived_at
             ) VALUES
                (10, {customer_id}, {make_id}, 'Plate Model', 2001, {plate_code_id}, 101,
                 NULL, {color_id}, 'Plate notes', 1100, 1200, NULL),
                (20, {customer_id}, {make_id}, 'VIN Model', 2002, NULL, NULL,
                 '1HGCM82633A004352', {color_id}, 'VIN notes', 2100, 2200, NULL),
                (30, {customer_id}, {make_id}, 'Combined Model', 2003, {plate_code_id}, 303,
                 'ABCDEFGHJKLMNPRST', {color_id}, NULL, 3100, 3200, NULL),
                (40, {customer_id}, {make_id}, 'Archived Model', NULL, {plate_code_id}, 404,
                 NULL, {color_id}, 'Archived notes', 4100, 4200, 4300);"
        ))
        .expect("representative version-three motorcycles should be inserted");
    let before = motorcycle_snapshots(&connection, "NULL");

    connection
        .execute_batch("CREATE VIEW service_visits AS SELECT 1 AS migration_blocker;")
        .expect("migration-five blocker should be created");

    // # Act
    let result = migrate_database(&mut connection);

    // # Assert
    assert!(result.is_err());
    connection
        .execute_batch("DROP VIEW service_visits;")
        .expect("migration-five blocker should be removed");

    assert_eq!(current_schema_version(&connection), 4);
    assert!(column_exists(&connection, "motorcycles", "chassis_number"));
    assert_eq!(motorcycle_snapshots(&connection, "chassis_number"), before);
    assert!(before
        .iter()
        .all(|motorcycle| motorcycle.chassis_number.is_none()));
    assert!(motorcycle_column_is_indexed(&connection, "customer_id"));
    assert!(motorcycle_column_is_indexed(&connection, "make_id"));
    let joined_count: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM motorcycles m
             JOIN customers c ON c.id = m.customer_id
             JOIN motorcycle_makes mk ON mk.id = m.make_id
             JOIN motorcycle_colors co ON co.id = m.color_id",
            [],
            |row| row.get(0),
        )
        .expect("preserved relationships should remain queryable");
    assert_eq!(joined_count, 4);
    assert!(connection
        .execute("DELETE FROM customers WHERE id = ?1", [customer_id])
        .is_err());

    connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, chassis_number, color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'New Model', 'FRAME/NEW-41', ?3, 5100, 5200)",
            (customer_id, make_id, color_id),
        )
        .expect("post-migration chassis-only motorcycle should be inserted");
    assert!(connection.last_insert_rowid() > 40);
}

#[test]
fn version_four_database_upgrades_to_five_preserving_customer_and_motorcycle() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    stop_after_migration_four(&mut connection);
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES ('Ahmad', '+962791234567', 1000, 1000)",
            [],
        )
        .expect("customer should be inserted");
    let customer_id = connection.last_insert_rowid();
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .expect("make should exist");
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .expect("color should exist");
    connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, chassis_number, color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'Legacy', 'FRAME/LEGACY', ?3, 1000, 1000)",
            (customer_id, make_id, color_id),
        )
        .expect("motorcycle should be inserted");
    let motorcycle_id = connection.last_insert_rowid();

    // # Act
    migrate_database(&mut connection).expect("version-four database should upgrade");

    // # Assert
    assert_eq!(current_schema_version(&connection), 5);
    assert!(table_exists(&connection, "service_visits"));
    assert!(table_exists(&connection, "invoices"));
    let preserved: (i64, i64, String) = connection
        .query_row(
            "SELECT id, customer_id, chassis_number FROM motorcycles WHERE id = ?1",
            [motorcycle_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("motorcycle should survive");
    assert_eq!(
        preserved,
        (motorcycle_id, customer_id, "FRAME/LEGACY".to_string())
    );
}

#[test]
fn failed_migration_five_rolls_back_all_new_schema_objects() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    stop_after_migration_four(&mut connection);
    connection
        .execute_batch("CREATE INDEX idx_service_visits_motorcycle_id ON motorcycles(customer_id);")
        .expect("late migration-five blocker should be created");
    let schema_before = schema_objects(&connection);

    // # Act
    let result = migrate_database(&mut connection);

    // # Assert
    assert!(result.is_err());
    assert_eq!(current_schema_version(&connection), 4);
    assert_eq!(schema_objects(&connection), schema_before);
    assert!(!table_exists(&connection, "service_visits"));
    assert!(!table_exists(&connection, "invoices"));
    for object in [
        "one_active_service_visit_per_motorcycle",
        "validate_service_visit_owner_v5",
        "create_draft_invoice_for_service_visit_v5",
        "prevent_service_visit_delete_v5",
        "prevent_invoice_delete_v5",
    ] {
        assert!(!schema_object_exists(&connection, object));
    }
    connection
        .execute_batch("DROP INDEX idx_service_visits_motorcycle_id;")
        .expect("migration-five blocker should be removed");
}

#[test]
fn failed_migration_four_rolls_back_to_the_original_motorcycles_table() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    stop_after_migration_three(&mut connection);
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES ('Ahmad', '+962791234567', 1000, 1000)",
            [],
        )
        .expect("customer should be inserted");
    let customer_id = connection.last_insert_rowid();
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
                customer_id, make_id, model, vin, color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'Legacy Model', '1HGCM82633A004352', ?3, 1000, 1000)",
            (customer_id, make_id, color_id),
        )
        .expect("legacy motorcycle should be inserted");
    let before = motorcycle_snapshots(&connection, "NULL");
    connection
        .execute_batch(
            "DROP INDEX idx_motorcycles_make_id;
             CREATE INDEX idx_motorcycles_make_id ON customers(name);",
        )
        .expect("late migration-four index blocker should be created");

    // # Act
    let result = migrate_database(&mut connection);

    // # Assert
    assert!(result.is_err());
    assert_eq!(current_schema_version(&connection), 3);
    assert!(!column_exists(&connection, "motorcycles", "chassis_number"));
    assert_eq!(motorcycle_snapshots(&connection, "NULL"), before);
    assert!(table_exists(&connection, "motorcycles"));
    assert!(!schema_object_exists(&connection, "motorcycles_v4"));
    assert!(motorcycle_column_is_indexed(&connection, "customer_id"));
    connection
        .execute_batch(
            "DROP INDEX idx_motorcycles_make_id;
             CREATE INDEX idx_motorcycles_make_id ON motorcycles(make_id);",
        )
        .expect("migration-four index blocker should be removed");
}

#[test]
fn version_five_migration_is_a_no_op_when_rerun() {
    // # Arrange
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    migrate_database(&mut connection).expect("initial migration should succeed");
    let schema_before = schema_objects(&connection);

    // # Act
    migrate_database(&mut connection).expect("version-five rerun should succeed");

    // # Assert
    assert_eq!(current_schema_version(&connection), 5);
    assert_eq!(schema_objects(&connection), schema_before);
}

#[test]
fn databases_newer_than_supported_are_rejected_without_modification() {
    for future_version in [6_i64, 999_i64] {
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
                max_supported: 5
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

fn stop_after_migration_three(connection: &mut rusqlite::Connection) {
    connection
        .execute_batch("CREATE VIEW motorcycles_v4 AS SELECT 1 AS migration_blocker;")
        .expect("migration-four blocker should be created");

    let result = migrate_database(connection);
    assert!(
        result.is_err(),
        "the migration-four blocker should stop the migration runner"
    );
    assert_eq!(current_schema_version(connection), 3);

    connection
        .execute_batch("DROP VIEW motorcycles_v4;")
        .expect("migration-four blocker should be removed");
}

fn stop_after_migration_four(connection: &mut rusqlite::Connection) {
    connection
        .execute_batch("CREATE VIEW service_visits AS SELECT 1 AS migration_blocker;")
        .expect("migration-five blocker should be created");

    let result = migrate_database(connection);
    assert!(
        result.is_err(),
        "the migration-five blocker should stop the migration runner"
    );
    assert_eq!(current_schema_version(connection), 4);

    connection
        .execute_batch("DROP VIEW service_visits;")
        .expect("migration-five blocker should be removed");
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

fn column_exists(connection: &rusqlite::Connection, table_name: &str, column_name: &str) -> bool {
    connection
        .query_row(
            "SELECT EXISTS (
                SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2
             )",
            (table_name, column_name),
            |row| row.get(0),
        )
        .expect("column existence should be queryable")
}

fn schema_object_exists(connection: &rusqlite::Connection, object_name: &str) -> bool {
    connection
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE name = ?1)",
            [object_name],
            |row| row.get(0),
        )
        .expect("schema object existence should be queryable")
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

fn motorcycle_column_is_indexed(connection: &rusqlite::Connection, column: &str) -> bool {
    connection
        .query_row(
            "SELECT EXISTS (
                SELECT 1
                FROM pragma_index_list('motorcycles') AS index_list
                JOIN pragma_index_info(index_list.name) AS index_info ON TRUE
                WHERE index_info.name = ?1
             )",
            [column],
            |row| row.get(0),
        )
        .expect("motorcycle indexes should be queryable")
}

#[derive(Debug, PartialEq, Eq)]
struct MotorcycleSnapshot {
    id: i64,
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
    created_at: i64,
    updated_at: i64,
    archived_at: Option<i64>,
}

fn motorcycle_snapshots(
    connection: &rusqlite::Connection,
    chassis_expression: &str,
) -> Vec<MotorcycleSnapshot> {
    let mut statement = connection
        .prepare(&format!(
            "SELECT
                id, customer_id, make_id, model, year, plate_code_id, plate_number,
                vin, {chassis_expression}, color_id, notes, created_at, updated_at, archived_at
             FROM motorcycles
             ORDER BY id"
        ))
        .expect("motorcycle snapshots should be queryable");

    statement
        .query_map([], |row| {
            Ok(MotorcycleSnapshot {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                make_id: row.get(2)?,
                model: row.get(3)?,
                year: row.get(4)?,
                plate_code_id: row.get(5)?,
                plate_number: row.get(6)?,
                vin: row.get(7)?,
                chassis_number: row.get(8)?,
                color_id: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                archived_at: row.get(13)?,
            })
        })
        .expect("motorcycle snapshot rows should be readable")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("motorcycle snapshots should be collected")
}

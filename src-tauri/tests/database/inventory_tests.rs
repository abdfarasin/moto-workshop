use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::db::{migrate_database, open_database};

const MAX: i64 = 1_000_000_000;

#[test]
fn units_are_seeded_and_database_enforces_catalog_rules() {
    // # Arrange
    let fixture = fixture();

    // # Act
    let units = unit_snapshots(&fixture.connection);

    // # Assert
    assert_eq!(
        units,
        vec![("Liter".to_string(), 1_000, 1), ("Piece".to_string(), 1, 1)]
    );
    fixture
        .connection
        .execute(
            "INSERT INTO inventory_units (name, quantity_scale) VALUES ('Pack', 10)",
            [],
        )
        .expect("valid custom unit should be accepted");
    assert!(fixture
        .connection
        .execute(
            "INSERT INTO inventory_units (name, quantity_scale) VALUES ('piece', 1)",
            []
        )
        .is_err());
    for invalid_name in ["", " padded", "padded "] {
        assert!(fixture
            .connection
            .execute(
                "INSERT INTO inventory_units (name, quantity_scale) VALUES (?1, 1)",
                [invalid_name]
            )
            .is_err());
    }
    assert!(fixture
        .connection
        .execute(
            "INSERT INTO inventory_units (name, quantity_scale) VALUES (?1, 1)",
            [&"u".repeat(41)]
        )
        .is_err());
    for invalid_scale in [0, 2, 50, 10_000] {
        assert!(fixture
            .connection
            .execute(
                "INSERT INTO inventory_units (name, quantity_scale) VALUES (?1, ?2)",
                (format!("Scale {invalid_scale}"), invalid_scale)
            )
            .is_err());
    }
    for invalid_active in [-1, 2] {
        assert!(fixture
            .connection
            .execute(
                "INSERT INTO inventory_units (name, quantity_scale, active) VALUES (?1, 1, ?2)",
                (format!("Active {invalid_active}"), invalid_active)
            )
            .is_err());
    }
    assert!(fixture
        .connection
        .execute("DELETE FROM inventory_units WHERE name = 'Pack'", [])
        .is_err());
}

#[test]
fn inventory_items_support_units_duplicate_names_and_nullable_unique_skus() {
    // # Arrange
    let fixture = fixture();

    // # Act
    let first_id = insert_item(
        &fixture.connection,
        "Oil Filter",
        Some("FILTER-01"),
        fixture.piece_id,
        (None, 7_000, 5),
        None,
    )
    .expect("Piece item should insert");
    let second_id = insert_item(
        &fixture.connection,
        "Oil Filter",
        None,
        fixture.liter_id,
        (Some(5_000), 7_000, 2_500),
        Some("Bulk oil"),
    )
    .expect("Liter item with duplicate name and NULL SKU should insert");
    insert_item(
        &fixture.connection,
        "Oil Filter",
        None,
        fixture.piece_id,
        (None, 0, 0),
        None,
    )
    .expect("another NULL SKU should insert");

    // # Assert
    assert_eq!((first_id, second_id), (1, 2));
    assert!(insert_item(
        &fixture.connection,
        "Other",
        Some("filter-01"),
        fixture.piece_id,
        (None, 0, 0),
        None,
    )
    .is_err());
    assert!(insert_item(
        &fixture.connection,
        "Missing unit",
        Some("MISSING"),
        999_999,
        (None, 0, 0),
        None,
    )
    .is_err());
}

#[test]
fn inventory_item_database_enforces_canonical_text_numeric_bounds_and_history() {
    let invalid_names = ["", "   "];
    for name in invalid_names {
        let fixture = fixture();
        assert!(insert_item(
            &fixture.connection,
            name,
            None,
            fixture.piece_id,
            (None, 0, 0),
            None,
        )
        .is_err());
    }
    for name in [" padded", "padded "] {
        let fixture = fixture();
        assert!(insert_item(
            &fixture.connection,
            name,
            None,
            fixture.piece_id,
            (None, 0, 0),
            None,
        )
        .is_err());
    }
    let boundary_fixture = fixture();
    assert!(insert_item(
        &boundary_fixture.connection,
        &"a".repeat(150),
        Some(&"S".repeat(64)),
        boundary_fixture.piece_id,
        (Some(MAX), MAX, MAX),
        Some(&"n".repeat(2_000)),
    )
    .is_ok());

    for sql in [
        "INSERT INTO inventory_items (name, sku, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Item', ' padded', 1, 0, 1, 1)",
        "INSERT INTO inventory_items (name, sku, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Item', 'XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX', 1, 0, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_purchase_price_fils, default_selling_price_fils, created_at, updated_at) VALUES ('Item', 1, -1, 0, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_purchase_price_fils, default_selling_price_fils, created_at, updated_at) VALUES ('Item', 1, 1000000001, 0, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_purchase_price_fils, default_selling_price_fils, created_at, updated_at) VALUES ('Item', 1, 1.5, 0, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Item', 1, -1, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Item', 1, 1000000001, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Item', 1, 1.5, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, minimum_stock_quantity, created_at, updated_at) VALUES ('Item', 1, 0, -1, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, minimum_stock_quantity, created_at, updated_at) VALUES ('Item', 1, 0, 1000000001, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, minimum_stock_quantity, created_at, updated_at) VALUES ('Item', 1, 0, 1.5, 1, 1)",
        "INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, notes, created_at, updated_at) VALUES ('Item', 1, 0, ' padded', 1, 1)",
    ] {
        let fixture = fixture();
        assert!(fixture.connection.execute(sql, []).is_err(), "SQL: {sql}");
    }

    let overlong_fixture = fixture();
    assert!(insert_item(
        &overlong_fixture.connection,
        &"a".repeat(151),
        None,
        overlong_fixture.piece_id,
        (None, 0, 0),
        None,
    )
    .is_err());
    assert!(insert_item(
        &overlong_fixture.connection,
        "Notes",
        None,
        overlong_fixture.piece_id,
        (None, 0, 0),
        Some(&"n".repeat(2_001)),
    )
    .is_err());

    let archive_fixture = fixture();
    let item_id = insert_item(
        &archive_fixture.connection,
        "Archived",
        Some("ARCHIVED-SKU"),
        archive_fixture.piece_id,
        (None, 0, 0),
        None,
    )
    .expect("item should insert");
    archive_fixture
        .connection
        .execute(
            "UPDATE inventory_items SET archived_at = 2000 WHERE id = ?1",
            [item_id],
        )
        .expect("item should archive");
    let archived_at: Option<i64> = archive_fixture
        .connection
        .query_row(
            "SELECT archived_at FROM inventory_items WHERE id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .expect("archived item should remain queryable");
    assert_eq!(archived_at, Some(2_000));
    assert!(archive_fixture
        .connection
        .execute("DELETE FROM inventory_items WHERE id = ?1", [item_id])
        .is_err());
    assert!(insert_item(
        &archive_fixture.connection,
        "Replacement",
        Some("archived-sku"),
        archive_fixture.piece_id,
        (None, 0, 0),
        None,
    )
    .is_err());
}

#[test]
fn unit_meaning_is_protected_at_the_correct_history_boundaries() {
    // # Arrange
    let fixture = fixture();
    fixture
        .connection
        .execute(
            "INSERT INTO inventory_units (name, quantity_scale) VALUES ('Pack', 10)",
            [],
        )
        .expect("custom unit should insert");
    let pack_id = fixture.connection.last_insert_rowid();
    let item_id = insert_item(
        &fixture.connection,
        "Correctable",
        None,
        pack_id,
        (None, 0, 0),
        None,
    )
    .expect("item should insert");

    // # Act / # Assert
    assert!(fixture
        .connection
        .execute(
            "UPDATE inventory_units SET quantity_scale = 100 WHERE id = ?1",
            [pack_id]
        )
        .is_err());
    fixture
        .connection
        .execute(
            "UPDATE inventory_items SET unit_id = ?1 WHERE id = ?2",
            (fixture.piece_id, item_id),
        )
        .expect("item unit may be corrected before movements");
    insert_movement(
        &fixture.connection,
        item_id,
        "OPENING_STOCK",
        1,
        None,
        1_000,
    )
    .expect("movement should insert");
    assert!(fixture
        .connection
        .execute(
            "UPDATE inventory_items SET unit_id = ?1 WHERE id = ?2",
            (fixture.liter_id, item_id)
        )
        .is_err());
}

#[test]
fn stock_movement_database_enforces_types_signs_bounds_and_foreign_keys() {
    // # Arrange
    let fixture = fixture();
    let item_id = basic_item(&fixture, "Ledger item");

    // # Act / # Assert
    for (movement_type, delta) in [
        ("OPENING_STOCK", 1),
        ("OPENING_STOCK", MAX),
        ("PURCHASE", 1),
        ("PURCHASE", MAX),
        ("ADJUSTMENT_IN", 1),
        ("ADJUSTMENT_IN", MAX),
        ("ADJUSTMENT_OUT", -1),
        ("ADJUSTMENT_OUT", -MAX),
    ] {
        insert_movement(
            &fixture.connection,
            item_id,
            movement_type,
            delta,
            Some("Valid"),
            1_000,
        )
        .expect("valid movement should insert");
    }
    for (movement_type, delta) in [
        ("UNKNOWN", 1),
        ("OPENING_STOCK", 0),
        ("OPENING_STOCK", -1),
        ("OPENING_STOCK", MAX + 1),
        ("PURCHASE", -1),
        ("ADJUSTMENT_IN", -1),
        ("ADJUSTMENT_OUT", 1),
        ("ADJUSTMENT_OUT", -MAX - 1),
    ] {
        assert!(insert_movement(
            &fixture.connection,
            item_id,
            movement_type,
            delta,
            None,
            1_000,
        )
        .is_err());
    }
    assert!(insert_movement(&fixture.connection, 999_999, "PURCHASE", 1, None, 1_000,).is_err());
    assert!(fixture
        .connection
        .execute(
            "INSERT INTO stock_movements (inventory_item_id, movement_type, quantity_delta, created_at)
             VALUES (?1, 'PURCHASE', 'not-an-integer', 1000)",
            [item_id]
        )
        .is_err());
    assert!(fixture
        .connection
        .execute(
            "INSERT INTO stock_movements (inventory_item_id, movement_type, quantity_delta, created_at)
             VALUES (?1, 'PURCHASE', 1.5, 1000)",
            [item_id]
        )
        .is_err());
    assert!(fixture
        .connection
        .execute(
            "INSERT INTO stock_movements (inventory_item_id, movement_type, quantity_delta, created_at)
             VALUES (?1, 'PURCHASE', 1, 'not-an-integer')",
            [item_id]
        )
        .is_err());
    assert!(insert_movement(
        &fixture.connection,
        item_id,
        "PURCHASE",
        1,
        Some(" padded"),
        1_000,
    )
    .is_err());
    assert!(insert_movement(
        &fixture.connection,
        item_id,
        "PURCHASE",
        1,
        Some(&"n".repeat(2_001)),
        1_000,
    )
    .is_err());
    assert!(insert_movement(&fixture.connection, item_id, "PURCHASE", 1, None, -1,).is_err());
}

#[test]
fn stock_movements_are_immutable_and_use_compensating_entries() {
    // # Arrange
    let fixture = fixture();
    let item_id = basic_item(&fixture, "Immutable ledger");
    insert_movement(
        &fixture.connection,
        item_id,
        "ADJUSTMENT_OUT",
        -5,
        Some("Mistake"),
        1_000,
    )
    .expect("movement should insert");
    let movement_id = fixture.connection.last_insert_rowid();

    // # Act / # Assert
    assert!(fixture
        .connection
        .execute(
            "UPDATE stock_movements SET quantity_delta = -4 WHERE id = ?1",
            [movement_id]
        )
        .is_err());
    assert!(fixture
        .connection
        .execute("DELETE FROM stock_movements WHERE id = ?1", [movement_id])
        .is_err());
    insert_movement(
        &fixture.connection,
        item_id,
        "ADJUSTMENT_IN",
        5,
        Some("Correction"),
        2_000,
    )
    .expect("compensating movement should insert");
    assert_eq!(stock(&fixture.connection, item_id), 0);
    let movement_count: i64 = fixture
        .connection
        .query_row(
            "SELECT COUNT(*) FROM stock_movements WHERE inventory_item_id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .expect("movement count should be queryable");
    assert_eq!(movement_count, 2);
}

#[test]
fn ledger_derives_zero_positive_and_negative_stock_without_mixing_items() {
    // # Arrange
    let fixture = fixture();
    let first_id = basic_item(&fixture, "First");
    let second_id = basic_item(&fixture, "Second");
    assert_eq!(stock(&fixture.connection, first_id), 0);

    // # Act
    insert_movement(&fixture.connection, first_id, "OPENING_STOCK", 10, None, 1).unwrap();
    assert_eq!(stock(&fixture.connection, first_id), 10);
    insert_movement(&fixture.connection, first_id, "PURCHASE", 5, None, 2).unwrap();
    insert_movement(&fixture.connection, first_id, "ADJUSTMENT_OUT", -2, None, 3).unwrap();
    insert_movement(&fixture.connection, second_id, "OPENING_STOCK", 1, None, 1).unwrap();
    insert_movement(
        &fixture.connection,
        second_id,
        "ADJUSTMENT_OUT",
        -2,
        None,
        2,
    )
    .unwrap();

    // # Assert
    assert_eq!(stock(&fixture.connection, first_id), 13);
    assert_eq!(stock(&fixture.connection, second_id), -1);
}

#[test]
fn liter_scaled_ledger_preserves_three_decimal_quantity_exactly() {
    // # Arrange
    let fixture = fixture();
    let item_id = insert_item(
        &fixture.connection,
        "Engine Oil",
        Some("OIL-LITER"),
        fixture.liter_id,
        (Some(5_000), 7_000, 2_500),
        None,
    )
    .expect("Liter item should insert");

    // # Act
    insert_movement(
        &fixture.connection,
        item_id,
        "OPENING_STOCK",
        5_000,
        None,
        1,
    )
    .unwrap();
    insert_movement(
        &fixture.connection,
        item_id,
        "ADJUSTMENT_OUT",
        -1_250,
        None,
        2,
    )
    .unwrap();

    // # Assert
    assert_eq!(stock(&fixture.connection, item_id), 3_750);
    let scale: i64 = fixture
        .connection
        .query_row(
            "SELECT u.quantity_scale
             FROM inventory_items i
             JOIN inventory_units u ON u.id = i.unit_id
             WHERE i.id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .expect("scale should be queryable");
    assert_eq!(scale, 1_000);
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
    piece_id: i64,
    liter_id: i64,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    migrate_database(&mut connection).expect("database should migrate");
    let piece_id = unit_id(&connection, "Piece");
    let liter_id = unit_id(&connection, "Liter");
    Fixture {
        _temp_dir: temp_dir,
        connection,
        piece_id,
        liter_id,
    }
}

fn unit_id(connection: &Connection, name: &str) -> i64 {
    connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
        .expect("seeded unit should exist")
}

fn unit_snapshots(connection: &Connection) -> Vec<(String, i64, i64)> {
    let mut statement = connection
        .prepare("SELECT name, quantity_scale, active FROM inventory_units ORDER BY name")
        .expect("units should be queryable");
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("unit rows should be readable")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("unit rows should collect")
}

fn insert_item(
    connection: &Connection,
    name: &str,
    sku: Option<&str>,
    unit_id: i64,
    prices_and_minimum: (Option<i64>, i64, i64),
    notes: Option<&str>,
) -> rusqlite::Result<i64> {
    let (purchase_price, selling_price, minimum_stock) = prices_and_minimum;
    connection.execute(
        "INSERT INTO inventory_items (
            name, sku, unit_id, default_purchase_price_fils,
            default_selling_price_fils, minimum_stock_quantity,
            notes, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1000, 1000)",
        params![
            name,
            sku,
            unit_id,
            purchase_price,
            selling_price,
            minimum_stock,
            notes
        ],
    )?;
    Ok(connection.last_insert_rowid())
}

fn basic_item(fixture: &Fixture, name: &str) -> i64 {
    insert_item(
        &fixture.connection,
        name,
        None,
        fixture.piece_id,
        (None, 0, 0),
        None,
    )
    .expect("item should insert")
}

fn insert_movement(
    connection: &Connection,
    item_id: i64,
    movement_type: &str,
    quantity_delta: i64,
    notes: Option<&str>,
    created_at: i64,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO stock_movements (
            inventory_item_id, movement_type, quantity_delta, notes, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![item_id, movement_type, quantity_delta, notes, created_at],
    )
}

fn stock(connection: &Connection, item_id: i64) -> i64 {
    connection
        .query_row(
            "SELECT COALESCE(SUM(quantity_delta), 0)
             FROM stock_movements WHERE inventory_item_id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .expect("stock should be derived from the ledger")
}

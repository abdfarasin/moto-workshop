use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::db::{migrate_database, open_database};

#[test]
fn creating_piece_part_snapshots_total_and_automatic_usage() {
    // # Arrange
    let fixture = fixture();

    // # Act
    let part_id = insert_part(
        &fixture.connection,
        fixture.open_visit_id,
        fixture.piece_item_id,
        "Oil Filter",
        "Piece",
        2,
        (1, 3_500, 7_000, 2_000),
    )
    .expect("part should insert");

    // # Assert
    assert_eq!(part_id, 1);
    let part: (
        String,
        String,
        i64,
        i64,
        i64,
        i64,
        String,
        Option<i64>,
        Option<String>,
    ) = fixture
        .connection
        .query_row(
            "SELECT item_name, unit_name, quantity, quantity_scale, unit_price_fils,
                line_total_fils, status, voided_at, void_reason
         FROM service_visit_parts WHERE id = ?1",
            [part_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .expect("part should be queryable");
    assert_eq!(
        part,
        (
            "Oil Filter".into(),
            "Piece".into(),
            2,
            1,
            3_500,
            7_000,
            "ACTIVE".into(),
            None,
            None
        )
    );
    assert_eq!(
        movement_snapshot(&fixture.connection, part_id, "SERVICE_USAGE"),
        (fixture.piece_item_id, -2, 2_000)
    );
}

#[test]
fn snapshot_validation_fractional_rounding_and_catalog_changes_preserve_history() {
    let fixture = fixture();
    let liter_id: i64 = fixture
        .connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name='Liter'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    fixture.connection.execute("INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Engine Oil', ?1, 7000, 1, 1)", [liter_id]).unwrap();
    let item_id = fixture.connection.last_insert_rowid();
    let part_id = insert_part(
        &fixture.connection,
        fixture.open_visit_id,
        item_id,
        "Engine Oil",
        "Liter",
        333,
        (1_000, 5_500, 1_832, 2_000),
    )
    .unwrap();
    assert_eq!(
        movement_snapshot(&fixture.connection, part_id, "SERVICE_USAGE"),
        (item_id, -333, 2_000)
    );
    for (name, unit, scale) in [
        ("Wrong", "Liter", 1_000),
        ("Engine Oil", "Wrong", 1_000),
        ("Engine Oil", "Liter", 100),
    ] {
        assert!(insert_part(
            &fixture.connection,
            fixture.open_visit_id,
            item_id,
            name,
            unit,
            1,
            (scale, 1, 1, 3_000)
        )
        .is_err());
    }
    fixture.connection.execute("UPDATE inventory_items SET name='Renamed Oil', default_selling_price_fils=9000 WHERE id=?1", [item_id]).unwrap();
    fixture
        .connection
        .execute(
            "UPDATE inventory_units SET name='Litre' WHERE id=?1",
            [liter_id],
        )
        .unwrap();
    let snapshot: (String, String, i64) = fixture
        .connection
        .query_row(
            "SELECT item_name, unit_name, unit_price_fils FROM service_visit_parts WHERE id=?1",
            [part_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(snapshot, ("Engine Oil".into(), "Liter".into(), 5_500));
}

#[test]
fn invalid_sources_numeric_values_and_line_totals_are_rejected() {
    let boundaries = fixture();
    assert!(insert_part(
        &boundaries.connection,
        boundaries.open_visit_id,
        boundaries.piece_item_id,
        "Oil Filter",
        "Piece",
        1_000_000_000,
        (1, 0, 0, 10),
    )
    .is_ok());
    assert!(insert_part(
        &boundaries.connection,
        boundaries.open_visit_id,
        boundaries.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1_000_000_000, 1_000_000_000, 11),
    )
    .is_ok());
    for mutate in 0..8 {
        let fixture = fixture();
        let mut values = (1_i64, 1_i64, 1_i64, 1_i64);
        match mutate {
            0 => values.0 = 0,
            1 => values.0 = -1,
            2 => values.0 = 1_000_000_001,
            3 => values.1 = 2,
            4 => values.2 = -1,
            5 => values.2 = 1_000_000_001,
            6 => values.3 = 2,
            _ => values.3 = -1,
        }
        assert!(insert_part(
            &fixture.connection,
            fixture.open_visit_id,
            fixture.piece_item_id,
            "Oil Filter",
            "Piece",
            values.0,
            (values.1, values.2, values.3, 2_000)
        )
        .is_err());
    }
    let archived = fixture();
    archived
        .connection
        .execute(
            "UPDATE inventory_items SET archived_at=2 WHERE id=?1",
            [archived.piece_item_id],
        )
        .unwrap();
    assert!(insert_part(
        &archived.connection,
        archived.open_visit_id,
        archived.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1, 1, 2_000)
    )
    .is_err());
    assert!(insert_part(
        &archived.connection,
        99999,
        archived.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1, 1, 2_000)
    )
    .is_err());
    assert!(insert_part(
        &archived.connection,
        archived.open_visit_id,
        99999,
        "Missing",
        "Piece",
        1,
        (1, 1, 1, 2_000)
    )
    .is_err());
}

#[test]
fn ready_visit_can_add_and_void_part_while_terminal_visit_cannot_mutate_parts() {
    let ready = fixture();
    ready.connection.execute("UPDATE service_visits SET status='READY_FOR_PICKUP', completed_at=2, work_performed='Done' WHERE id=?1", [ready.open_visit_id]).unwrap();
    let part_id = insert_part(
        &ready.connection,
        ready.open_visit_id,
        ready.piece_item_id,
        "Oil Filter",
        "Piece",
        2,
        (1, 1, 2, 3),
    )
    .unwrap();
    ready
        .connection
        .execute(
            "UPDATE service_visit_parts SET status='VOIDED', voided_at=4 WHERE id=?1",
            [part_id],
        )
        .unwrap();
    assert_eq!(
        movement_snapshot(&ready.connection, part_id, "SERVICE_USAGE_REVERSAL"),
        (ready.piece_item_id, 2, 4)
    );
    assert!(ready
        .connection
        .execute(
            "UPDATE service_visit_parts SET void_reason='again' WHERE id=?1",
            [part_id]
        )
        .is_err());
    assert!(ready
        .connection
        .execute("DELETE FROM service_visit_parts WHERE id=?1", [part_id])
        .is_err());

    let closed = fixture();
    closed.connection.execute("UPDATE service_visits SET status='READY_FOR_PICKUP', completed_at=2, work_performed='Done' WHERE id=?1", [closed.open_visit_id]).unwrap();
    let active_id = insert_part(
        &closed.connection,
        closed.open_visit_id,
        closed.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1, 1, 3),
    )
    .unwrap();
    closed
        .connection
        .execute(
            "UPDATE service_visits SET status='CLOSED', closed_at=4 WHERE id=?1",
            [closed.open_visit_id],
        )
        .unwrap();
    assert!(insert_part(
        &closed.connection,
        closed.open_visit_id,
        closed.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1, 1, 5)
    )
    .is_err());
    assert!(closed
        .connection
        .execute(
            "UPDATE service_visit_parts SET status='VOIDED', voided_at=5 WHERE id=?1",
            [active_id]
        )
        .is_err());
}

#[test]
fn linked_movements_are_unique_validated_immutable_and_allow_negative_stock() {
    let fixture = fixture();
    fixture.connection.execute("INSERT INTO stock_movements (inventory_item_id, movement_type, quantity_delta, created_at) VALUES (?1, 'OPENING_STOCK', 1, 1)", [fixture.piece_item_id]).unwrap();
    let part_id = insert_part(
        &fixture.connection,
        fixture.open_visit_id,
        fixture.piece_item_id,
        "Oil Filter",
        "Piece",
        2,
        (1, 1, 2, 2),
    )
    .unwrap();
    let stock: i64 = fixture
        .connection
        .query_row(
            "SELECT SUM(quantity_delta) FROM stock_movements WHERE inventory_item_id=?1",
            [fixture.piece_item_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stock, -1);
    for (kind, item_id, delta) in [
        ("SERVICE_USAGE", fixture.piece_item_id, -2),
        ("SERVICE_USAGE", fixture.piece_item_id, -1),
        ("SERVICE_USAGE", 99999, -2),
        ("SERVICE_USAGE_REVERSAL", fixture.piece_item_id, 2),
    ] {
        assert!(fixture.connection.execute("INSERT INTO stock_movements (inventory_item_id, service_visit_part_id, movement_type, quantity_delta, created_at) VALUES (?1, ?2, ?3, ?4, 3)", params![item_id, part_id, kind, delta]).is_err());
    }
    let usage_id: i64 = fixture
        .connection
        .query_row(
            "SELECT id FROM stock_movements WHERE service_visit_part_id=?1",
            [part_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(fixture
        .connection
        .execute(
            "UPDATE stock_movements SET quantity_delta=-1 WHERE id=?1",
            [usage_id]
        )
        .is_err());
    assert!(fixture
        .connection
        .execute("DELETE FROM stock_movements WHERE id=?1", [usage_id])
        .is_err());
    fixture.connection.execute("UPDATE service_visit_parts SET status='VOIDED', voided_at=4, void_reason=NULL WHERE id=?1", [part_id]).unwrap();
    assert_eq!(
        fixture
            .connection
            .query_row(
                "SELECT SUM(quantity_delta) FROM stock_movements WHERE inventory_item_id=?1",
                [fixture.piece_item_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(fixture.connection.execute("INSERT INTO stock_movements (inventory_item_id, service_visit_part_id, movement_type, quantity_delta, created_at) VALUES (?1, ?2, 'SERVICE_USAGE_REVERSAL', 2, 5)", (fixture.piece_item_id, part_id)).is_err());
}

#[test]
fn database_rounding_and_integer_boundaries_match_the_domain_formula() {
    let fixture = fixture();
    let liter_id: i64 = fixture
        .connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name='Liter'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    fixture.connection.execute("INSERT INTO inventory_items (name, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Rounding Oil', ?1, 5500, 1, 1)", [liter_id]).unwrap();
    let rounding_item_id = fixture.connection.last_insert_rowid();
    for (quantity, total) in [(332, 1_826), (333, 1_832), (334, 1_837)] {
        insert_part(
            &fixture.connection,
            fixture.open_visit_id,
            rounding_item_id,
            "Rounding Oil",
            "Liter",
            quantity,
            (1_000, 5_500, total, 2_000 + quantity),
        )
        .expect("correct half-up total should insert");
    }
    assert!(insert_part(
        &fixture.connection,
        fixture.open_visit_id,
        rounding_item_id,
        "Rounding Oil",
        "Liter",
        333,
        (1_000, 5_500, 1_831, 4_000),
    )
    .is_err());
    for sql in [
        "INSERT INTO service_visit_parts (service_visit_id, inventory_item_id, item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils, created_at) VALUES (?1, ?2, 'Oil Filter', 'Piece', 1.5, 1, 1, 1, 5000)",
        "INSERT INTO service_visit_parts (service_visit_id, inventory_item_id, item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils, created_at) VALUES (?1, ?2, 'Oil Filter', 'Piece', 1, 1, 1.5, 1, 5000)",
        "INSERT INTO service_visit_parts (service_visit_id, inventory_item_id, item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils, created_at) VALUES (?1, ?2, 'Oil Filter', 'Piece', 1, 1, 1, 1.5, 5000)",
    ] {
        assert!(fixture.connection.execute(sql, (fixture.open_visit_id, fixture.piece_item_id)).is_err());
    }
}

#[test]
fn void_status_reason_chronology_and_snapshot_immutability_are_enforced() {
    let direct = fixture();
    let fixture = fixture();
    let part_id = insert_part(
        &fixture.connection,
        fixture.open_visit_id,
        fixture.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1, 1, 2_000),
    )
    .unwrap();
    for update in [
        "status='VOIDED'",
        "status='VOIDED', voided_at=1999",
        "status='VOIDED', voided_at=2000, void_reason=' padded'",
    ] {
        assert!(fixture
            .connection
            .execute(
                &format!("UPDATE service_visit_parts SET {update} WHERE id=?1"),
                [part_id]
            )
            .is_err());
    }
    assert!(fixture.connection.execute(
        "UPDATE service_visit_parts SET status='VOIDED', voided_at=2000, void_reason=?1 WHERE id=?2",
        (&"r".repeat(1_001), part_id),
    ).is_err());
    for update in [
        "service_visit_id=999",
        "inventory_item_id=999",
        "item_name='Changed'",
        "unit_name='Changed'",
        "quantity=2",
        "quantity_scale=10",
        "unit_price_fils=2",
        "line_total_fils=2",
        "created_at=1",
    ] {
        assert!(fixture
            .connection
            .execute(
                &format!("UPDATE service_visit_parts SET {update} WHERE id=?1"),
                [part_id]
            )
            .is_err());
    }
    fixture.connection.execute("UPDATE service_visit_parts SET status='VOIDED', voided_at=2000, void_reason='Wrong quantity' WHERE id=?1", [part_id]).unwrap();
    let reason: Option<String> = fixture
        .connection
        .query_row(
            "SELECT void_reason FROM service_visit_parts WHERE id=?1",
            [part_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(reason.as_deref(), Some("Wrong quantity"));

    assert!(direct.connection.execute(
        "INSERT INTO service_visit_parts (service_visit_id, inventory_item_id, item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils, status, voided_at, created_at) VALUES (?1, ?2, 'Oil Filter', 'Piece', 1, 1, 1, 1, 'VOIDED', 3, 2)",
        (direct.open_visit_id, direct.piece_item_id),
    ).is_err());
}

#[test]
fn cancelled_visit_rejects_new_parts_and_existing_part_voids() {
    let new_part = fixture();
    new_part.connection.execute("UPDATE service_visits SET status='CANCELLED', cancelled_at=2, cancellation_reason='Declined' WHERE id=?1", [new_part.open_visit_id]).unwrap();
    assert!(insert_part(
        &new_part.connection,
        new_part.open_visit_id,
        new_part.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1, 1, 3)
    )
    .is_err());

    let existing = fixture();
    let part_id = insert_part(
        &existing.connection,
        existing.open_visit_id,
        existing.piece_item_id,
        "Oil Filter",
        "Piece",
        1,
        (1, 1, 1, 2),
    )
    .unwrap();
    existing.connection.execute("UPDATE service_visits SET status='CANCELLED', cancelled_at=3, cancellation_reason='Declined' WHERE id=?1", [existing.open_visit_id]).unwrap();
    assert!(existing
        .connection
        .execute(
            "UPDATE service_visit_parts SET status='VOIDED', voided_at=4 WHERE id=?1",
            [part_id]
        )
        .is_err());
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
    open_visit_id: i64,
    piece_item_id: i64,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    connection.execute("INSERT INTO customers (name, phone, created_at, updated_at) VALUES ('Owner', '+962791234567', 1, 1)", []).unwrap();
    let customer_id = connection.last_insert_rowid();
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name='Honda'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name='Black'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    connection.execute("INSERT INTO motorcycles (customer_id, make_id, model, plate_number, chassis_number, color_id, created_at, updated_at) VALUES (?1, ?2, 'Bike', '1', 'FRAME/PART', ?3, 1, 1)", (customer_id, make_id, color_id)).unwrap();
    let motorcycle_id = connection.last_insert_rowid();
    connection.execute("INSERT INTO service_visits (motorcycle_id, owner_customer_id, status, opened_at, customer_complaint, created_at, updated_at) VALUES (?1, ?2, 'OPEN', 1, 'Repair', 1, 1)", (motorcycle_id, customer_id)).unwrap();
    let open_visit_id = connection.last_insert_rowid();
    let piece_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name='Piece'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    connection.execute("INSERT INTO inventory_items (name, sku, unit_id, default_selling_price_fils, created_at, updated_at) VALUES ('Oil Filter', 'FILTER', ?1, 4500, 1, 1)", [piece_id]).unwrap();
    let piece_item_id = connection.last_insert_rowid();
    Fixture {
        _temp_dir: temp_dir,
        connection,
        open_visit_id,
        piece_item_id,
    }
}

fn insert_part(
    connection: &Connection,
    visit_id: i64,
    item_id: i64,
    item_name: &str,
    unit_name: &str,
    quantity: i64,
    line: (i64, i64, i64, i64),
) -> rusqlite::Result<i64> {
    let (scale, price, total, created_at) = line;
    connection.execute("INSERT INTO service_visit_parts (service_visit_id, inventory_item_id, item_name, unit_name, quantity, quantity_scale, unit_price_fils, line_total_fils, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", params![visit_id, item_id, item_name, unit_name, quantity, scale, price, total, created_at])?;
    Ok(connection.last_insert_rowid())
}

fn movement_snapshot(
    connection: &Connection,
    part_id: i64,
    movement_type: &str,
) -> (i64, i64, i64) {
    connection.query_row("SELECT inventory_item_id, quantity_delta, created_at FROM stock_movements WHERE service_visit_part_id=?1 AND movement_type=?2", (part_id, movement_type), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap()
}

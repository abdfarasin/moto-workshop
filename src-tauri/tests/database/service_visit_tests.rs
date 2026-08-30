use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::db::{migrate_database, open_database};

#[test]
fn open_visit_requires_existing_motorcycle_and_matching_owner_and_creates_invoice() {
    // # Arrange
    let fixture = fixture();

    // # Act
    insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "Engine cuts out when hot",
        None,
        0_i64,
        1_000,
    )
    .expect("matching visit should be inserted");
    let visit_id = fixture.connection.last_insert_rowid();
    let owner_mismatch = insert_open_visit(
        &fixture.connection,
        fixture.second_motorcycle_id,
        fixture.owner_id,
        "Mismatch",
        None,
        0_i64,
        2_000,
    );
    let missing_motorcycle = insert_open_visit(
        &fixture.connection,
        999_999,
        fixture.owner_id,
        "Missing",
        None,
        0_i64,
        3_000,
    );
    let missing_owner = insert_open_visit(
        &fixture.connection,
        fixture.second_motorcycle_id,
        999_999,
        "Missing owner",
        None,
        0_i64,
        4_000,
    );

    // # Assert
    assert_eq!(visit_id, 1);
    assert!(owner_mismatch.is_err());
    assert!(missing_motorcycle.is_err());
    assert!(missing_owner.is_err());
    let invoice: InvoiceSnapshot = fixture
        .connection
        .query_row(
            "SELECT service_visit_id, status, invoice_number, issued_at,
                    cancelled_at, notes, created_at, updated_at
             FROM invoices WHERE service_visit_id = ?1",
            [visit_id],
            |row| {
                Ok(InvoiceSnapshot {
                    service_visit_id: row.get(0)?,
                    status: row.get(1)?,
                    invoice_number: row.get(2)?,
                    issued_at: row.get(3)?,
                    cancelled_at: row.get(4)?,
                    notes: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .expect("draft invoice should be created");
    assert_eq!(
        invoice,
        InvoiceSnapshot {
            service_visit_id: visit_id,
            status: "DRAFT".to_string(),
            invoice_number: None,
            issued_at: None,
            cancelled_at: None,
            notes: None,
            created_at: 1_000,
            updated_at: 1_000,
        }
    );
}

#[test]
fn owner_snapshot_survives_later_motorcycle_ownership_change() {
    // # Arrange
    let fixture = fixture();
    insert_closed_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        1_000,
    )
    .expect("historical visit should be inserted");
    let visit_id = fixture.connection.last_insert_rowid();

    // # Act
    fixture
        .connection
        .execute(
            "UPDATE motorcycles SET customer_id = ?1 WHERE id = ?2",
            (fixture.second_owner_id, fixture.motorcycle_id),
        )
        .expect("motorcycle ownership should change");

    // # Assert
    let snapshot_owner: i64 = fixture
        .connection
        .query_row(
            "SELECT owner_customer_id FROM service_visits WHERE id = ?1",
            [visit_id],
            |row| row.get(0),
        )
        .expect("snapshot owner should be queryable");
    assert_eq!(snapshot_owner, fixture.owner_id);
}

#[test]
fn one_active_visit_per_motorcycle_allows_unlimited_inactive_history() {
    // # Arrange
    let fixture = fixture();
    insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "First",
        None,
        0_i64,
        1_000,
    )
    .expect("first open visit should be inserted");
    let first_id = fixture.connection.last_insert_rowid();

    // # Act / # Assert
    assert!(insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "Second",
        None,
        0_i64,
        2_000,
    )
    .is_err());
    fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET status = 'READY_FOR_PICKUP', completed_at = 1100,
                 work_performed = 'Diagnostic work', updated_at = 1100
             WHERE id = ?1",
            [first_id],
        )
        .expect("open visit should become ready");
    assert!(insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "Still blocked",
        None,
        0_i64,
        2_000,
    )
    .is_err());
    fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET status = 'CLOSED', closed_at = 1200, updated_at = 1200
             WHERE id = ?1",
            [first_id],
        )
        .expect("ready visit should close");
    insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "After close",
        None,
        0_i64,
        2_000,
    )
    .expect("new visit should open after close");
    let second_id = fixture.connection.last_insert_rowid();
    fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET status = 'CANCELLED', cancelled_at = 2100,
                 cancellation_reason = 'Declined', labor_charge_fils = 5000,
                 updated_at = 2100
             WHERE id = ?1",
            [second_id],
        )
        .expect("open visit should cancel");
    let cancelled_labor: i64 = fixture
        .connection
        .query_row(
            "SELECT labor_charge_fils FROM service_visits WHERE id = ?1",
            [second_id],
            |row| row.get(0),
        )
        .expect("cancelled visit labor should be queryable");
    assert_eq!(cancelled_labor, 5_000);
    insert_closed_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        3_000,
    )
    .expect("a second closed visit should be allowed");
    insert_closed_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        4_000,
    )
    .expect("a third closed visit should be allowed");
    insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "After cancel",
        None,
        0_i64,
        5_000,
    )
    .expect("new visit should open after cancellation");
}

#[test]
fn database_enforces_odometer_and_integer_labor_boundaries() {
    for odometer in [None, Some(0), Some(9_999_999)] {
        let fixture = fixture();
        assert!(insert_open_visit(
            &fixture.connection,
            fixture.motorcycle_id,
            fixture.owner_id,
            "Complaint",
            odometer,
            5_000_i64,
            1_000,
        )
        .is_ok());
    }
    for odometer in [Some(-1), Some(10_000_000)] {
        let fixture = fixture();
        assert!(insert_open_visit(
            &fixture.connection,
            fixture.motorcycle_id,
            fixture.owner_id,
            "Complaint",
            odometer,
            0_i64,
            1_000,
        )
        .is_err());
    }

    let text_fixture = fixture();
    let text_odometer = text_fixture.connection.execute(
        "INSERT INTO service_visits (
            motorcycle_id, owner_customer_id, status, opened_at, odometer_km,
            customer_complaint, created_at, updated_at
         ) VALUES (?1, ?2, 'OPEN', 1000, 'not-an-integer', 'Complaint', 1000, 1000)",
        (text_fixture.motorcycle_id, text_fixture.owner_id),
    );
    assert!(text_odometer.is_err());

    let labor_fixture = fixture();
    assert!(insert_open_visit(
        &labor_fixture.connection,
        labor_fixture.motorcycle_id,
        labor_fixture.owner_id,
        "Complaint",
        None,
        -1_i64,
        1_000,
    )
    .is_err());
    let real_labor = insert_open_visit(
        &labor_fixture.connection,
        labor_fixture.second_motorcycle_id,
        labor_fixture.second_owner_id,
        "Complaint",
        None,
        1.5_f64,
        2_000,
    );
    assert!(real_labor.is_err());
}

#[test]
fn database_enforces_canonical_visit_text() {
    // # Arrange / # Act / # Assert
    let valid_complaints = [
        "English complaint".to_string(),
        "شكوى عربية".to_string(),
        "C".repeat(4_000),
    ];
    for complaint in valid_complaints {
        let fixture = fixture();
        assert!(insert_open_visit(
            &fixture.connection,
            fixture.motorcycle_id,
            fixture.owner_id,
            &complaint,
            None,
            0_i64,
            1_000,
        )
        .is_ok());
    }
    let invalid_complaints = ["".to_string(), " Complaint ".to_string(), "C".repeat(4_001)];
    for complaint in invalid_complaints {
        let fixture = fixture();
        assert!(insert_open_visit(
            &fixture.connection,
            fixture.motorcycle_id,
            fixture.owner_id,
            &complaint,
            None,
            0_i64,
            1_000,
        )
        .is_err());
    }

    for (column, value) in [
        ("diagnosis", " Diagnosis ".to_string()),
        ("work_performed", "".to_string()),
        ("notes", "N".repeat(4_001)),
    ] {
        let fixture = fixture();
        let result = fixture.connection.execute(
            &format!(
                "INSERT INTO service_visits (
                    motorcycle_id, owner_customer_id, status, opened_at,
                    customer_complaint, {column}, created_at, updated_at
                 ) VALUES (?1, ?2, 'OPEN', 1000, 'Complaint', ?3, 1000, 1000)"
            ),
            params![fixture.motorcycle_id, fixture.owner_id, value],
        );
        assert!(result.is_err(), "invalid optional field: {column}");
    }

    for reason in [" Declined ".to_string(), "R".repeat(1_001)] {
        let fixture = fixture();
        assert!(insert_visit_state(
            &fixture.connection,
            fixture.motorcycle_id,
            fixture.owner_id,
            VisitState {
                status: "CANCELLED",
                completed_at: None,
                closed_at: None,
                cancelled_at: Some(1_100),
                work_performed: None,
                cancellation_reason: Some(&reason),
            },
        )
        .is_err());
    }
}

#[test]
fn database_enforces_status_rows_transitions_and_chronology() {
    // # Arrange
    let valid_rows = [
        ("OPEN", None, None, None, None, None),
        (
            "READY_FOR_PICKUP",
            Some(1_100),
            None,
            None,
            Some("Work"),
            None,
        ),
        ("CLOSED", Some(1_100), Some(1_200), None, Some("Work"), None),
        ("CANCELLED", None, None, Some(1_100), None, Some("Declined")),
    ];
    for (status, completed, closed, cancelled, work, reason) in valid_rows {
        let fixture = fixture();
        assert!(
            insert_visit_state(
                &fixture.connection,
                fixture.motorcycle_id,
                fixture.owner_id,
                VisitState {
                    status,
                    completed_at: completed,
                    closed_at: closed,
                    cancelled_at: cancelled,
                    work_performed: work,
                    cancellation_reason: reason
                },
            )
            .is_ok(),
            "valid status: {status}"
        );
    }

    let invalid_rows = [
        ("UNKNOWN", None, None, None, None, None),
        ("OPEN", Some(1_100), None, None, None, None),
        ("READY_FOR_PICKUP", None, None, None, Some("Work"), None),
        (
            "READY_FOR_PICKUP",
            Some(999),
            None,
            None,
            Some("Work"),
            None,
        ),
        ("READY_FOR_PICKUP", Some(1_100), None, None, None, None),
        ("CLOSED", Some(1_100), Some(1_099), None, Some("Work"), None),
        ("CANCELLED", None, None, Some(999), None, Some("Declined")),
        ("CANCELLED", None, None, Some(1_100), None, None),
        ("OPEN", None, None, None, None, Some("Not allowed")),
    ];
    for (status, completed, closed, cancelled, work, reason) in invalid_rows {
        let fixture = fixture();
        assert!(
            insert_visit_state(
                &fixture.connection,
                fixture.motorcycle_id,
                fixture.owner_id,
                VisitState {
                    status,
                    completed_at: completed,
                    closed_at: closed,
                    cancelled_at: cancelled,
                    work_performed: work,
                    cancellation_reason: reason
                },
            )
            .is_err(),
            "invalid status row: {status}"
        );
    }

    let fixture = fixture();
    insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "Complaint",
        None,
        0_i64,
        1_000,
    )
    .expect("visit should open");
    let visit_id = fixture.connection.last_insert_rowid();
    assert!(fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'CLOSED', completed_at = 1100,
         closed_at = 1200, work_performed = 'Work' WHERE id = ?1",
            [visit_id]
        )
        .is_err());
    assert!(fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'READY_FOR_PICKUP', completed_at = 1100,
         work_performed = 'Work' WHERE id = ?1",
            [visit_id]
        )
        .is_ok());
    assert!(fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'CANCELLED', completed_at = NULL,
         cancelled_at = 1200, cancellation_reason = 'Declined' WHERE id = ?1",
            [visit_id]
        )
        .is_err());
    assert!(fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'OPEN', completed_at = NULL WHERE id = ?1",
            [visit_id]
        )
        .is_ok());
    assert!(fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'CANCELLED', cancelled_at = 1200,
         cancellation_reason = 'Declined' WHERE id = ?1",
            [visit_id]
        )
        .is_ok());
}

#[test]
fn historical_identity_terminal_rows_and_deletion_are_protected() {
    // # Arrange
    let fixture = fixture();
    insert_closed_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        1_000,
    )
    .expect("closed visit should be inserted");
    let visit_id = fixture.connection.last_insert_rowid();
    insert_open_visit(
        &fixture.connection,
        fixture.second_motorcycle_id,
        fixture.second_owner_id,
        "Active visit",
        None,
        0_i64,
        2_000,
    )
    .expect("active visit should be inserted");
    let active_visit_id = fixture.connection.last_insert_rowid();

    // # Act / # Assert
    for update in [
        "UPDATE service_visits SET motorcycle_id = 999 WHERE id = ?1",
        "UPDATE service_visits SET owner_customer_id = 999 WHERE id = ?1",
        "UPDATE service_visits SET opened_at = 999 WHERE id = ?1",
    ] {
        assert!(fixture
            .connection
            .execute(update, [active_visit_id])
            .is_err());
    }
    assert!(fixture
        .connection
        .execute(
            "UPDATE service_visits SET notes = 'Changed' WHERE id = ?1",
            [visit_id]
        )
        .is_err());
    assert!(fixture.connection.execute(
        "UPDATE service_visits SET status = 'OPEN', completed_at = NULL, closed_at = NULL WHERE id = ?1",
        [visit_id]
    ).is_err());
    assert!(fixture
        .connection
        .execute("DELETE FROM service_visits WHERE id = ?1", [visit_id])
        .is_err());
}

#[test]
fn invoice_cardinality_identity_and_deletion_are_protected() {
    // # Arrange
    let fixture = fixture();
    insert_open_visit(
        &fixture.connection,
        fixture.motorcycle_id,
        fixture.owner_id,
        "One",
        None,
        0_i64,
        1_000,
    )
    .expect("first visit should open");
    let first_visit_id = fixture.connection.last_insert_rowid();
    insert_open_visit(
        &fixture.connection,
        fixture.second_motorcycle_id,
        fixture.second_owner_id,
        "Two",
        None,
        0_i64,
        2_000,
    )
    .expect("second visit should open");
    let second_visit_id = fixture.connection.last_insert_rowid();
    let first_invoice_id: i64 = fixture
        .connection
        .query_row(
            "SELECT id FROM invoices WHERE service_visit_id = ?1",
            [first_visit_id],
            |row| row.get(0),
        )
        .expect("first invoice should exist");

    // # Act / # Assert
    let invoice_count: i64 = fixture
        .connection
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE service_visit_id IN (?1, ?2)",
            (first_visit_id, second_visit_id),
            |row| row.get(0),
        )
        .expect("invoice count should be queryable");
    assert_eq!(invoice_count, 2);
    assert!(fixture
        .connection
        .execute(
            "INSERT INTO invoices (service_visit_id, status, created_at, updated_at)
         VALUES (?1, 'DRAFT', 1000, 1000)",
            [first_visit_id]
        )
        .is_err());
    assert!(fixture
        .connection
        .execute(
            "UPDATE invoices SET service_visit_id = ?1 WHERE id = ?2",
            (second_visit_id, first_invoice_id)
        )
        .is_err());
    assert!(fixture
        .connection
        .execute("DELETE FROM invoices WHERE id = ?1", [first_invoice_id])
        .is_err());
    assert!(fixture
        .connection
        .execute("DELETE FROM service_visits WHERE id = ?1", [first_visit_id])
        .is_err());
}

#[derive(Debug, PartialEq, Eq)]
struct InvoiceSnapshot {
    service_visit_id: i64,
    status: String,
    invoice_number: Option<String>,
    issued_at: Option<i64>,
    cancelled_at: Option<i64>,
    notes: Option<String>,
    created_at: i64,
    updated_at: i64,
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
    owner_id: i64,
    second_owner_id: i64,
    motorcycle_id: i64,
    second_motorcycle_id: i64,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().expect("temporary directory should be created");
    let database_path = temp_dir.path().join("test.db");
    let mut connection = open_database(&database_path).expect("database should open");
    migrate_database(&mut connection).expect("database should migrate");
    let owner_id = insert_customer(&connection, "Ahmad", "+962791111111");
    let second_owner_id = insert_customer(&connection, "Omar", "+962792222222");
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
    let motorcycle_id = insert_motorcycle(&connection, owner_id, make_id, color_id, "FRAME/ONE");
    let second_motorcycle_id =
        insert_motorcycle(&connection, second_owner_id, make_id, color_id, "FRAME/TWO");
    Fixture {
        _temp_dir: temp_dir,
        connection,
        owner_id,
        second_owner_id,
        motorcycle_id,
        second_motorcycle_id,
    }
}

fn insert_customer(connection: &Connection, name: &str, phone: &str) -> i64 {
    connection.execute(
        "INSERT INTO customers (name, phone, created_at, updated_at) VALUES (?1, ?2, 1000, 1000)",
        (name, phone),
    ).expect("customer should be inserted");
    connection.last_insert_rowid()
}

fn insert_motorcycle(
    connection: &Connection,
    owner_id: i64,
    make_id: i64,
    color_id: i64,
    chassis: &str,
) -> i64 {
    connection
        .execute(
            "INSERT INTO motorcycles (
            customer_id, make_id, model, chassis_number, color_id, created_at, updated_at
         ) VALUES (?1, ?2, 'Model', ?3, ?4, 1000, 1000)",
            (owner_id, make_id, chassis, color_id),
        )
        .expect("motorcycle should be inserted");
    connection.last_insert_rowid()
}

fn insert_open_visit(
    connection: &Connection,
    motorcycle_id: i64,
    owner_customer_id: i64,
    complaint: &str,
    odometer_km: Option<i64>,
    labor_charge_fils: impl rusqlite::ToSql,
    created_at: i64,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO service_visits (
            motorcycle_id, owner_customer_id, status, opened_at, odometer_km,
            customer_complaint, labor_charge_fils, created_at, updated_at
         ) VALUES (?1, ?2, 'OPEN', ?3, ?4, ?5, ?6, ?3, ?3)",
        params![
            motorcycle_id,
            owner_customer_id,
            created_at,
            odometer_km,
            complaint,
            labor_charge_fils
        ],
    )
}

struct VisitState<'a> {
    status: &'a str,
    completed_at: Option<i64>,
    closed_at: Option<i64>,
    cancelled_at: Option<i64>,
    work_performed: Option<&'a str>,
    cancellation_reason: Option<&'a str>,
}

fn insert_visit_state(
    connection: &Connection,
    motorcycle_id: i64,
    owner_customer_id: i64,
    state: VisitState<'_>,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO service_visits (
            motorcycle_id, owner_customer_id, status, opened_at, completed_at,
            closed_at, cancelled_at, customer_complaint, work_performed,
            cancellation_reason, created_at, updated_at
         ) VALUES (?1, ?2, ?3, 1000, ?4, ?5, ?6, 'Complaint', ?7, ?8, 1000, 1000)",
        params![
            motorcycle_id,
            owner_customer_id,
            state.status,
            state.completed_at,
            state.closed_at,
            state.cancelled_at,
            state.work_performed,
            state.cancellation_reason
        ],
    )
}

fn insert_closed_visit(
    connection: &Connection,
    motorcycle_id: i64,
    owner_customer_id: i64,
    opened_at: i64,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO service_visits (
            motorcycle_id, owner_customer_id, status, opened_at, completed_at,
            closed_at, customer_complaint, work_performed, created_at, updated_at
         ) VALUES (?1, ?2, 'CLOSED', ?3, ?3 + 100, ?3 + 200,
                   'Complaint', 'Work', ?3, ?3 + 200)",
        (motorcycle_id, owner_customer_id, opened_at),
    )
}

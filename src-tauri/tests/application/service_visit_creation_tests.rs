use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::service_visit_workspace::{
        CreateServiceVisitInput, ServiceVisitWorkspaceError, ServiceVisitWorkspaceService,
    },
    db::{migrate_database, open_database},
    domain::service_visit::{ServiceVisitStatus, ServiceVisitValidationError},
};

type InvoiceSnapshot = (
    i64,
    String,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    i64,
    i64,
);

#[test]
fn creates_open_visit_from_authoritative_owner_with_normalized_values_and_one_draft_invoice() {
    // # Arrange
    let mut fixture = fixture();
    let motorcycle_id = insert_motorcycle(
        &fixture.connection,
        fixture.owner_id,
        "CREATION-OWNER-SNAPSHOT",
    );

    // # Act
    let created = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .create_service_visit(CreateServiceVisitInput {
            motorcycle_id,
            opened_at: 2_000,
            odometer_km: Some(18_750),
            customer_complaint: "  Engine stalls at idle  ".into(),
            notes: Some("  Customer will wait  ".into()),
            created_at: 2_100,
        })
        .expect("valid visit should be created");

    // # Assert
    assert_eq!(created.visit.status, ServiceVisitStatus::Open);
    assert_eq!(created.visit.motorcycle_id, motorcycle_id);
    assert_eq!(created.visit.owner_customer_id, fixture.owner_id);
    assert_eq!(created.visit.opened_at, 2_000);
    assert_eq!(created.visit.odometer_km, Some(18_750));
    assert_eq!(created.visit.customer_complaint, "Engine stalls at idle");
    assert_eq!(created.visit.notes.as_deref(), Some("Customer will wait"));
    assert_eq!(created.visit.diagnosis, None);
    assert_eq!(created.visit.work_performed, None);
    assert_eq!(created.visit.labor_charge_fils, 0);
    assert_eq!(created.visit.completed_at, None);
    assert_eq!(created.visit.closed_at, None);
    assert_eq!(created.visit.cancelled_at, None);
    assert_eq!(created.visit.cancellation_reason, None);
    assert_eq!(created.visit.created_at, 2_100);
    assert_eq!(created.visit.updated_at, 2_100);
    assert_eq!(created.owner.id, fixture.owner_id);
    assert_eq!(created.owner.name, "Ahmad Ali");
    assert_eq!(created.motorcycle.id, motorcycle_id);
    assert!(created.parts.is_empty());

    let invoice: InvoiceSnapshot = fixture
        .connection
        .query_row(
            "SELECT COUNT(*), status, invoice_number, issued_at, cancelled_at,
                    notes, created_at, updated_at
             FROM invoices WHERE service_visit_id = ?1",
            [created.visit.id],
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
                ))
            },
        )
        .unwrap();
    assert_eq!(
        invoice,
        (1, "DRAFT".into(), None, None, None, None, 2_100, 2_100)
    );

    let new_owner_id = insert_customer(&fixture.connection, "Maya Saleh", "+962791234568");
    fixture
        .connection
        .execute(
            "UPDATE motorcycles SET customer_id = ?1 WHERE id = ?2",
            (new_owner_id, motorcycle_id),
        )
        .unwrap();
    let historical = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .load_workspace(created.visit.id)
        .unwrap();
    assert_eq!(historical.visit.owner_customer_id, fixture.owner_id);
    assert_eq!(historical.owner.id, fixture.owner_id);
    assert_eq!(historical.owner.name, "Ahmad Ali");
}

#[test]
fn creation_returns_typed_missing_active_and_domain_validation_errors_without_side_effects() {
    // # Arrange
    let mut fixture = fixture();
    let invalid_motorcycle_id =
        insert_motorcycle(&fixture.connection, fixture.owner_id, "CREATION-INVALID");

    // # Act
    let missing = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .create_service_visit(valid_input(999_999, 2_000))
        .expect_err("missing motorcycle should be typed");
    let active_open = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .create_service_visit(valid_input(fixture.motorcycle_id, 2_000))
        .expect_err("open visit should block another active visit");
    fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET work_performed = 'Completed work', status = 'READY_FOR_PICKUP',
                 completed_at = 1500, updated_at = 1500
             WHERE id = ?1",
            [fixture.visit_id],
        )
        .unwrap();
    let active_ready = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .create_service_visit(valid_input(fixture.motorcycle_id, 2_100))
        .expect_err("ready visit should block another active visit");
    let invalid_odometer = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .create_service_visit(CreateServiceVisitInput {
            motorcycle_id: invalid_motorcycle_id,
            opened_at: 2_000,
            odometer_km: Some(10_000_000),
            customer_complaint: "Engine stalls".into(),
            notes: None,
            created_at: 2_000,
        })
        .expect_err("domain odometer limit must be used");

    // # Assert
    assert!(matches!(
        missing,
        ServiceVisitWorkspaceError::MotorcycleNotFound(999_999)
    ));
    assert!(matches!(
        active_open,
        ServiceVisitWorkspaceError::ActiveServiceVisitExists(id)
            if id == fixture.motorcycle_id
    ));
    assert!(matches!(
        active_ready,
        ServiceVisitWorkspaceError::ActiveServiceVisitExists(id)
            if id == fixture.motorcycle_id
    ));
    assert!(matches!(
        invalid_odometer,
        ServiceVisitWorkspaceError::VisitValidation(ServiceVisitValidationError::InvalidOdometer)
    ));
    let rejected_visits: i64 = fixture
        .connection
        .query_row(
            "SELECT COUNT(*) FROM service_visits WHERE motorcycle_id = ?1",
            [invalid_motorcycle_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(rejected_visits, 0);
}

#[test]
fn creation_is_allowed_after_closed_or_cancelled_history() {
    // # Arrange
    let mut closed_fixture = fixture();
    closed_fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET work_performed = 'Completed work', status = 'READY_FOR_PICKUP',
                 completed_at = 1500, updated_at = 1500
             WHERE id = ?1",
            [closed_fixture.visit_id],
        )
        .unwrap();
    closed_fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'CLOSED', closed_at = 1600, updated_at = 1600
             WHERE id = ?1",
            [closed_fixture.visit_id],
        )
        .unwrap();
    let mut cancelled_fixture = fixture();
    cancelled_fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET status = 'CANCELLED', cancelled_at = 1500,
                 cancellation_reason = 'Customer declined', updated_at = 1500
             WHERE id = ?1",
            [cancelled_fixture.visit_id],
        )
        .unwrap();

    // # Act
    let after_closed = ServiceVisitWorkspaceService::new(&mut closed_fixture.connection)
        .create_service_visit(valid_input(closed_fixture.motorcycle_id, 2_000))
        .expect("closed history should allow a new visit");
    let after_cancelled = ServiceVisitWorkspaceService::new(&mut cancelled_fixture.connection)
        .create_service_visit(valid_input(cancelled_fixture.motorcycle_id, 2_000))
        .expect("cancelled history should allow a new visit");

    // # Assert
    assert_eq!(after_closed.visit.status, ServiceVisitStatus::Open);
    assert_eq!(after_cancelled.visit.status, ServiceVisitStatus::Open);
    assert_ne!(after_closed.visit.id, closed_fixture.visit_id);
    assert_ne!(after_cancelled.visit.id, cancelled_fixture.visit_id);
    assert_eq!(
        visit_and_invoice_counts(&closed_fixture.connection, closed_fixture.motorcycle_id),
        (2, 2)
    );
    assert_eq!(
        visit_and_invoice_counts(
            &cancelled_fixture.connection,
            cancelled_fixture.motorcycle_id,
        ),
        (2, 2)
    );
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
    owner_id: i64,
    motorcycle_id: i64,
    visit_id: i64,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("creation-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    let owner_id = insert_customer(&connection, "Ahmad Ali", "+962791234567");
    let motorcycle_id = insert_motorcycle(&connection, owner_id, "CREATION-BASE");
    connection
        .execute(
            "INSERT INTO service_visits (
                motorcycle_id, owner_customer_id, status, opened_at,
                customer_complaint, created_at, updated_at
             ) VALUES (?1, ?2, 'OPEN', 1000, 'Oil leak', 1000, 1000)",
            (motorcycle_id, owner_id),
        )
        .unwrap();
    let visit_id = connection.last_insert_rowid();
    Fixture {
        _temp_dir: temp_dir,
        connection,
        owner_id,
        motorcycle_id,
        visit_id,
    }
}

fn insert_customer(connection: &Connection, name: &str, phone: &str) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES (?1, ?2, 1000, 1000)",
            (name, phone),
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_motorcycle(connection: &Connection, owner_id: i64, chassis_number: &str) -> i64 {
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, chassis_number, color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'CB150R', ?3, ?4, 1000, 1000)",
            params![owner_id, make_id, chassis_number, color_id],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn valid_input(motorcycle_id: i64, timestamp: i64) -> CreateServiceVisitInput {
    CreateServiceVisitInput {
        motorcycle_id,
        opened_at: timestamp,
        odometer_km: Some(18_750),
        customer_complaint: "Engine stalls".into(),
        notes: None,
        created_at: timestamp,
    }
}

fn visit_and_invoice_counts(connection: &Connection, motorcycle_id: i64) -> (i64, i64) {
    connection
        .query_row(
            "SELECT COUNT(DISTINCT v.id), COUNT(DISTINCT i.id)
             FROM service_visits v
             LEFT JOIN invoices i ON i.service_visit_id = v.id
             WHERE v.motorcycle_id = ?1",
            [motorcycle_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
}
